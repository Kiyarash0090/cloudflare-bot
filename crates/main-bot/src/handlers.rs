use crate::bale_api::BaleBot;
use shared::{BaleUpdate, Channel, ChannelInput, D1Storage, KVStorage, Result, Error};

pub struct CommandHandler {
    bale: BaleBot,
    d1: D1Storage,
    kv: KVStorage,
    admin_id: i64,
}

impl CommandHandler {
    pub fn new(bale: BaleBot, d1: D1Storage, kv: KVStorage, admin_id: i64) -> Self {
        Self {
            bale,
            d1,
            kv,
            admin_id,
        }
    }

    pub async fn handle_update(&self, update: BaleUpdate) -> Result<()> {
        let message = match update.message {
            Some(msg) => msg,
            None => return Ok(()), // Ignore non-message updates
        };

        let user_id = match &message.from {
            Some(user) => user.id,
            None => return Ok(()),
        };

        // Check admin access
        if !self.bale.is_admin(user_id, self.admin_id) {
            self.bale
                .reply_to(message.message_id, "❌ شما اجازه استفاده از این ربات را ندارید.")
                .await?;
            return Ok(());
        }

        // Check rate limit
        let rate_limit_key = format!("command:{}", user_id);
        if !self.kv.check_rate_limit(&rate_limit_key, 10).await? {
            return Ok(()); // Silently ignore if rate limited
        }

        let text = match message.text {
            Some(ref t) => t.trim(),
            None => return Ok(()),
        };

        // Route commands
        if text.starts_with("/start") {
            self.handle_start(&message.message_id).await?;
        } else if text.starts_with("/help") {
            self.handle_help(&message.message_id).await?;
        } else if text.starts_with("/add_channel") {
            self.handle_add_channel(text, &message.message_id).await?;
        } else if text.starts_with("/remove_channel") {
            self.handle_remove_channel(text, &message.message_id).await?;
        } else if text.starts_with("/list_channels") {
            self.handle_list_channels(&message.message_id).await?;
        } else if text.starts_with("/start_monitor") {
            self.handle_start_monitor(&message.message_id).await?;
        } else if text.starts_with("/stop_monitor") {
            self.handle_stop_monitor(&message.message_id).await?;
        } else if text.starts_with("/status") {
            self.handle_status(&message.message_id).await?;
        }

        Ok(())
    }

    async fn handle_start(&self, message_id: &i64) -> Result<()> {
        let text = "🤖 *خوش آمدید!*\n\n\
            این ربات کانال‌های تلگرام را مانیتور کرده و پست‌ها را به بله ارسال می‌کند.\n\n\
            *دستورات موجود:*\n\
            /add\\_channel - افزودن کانال\n\
            /remove\\_channel - حذف کانال\n\
            /list\\_channels - لیست کانال‌ها\n\
            /start\\_monitor - شروع مانیتورینگ\n\
            /stop\\_monitor - توقف مانیتورینگ\n\
            /status - وضعیت فعلی\n\
            /help - راهنما\n\n\
            برای شروع، ابتدا کانال‌های مورد نظر را اضافه کنید.";

        self.bale.reply_to(*message_id, text).await?;
        Ok(())
    }

    async fn handle_help(&self, message_id: &i64) -> Result<()> {
        let text = "📖 *راهنمای استفاده*\n\n\
            *افزودن کانال:*\n\
            `/add_channel @username` - افزودن ساده\n\
            `/add_channel @username +100` - فایل‌های 20-100MB فشرده می‌شوند\n\
            `/add_channel @username +zip50` - همه فایل‌ها فشرده و split می‌شوند\n\n\
            *حذف کانال:*\n\
            `/remove_channel @username`\n\n\
            *لیست کانال‌ها:*\n\
            `/list_channels` - نمایش تمام کانال‌ها\n\n\
            *کنترل مانیتورینگ:*\n\
            `/start_monitor` - شروع forward از تلگرام به بله\n\
            `/stop_monitor` - توقف مانیتورینگ\n\n\
            *وضعیت:*\n\
            `/status` - نمایش وضعیت فعلی ربات";

        self.bale.reply_to(*message_id, text).await?;
        Ok(())
    }

    async fn handle_add_channel(&self, text: &str, message_id: &i64) -> Result<()> {
        let input = text.trim_start_matches("/add_channel").trim();
        
        if input.is_empty() {
            self.bale
                .reply_to(*message_id, "❌ لطفاً username یا ID کانال را وارد کنید.\n\nمثال: `/add_channel @channelname`")
                .await?;
            return Ok(());
        }

        let parsed = match ChannelInput::parse(input) {
            Ok(p) => p,
            Err(e) => {
                self.bale
                    .reply_to(*message_id, &format!("❌ خطا در parse کردن ورودی: {}", e))
                    .await?;
                return Ok(());
            }
        };

        // For MVP, we need to get channel info from Telegram
        // This will be implemented in Telegram Poller
        // For now, create a basic channel entry
        
        let channel = Channel {
            id: None,
            telegram_id: 0, // Will be filled by Telegram Poller
            name: parsed.identifier.clone(),
            username: Some(parsed.identifier.clone()),
            link: None,
            custom_max_size_mb: parsed.custom_size,
            zip_max_size_mb: parsed.zip_size,
            enabled: true,
            added_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        };

        match self.d1.add_channel(&channel).await {
            Ok(_) => {
                let response = format!(
                    "✅ کانال `{}` با موفقیت اضافه شد.\n\n\
                    تنظیمات:\n\
                    - Custom size: {}\n\
                    - Zip size: {}\n\n\
                    ⚠️ توجه: برای فعال شدن، `/start_monitor` را اجرا کنید.",
                    parsed.identifier,
                    parsed.custom_size.map(|s| format!("{}MB", s)).unwrap_or_else(|| "None".to_string()),
                    parsed.zip_size.map(|s| format!("{}MB", s)).unwrap_or_else(|| "None".to_string())
                );
                self.bale.reply_to(*message_id, &response).await?;
            }
            Err(e) => {
                self.bale
                    .reply_to(*message_id, &format!("❌ خطا در افزودن کانال: {}", e))
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_remove_channel(&self, text: &str, message_id: &i64) -> Result<()> {
        let input = text.trim_start_matches("/remove_channel").trim();
        
        if input.is_empty() {
            self.bale
                .reply_to(*message_id, "❌ لطفاً username یا ID کانال را وارد کنید.\n\nمثال: `/remove_channel @channelname`")
                .await?;
            return Ok(());
        }

        let identifier = input.trim_start_matches('@');

        // Find channel by username
        let channels = self.d1.get_all_channels().await?;
        let channel = channels.iter().find(|c| {
            c.username.as_ref().map(|u| u == identifier).unwrap_or(false)
        });

        match channel {
            Some(ch) => {
                self.d1.remove_channel(ch.telegram_id).await?;
                self.bale
                    .reply_to(*message_id, &format!("✅ کانال `{}` با موفقیت حذف شد.", identifier))
                    .await?;
            }
            None => {
                self.bale
                    .reply_to(*message_id, &format!("❌ کانال `{}` یافت نشد.", identifier))
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_list_channels(&self, message_id: &i64) -> Result<()> {
        let channels = self.d1.get_all_channels().await?;

        if channels.is_empty() {
            self.bale
                .reply_to(*message_id, "📋 هیچ کانالی ثبت نشده است.\n\nبرای افزودن: `/add_channel @username`")
                .await?;
            return Ok(());
        }

        let mut text = format!("📋 *لیست کانال‌ها* ({} کانال)\n\n", channels.len());

        for (idx, channel) in channels.iter().enumerate() {
            let status = if channel.enabled { "✅" } else { "❌" };
            let username = channel.username.as_deref().unwrap_or("N/A");
            let max_size = channel.max_file_size_mb();
            
            text.push_str(&format!(
                "{}. {} `{}`\n   📊 Max size: {}MB\n   🆔 ID: {}\n\n",
                idx + 1,
                status,
                username,
                max_size,
                channel.telegram_id
            ));
        }

        self.bale.reply_to(*message_id, &text).await?;
        Ok(())
    }

    async fn handle_start_monitor(&self, message_id: &i64) -> Result<()> {
        let state = self.d1.get_monitoring_state().await?;

        if state.is_running {
            self.bale
                .reply_to(*message_id, "ℹ️ مانیتورینگ در حال حاضر فعال است.")
                .await?;
            return Ok(());
        }

        self.d1.start_monitoring().await?;
        
        self.bale
            .reply_to(*message_id, "✅ مانیتورینگ شروع شد!\n\nپست‌های جدید کانال‌ها به بله ارسال می‌شوند.\n\n⏰ تاخیر: حدود 1 دقیقه (Cron)")
            .await?;

        Ok(())
    }

    async fn handle_stop_monitor(&self, message_id: &i64) -> Result<()> {
        let state = self.d1.get_monitoring_state().await?;

        if !state.is_running {
            self.bale
                .reply_to(*message_id, "ℹ️ مانیتورینگ در حال حاضر متوقف است.")
                .await?;
            return Ok(());
        }

        self.d1.stop_monitoring().await?;
        
        self.bale
            .reply_to(*message_id, "⏸️ مانیتورینگ متوقف شد.\n\nبرای شروع مجدد: `/start_monitor`")
            .await?;

        Ok(())
    }

    async fn handle_status(&self, message_id: &i64) -> Result<()> {
        let state = self.d1.get_monitoring_state().await?;
        let all_channels = self.d1.get_all_channels().await?;
        let enabled_channels = all_channels.iter().filter(|c| c.enabled).count();

        let status_emoji = if state.is_running { "✅" } else { "⏸️" };
        let status_text = if state.is_running { "فعال" } else { "غیرفعال" };

        let mut text = format!(
            "📊 *وضعیت ربات*\n\n\
            {} مانیتورینگ: *{}*\n\
            📋 کانال‌ها: {} کل / {} فعال\n\
            🔢 آخرین Update ID: {}\n\n",
            status_emoji,
            status_text,
            all_channels.len(),
            enabled_channels,
            state.last_update_id
        );

        if let Some(started) = state.started_at {
            text.push_str(&format!("🕐 شروع: {}\n", shared::utils::format_timestamp(started)));
        }

        if let Some(stopped) = state.stopped_at {
            text.push_str(&format!("🕐 توقف: {}\n", shared::utils::format_timestamp(stopped)));
        }

        self.bale.reply_to(*message_id, &text).await?;
        Ok(())
    }
}
