use serde::{Deserialize, Serialize};

/// Telegram session stored in D1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramSession {
    pub id: i64,
    pub user_id: i64,
    pub session_data: Vec<u8>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: Option<i64>,
    pub telegram_id: i64,
    pub name: String,
    pub username: Option<String>,
    pub link: Option<String>,
    pub custom_max_size_mb: Option<i32>,
    pub zip_max_size_mb: Option<i32>,
    pub enabled: bool,
    pub added_at: i64,
    pub updated_at: i64,
}

impl Channel {
    pub fn new(telegram_id: i64, name: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: None,
            telegram_id,
            name,
            username: None,
            link: None,
            custom_max_size_mb: None,
            zip_max_size_mb: None,
            enabled: true,
            added_at: now,
            updated_at: now,
        }
    }

    pub fn max_file_size_mb(&self) -> i32 {
        self.custom_max_size_mb
            .or(self.zip_max_size_mb)
            .unwrap_or(20)
    }
}

/// Monitoring state (singleton)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringState {
    pub id: i64,
    pub is_running: bool,
    pub last_update_id: i64,
    pub started_at: Option<i64>,
    pub stopped_at: Option<i64>,
}

impl MonitoringState {
    pub fn new() -> Self {
        Self {
            id: 1,
            is_running: false,
            last_update_id: 0,
            started_at: None,
            stopped_at: None,
        }
    }
}

impl Default for MonitoringState {
    fn default() -> Self {
        Self::new()
    }
}

/// Bale API message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaleMessage {
    pub message_id: i64,
    pub from: Option<BaleUser>,
    pub chat: BaleChat,
    pub text: Option<String>,
    pub caption: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaleUser {
    pub id: i64,
    pub first_name: String,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaleChat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
}

/// Bale API update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaleUpdate {
    pub update_id: i64,
    pub message: Option<BaleMessage>,
}

/// Bale API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaleApiResponse<T> {
    pub ok: bool,
    pub result: Option<T>,
    pub description: Option<String>,
}

/// Telegram message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    Photo,
    Document,
    Video,
    Audio,
    Voice,
    Sticker,
    Animation,
}

/// Channel input parsing result
#[derive(Debug, Clone)]
pub struct ChannelInput {
    pub identifier: String,
    pub custom_size: Option<i32>,
    pub zip_size: Option<i32>,
}

impl ChannelInput {
    pub fn parse(input: &str) -> crate::Result<Self> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Err(crate::Error::InvalidInput("Empty input".to_string()));
        }

        let identifier = parts[0].trim_start_matches('@').to_string();
        let mut custom_size = None;
        let mut zip_size = None;

        for part in &parts[1..] {
            if let Some(size_str) = part.strip_prefix("+zip") {
                zip_size = Some(size_str.parse().map_err(|_| {
                    crate::Error::InvalidInput("Invalid zip size".to_string())
                })?);
            } else if let Some(size_str) = part.strip_prefix('+') {
                custom_size = Some(size_str.parse().map_err(|_| {
                    crate::Error::InvalidInput("Invalid custom size".to_string())
                })?);
            }
        }

        Ok(Self {
            identifier,
            custom_size,
            zip_size,
        })
    }
}
