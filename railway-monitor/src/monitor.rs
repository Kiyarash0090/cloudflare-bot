use crate::bale::BaleClient;
use grammers_client::{Client, types::Update};
use grammers_tl_types as tl;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

pub struct ChannelMonitor {
    client: Client,
    monitored_channels: HashSet<i64>,
    channel_names: HashMap<i64, String>,
    sync_channel: String,
    bale_client: BaleClient,
    bale_chat_id: i64,
    processed_messages: Arc<RwLock<HashSet<i64>>>,
}

impl ChannelMonitor {
    pub fn new(
        client: Client,
        channels: Vec<crate::config::MonitoredChannel>,
        sync_channel: String,
        bale_token: String,
        bale_chat_id: i64,
    ) -> Self {
        let mut monitored_channels = HashSet::new();
        let mut channel_names = HashMap::new();

        for ch in channels {
            monitored_channels.insert(ch.id);
            channel_names.insert(ch.id, ch.name);
        }

        Self {
            client,
            monitored_channels,
            channel_names,
            sync_channel,
            bale_client: BaleClient::new(bale_token),
            bale_chat_id,
            processed_messages: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        info!("Starting channel monitor for {} channels", self.monitored_channels.len());
        info!("Sync channel: {}", self.sync_channel);

        // Get the sync channel entity
        let sync_entity = self.resolve_sync_channel().await?;

        loop {
            // Get updates
            let updates = self.client.next_updates().await?;

            for update in updates {
                if let Err(e) = self.handle_update(update, sync_entity.clone()).await {
                    error!("Error handling update: {}", e);
                }
            }

            // Small delay to prevent busy loop
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn resolve_sync_channel(&self) -> anyhow::Result<tl::enums::InputPeer> {
        // Try to find the sync channel by username
        let dialogs = self.client.iter_dialogs().collect::<Vec<_>>().await?;

        for dialog in dialogs {
            if let Some(username) = &dialog.entity.username() {
                if username == &self.sync_channel {
                    return Ok(dialog.entity.into());
                }
            }
        }

        // If not found by username, try to resolve
        let entity = self.client.resolve_username(&self.sync_channel).await?;
        Ok(entity.into())
    }

    async fn handle_update(&self, update: Update, sync_entity: tl::enums::InputPeer) -> anyhow::Result<()> {
        // Only handle new channel messages
        let message = match update {
            Update::NewMessage(msg) => msg,
            _ => return Ok(()),
        };

        // Check if message is from a monitored channel
        let chat = match message.chat() {
            Some(c) => c,
            None => return Ok(()),
        };

        let chat_id = chat.id();

        if !self.monitored_channels.contains(&chat_id) {
            return Ok(());
        }

        // Deduplication
        let msg_id = message.id();
        {
            let mut processed = self.processed_messages.write().await;
            if processed.contains(&msg_id) {
                return Ok(());
            }
            processed.insert(msg_id);

            // Keep only last 10000 message IDs
            if processed.len() > 10000 {
                let to_remove: Vec<_> = processed.iter().take(processed.len() - 10000).cloned().collect();
                for id in to_remove {
                    processed.remove(&id);
                }
            }
        }

        let channel_name = self.channel_names.get(&chat_id).cloned().unwrap_or_default();

        info!("New message from {}: {}", channel_name, msg_id);

        // Forward to Bale
        if let Err(e) = self.forward_to_bale(&message, &channel_name).await {
            error!("Failed to forward to Bale: {}", e);
        }

        // Forward to sync channel in Telegram
        if let Err(e) = self.forward_to_sync_channel(&message, &channel_name, sync_entity.clone()).await {
            error!("Failed to forward to sync channel: {}", e);
        }

        Ok(())
    }

    async fn forward_to_bale(&self, message: &grammers_client::types::Message, channel_name: &str) -> anyhow::Result<()> {
        let text = match &message.text() {
            Some(t) => t.to_string(),
            None => String::new(),
        };

        let caption = if !text.is_empty() {
            Some(format!("📢 *{}*\n\n{}", channel_name, escape_markdown(t)))
        } else {
            Some(format!("📢 *{}*", channel_name))
        };

        // Handle different media types
        if let Some(media) = message.media() {
            match media {
                tl::enums::MessageMedia::Photo(photo) => {
                    if let tl::enums::Photo::Photo(p) = photo {
                        self.bale_client.send_photo(self.bale_chat_id, &p.id.to_string(), caption.as_deref()).await?;
                    }
                }
                tl::enums::MessageMedia::Document(doc) => {
                    if let tl::enums::Document::Document(d) = doc {
                        let file_name = d.attributes.iter().find_map(|attr| {
                            if let tl::enums::DocumentAttribute::Filename { file_name } = attr {
                                Some(file_name.clone())
                            } else {
                                None
                            }
                        });
                        self.bale_client.send_document(self.bale_chat_id, &d.id.to_string(), file_name.as_deref(), caption.as_deref()).await?;
                    }
                }
                tl::enums::MessageMedia::Video(video) => {
                    if let tl::enums::Video::Video(v) = video {
                        self.bale_client.send_video(self.bale_chat_id, &v.id.to_string(), caption.as_deref()).await?;
                    }
                }
                _ => {
                    // Text only
                    self.bale_client.send_message(self.bale_chat_id, &caption.unwrap()).await?;
                }
            }
        } else {
            // Text only
            self.bale_client.send_message(self.bale_chat_id, &caption.unwrap()).await?;
        }

        Ok(())
    }

    async fn forward_to_sync_channel(
        &self,
        message: &grammers_client::types::Message,
        channel_name: &str,
        sync_entity: tl::enums::InputPeer,
    ) -> anyhow::Result<()> {
        // Forward the message to sync channel with a header
        let header = format!("📢 **{}** (ID: {})\n\n", channel_name, message.chat().map(|c| c.id()).unwrap_or(0));

        // Try to forward directly
        self.client
            .forward_messages(&sync_entity, message.chat().unwrap().id(), &[message.id()])
            .await?;

        // Send header as separate message
        self.client
            .send_message(&sync_entity, header)
            .await?;

        Ok(())
    }
}

fn escape_markdown(text: &str) -> String {
    text.replace('_', "\\_")
        .replace('*', "\\*")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('~', "\\~")
        .replace('`', "\\`")
        .replace('>', "\\>")
        .replace('#', "\\#")
        .replace('+', "\\+")
        .replace('-', "\\-")
        .replace('=', "\\=")
        .replace('|', "\\|")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('.', "\\.")
        .replace('!', "\\!")
}