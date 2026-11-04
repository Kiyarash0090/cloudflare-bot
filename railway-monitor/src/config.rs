use config::{Config, ConfigError, File};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub telegram_api_id: i32,
    pub telegram_api_hash: String,
    pub telegram_phone: String,
    pub telegram_session_name: String,
    pub monitored_channels: Vec<MonitoredChannel>,
    pub sync_channel: String,
    pub bale_token: String,
    pub bale_chat_id: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MonitoredChannel {
    pub id: i64,
    pub name: String,
    pub username: Option<String>,
    pub forward_to_sync: bool,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let mut builder = Config::builder()
            .add_source(File::with_name("config").required(false))
            .add_source(config::Environment::with_prefix("APP").separator("__"));

        // Add defaults
        builder = builder
            .set_default("telegram_session_name", "railway_monitor")?
            .set_default("sync_channel", "")?;

        builder.build()?.try_deserialize()
    }
}