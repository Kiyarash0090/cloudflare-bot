use crate::bale_api::BaleApi;
use crate::handlers::BotHandler;
use shared::{D1Storage, KVStorage, Channel};
use worker::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramUpdate {
    pub update_id: i64,
    pub channel_post: Option<TelegramMessage>,
    pub message: Option<TelegramMessage>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramMessage {
    pub message_id: i64,
    pub chat: TelegramChat,
    pub text: Option<String>,
    pub caption: Option<String>,
    pub photo: Option<Vec<TelegramPhotoSize>>,
    pub document: Option<TelegramDocument>,
    pub video: Option<TelegramVideo>,
    pub animation: Option<TelegramAnimation>,
    pub audio: Option<TelegramAudio>,
    pub voice: Option<TelegramVoice>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramChat {
    pub id: i64,
    pub username: Option<String>,
    #[serde(rename = "type")]
    pub chat_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramPhotoSize {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: i32,
    pub height: i32,
    pub file_size: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramDocument {
    pub file_id: String,
    pub file_unique_id: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramVideo {
    pub file_id: String,
    pub file_unique_id: String,
    pub width: i32,
    pub height: i32,
    pub duration: i32,
    pub file_size: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramAnimation {
    pub file_id: String,
    pub file_unique_id: String,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramAudio {
    pub file_id: String,
    pub file_unique_id: String,
    pub duration: i32,
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TelegramVoice {
    pub file_id: String,
    pub file_unique_id: String,
    pub duration: i32,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TelegramFile {
    file_id: String,
    file_size: Option<i64>,
    file_path: String,
}

pub async fn handle_telegram_webhook(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // Get environment variables
    let telegram_token = match ctx.secret("TELEGRAM_BOT_TOKEN") {
        Ok(t) => t.to_string(),
        Err(_) => {
            console_log!("TELEGRAM_BOT_TOKEN not configured");
            return Response::ok("OK");
        }
    };

    let bale_token = match ctx.secret("BALE_TOKEN") {
        Ok(t) => t.to_string(),
        Err(_) => return Response::error("BALE_TOKEN not configured", 500),
    };

    // Initialize storage
    let db = ctx.env.d1("DB")?;
    let kv = ctx.env.kv("KV")?;

    let storage = D1Storage::new(db);
    let kv_storage = KVStorage::new(kv);

    // Initialize Bale API
    let bale_api = BaleApi::new(bale_token);

    // Parse update
    let update: TelegramUpdate = match req.json().await {
        Ok(u) => u,
        Err(e) => {
            console_log!("Failed to parse Telegram update: {}", e);
            return Response::error("Bad Request", 400);
        }
    };

    // Handle channel_post (messages in channels where bot is admin)
    if let Some(message) = update.channel_post {
        if let Err(e) = forward_message(message, &bale_api, &storage, &kv_storage).await {
            console_log!("Error forwarding message: {}", e);
        }
    }

    Response::ok("OK")
}

async fn forward_message(
    message: TelegramMessage,
    bale_api: &BaleApi,
    storage: &D1Storage,
    kv: &KVStorage,
) -> Result<()> {
    // Get enabled channels from database
    let channels = storage.get_enabled_channels().await?;

    // Find matching channel by chat ID or username
    let channel = channels.iter().find(|c| {
        c.telegram_id == message.chat.id
            || c.username.as_ref().map(|u| u.to_lowercase() == message.chat.username.as_ref().map(|u| u.to_lowercase()).unwrap_or_default()).unwrap_or(false)
    });

    let channel = match channel {
        Some(c) => c,
        None => {
            console_log!("No matching channel found for chat_id: {}", message.chat.id);
            return Ok(());
        }
    };

    // Check file size limits if media
    if let Some(max_size) = check_file_size(&message, channel.max_file_size_mb()) {
        if max_size > 0 {
            console_log!("File size {}MB exceeds limit {}MB", max_size, channel.max_file_size_mb());
            // Could send error message to admin
        }
    }

    // Forward to Bale
    let chat_id = channel.telegram_id; // Use Bale chat ID from channel config

    // Forward text message
    if let Some(text) = &message.text {
        let formatted = format!(
            "📢 *{}*\n\n{}",
            channel.name,
            escape_markdown(text)
        );
        bale_api.send_message(chat_id, &formatted, Some("Markdown")).await?;
    }

    // Forward caption with media
    if let Some(caption) = &message.caption {
        let formatted = format!(
            "📢 *{}*\n\n{}",
            channel.name,
            escape_markdown(caption)
        );
        // Media will be sent separately
    }

    // Handle media files
    // Photo
    if let Some(photos) = &message.photo {
        if let Some(largest) = photos.iter().max_by_key(|p| p.file_size.unwrap_or(0)) {
            bale_api.send_photo_by_file_id(chat_id, &largest.file_id, message.caption.as_deref(), Some("Markdown")).await?;
        }
    }

    // Document
    if let Some(doc) = &message.document {
        bale_api.send_document_by_file_id(chat_id, &doc.file_id, doc.file_name.as_deref(), message.caption.as_deref(), Some("Markdown")).await?;
    }

    // Video
    if let Some(video) = &message.video {
        bale_api.send_video_by_file_id(chat_id, &video.file_id, message.caption.as_deref(), Some("Markdown")).await?;
    }

    // Animation (GIF)
    if let Some(anim) = &message.animation {
        bale_api.send_animation_by_file_id(chat_id, &anim.file_id, message.caption.as_deref(), Some("Markdown")).await?;
    }

    // Audio
    if let Some(audio) = &message.audio {
        bale_api.send_audio_by_file_id(chat_id, &audio.file_id, message.caption.as_deref(), Some("Markdown")).await?;
    }

    // Voice
    if let Some(voice) = &message.voice {
        bale_api.send_voice_by_file_id(chat_id, &voice.file_id, message.caption.as_deref(), Some("Markdown")).await?;
    }

    Ok(())
}

fn check_file_size(message: &TelegramMessage, limit_mb: i32) -> Option<i64> {
    let mut max_size = 0i64;

    if let Some(photos) = &message.photo {
        for p in photos {
            if let Some(size) = p.file_size {
                max_size = max_size.max(size);
            }
        }
    }

    if let Some(doc) = &message.document {
        if let Some(size) = doc.file_size {
            max_size = max_size.max(size);
        }
    }

    if let Some(video) = &message.video {
        if let Some(size) = video.file_size {
            max_size = max_size.max(size);
        }
    }

    if max_size > 0 {
        Some(max_size / (1024 * 1024))
    } else {
        None
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