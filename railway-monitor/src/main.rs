use grammers_client::{Client, Config, InitParams, types::LoginToken};
use grammers_session::Session;
use grammers_tl_types as tl;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

mod config;
mod bale;
mod monitor;

use config::Config as AppConfig;
use monitor::ChannelMonitor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Load configuration
    let config = AppConfig::load()?;
    info!("Starting Railway Monitor...");

    // Connect to Telegram
    let client = connect_telegram(&config).await?;
    info!("Connected to Telegram");

    // Get channel list from config
    let channels = config.monitored_channels.clone();
    let sync_channel = config.sync_channel.clone();
    let bale_token = config.bale_token.clone();
    let bale_chat_id = config.bale_chat_id;

    // Start monitoring
    let monitor = ChannelMonitor::new(
        client,
        channels,
        sync_channel,
        bale_token,
        bale_chat_id,
    );

    monitor.run().await?;

    Ok(())
}

async fn connect_telegram(config: &AppConfig) -> anyhow::Result<Client> {
    let session_file = format!("{}_session", config.telegram_session_name);

    let client = Client::connect(Config {
        session: Session::load_file_or_create(&session_file)?,
        api_id: config.telegram_api_id,
        api_hash: config.telegram_api_hash.clone(),
        params: InitParams {
            catch_up: true,
            ..Default::default()
        },
    }).await?;

    // Check if authorized, if not login
    if !client.is_authorized().await? {
        info!("Not authorized, logging in...");

        let token = client.request_login_code(
            config.telegram_phone.clone(),
            tl::enums::CodeSettings::default(),
        ).await?;

        // In production, you'd get this from user input or secure storage
        // For Railway, we'll use the login token from env
        let login_token = env::var("TELEGRAM_LOGIN_TOKEN")
            .ok()
            .and_then(|s| s.parse().ok());

        if let Some(token) = login_token {
            client.accept_login_token(token).await?;
        } else {
            // If no token, we can't proceed without user input
            // This would need manual intervention
            return Err(anyhow::anyhow!("Login token required for first run"));
        }
    }

    Ok(client)
}