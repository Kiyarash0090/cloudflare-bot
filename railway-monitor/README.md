# Railway Monitor - Telegram Userbot for Channel Monitoring

This is a Rust-based Telegram userbot that runs on Railway. It:
1. Logs into Telegram using a user account (via session string)
2. Monitors specified channels (both public and private)
3. Forwards messages to a sync channel in Telegram
4. Forwards messages to Bale messenger

## Prerequisites

- Railway account
- Telegram API credentials (api_id, api_hash)
- Telegram phone number
- Bale bot token

## Setup

### 1. Get Telegram API Credentials
Go to https://my.telegram.org and create an application to get `api_id` and `api_hash`.

### 2. Create Session String (First Run Only)

Run locally first to generate session:

```bash
# Clone the repo
git clone <your-repo>
cd railway-monitor

# Create config.toml from example
cp config.example.toml config.toml
# Edit config.toml with your credentials

# Run locally to generate session
cargo run
```

On first run, it will ask for login code. After successful login, a session file will be created.

### 3. Deploy to Railway

```bash
# Install Railway CLI
npm install -g @railway/cli

# Login
railway login

# Initialize project
railway init

# Add environment variables
railway variables set APP__TELEGRAM_API_ID=26820590
railway variables set APP__TELEGRAM_API_HASH=76c39222f20cc6138bad5f0ce9760a2c
railway variables set APP__TELEGRAM_PHONE="+98XXXXXXXXXX"
railway variables set APP__TELEGRAM_SESSION_NAME="railway_monitor"
railway variables set APP__SYNC_CHANNEL="your_sync_channel_username"
railway variables set APP__BALE_TOKEN="748785105:7aAorNTI9sqFKNGXfaL_1lWqaJTKNzP7n6E"
railway variables set APP__BALE_CHAT_ID=1575910263

# For monitored channels, you can either:
# 1. Add them in config.toml and deploy
# 2. Or set via environment variables (complex for arrays)

# Deploy
railway up
```

### 4. Add Monitored Channels

Edit `config.toml` before deploying, or use Railway's dashboard to add environment variables.

## Configuration

All config via `config.toml` or environment variables with `APP__` prefix:

```toml
telegram_api_id = 26820590
telegram_api_hash = "your_hash"
telegram_phone = "+98XXXXXXXXXX"
telegram_session_name = "railway_monitor"

[[monitored_channels]]
id = -1001234567890
name = "Channel Name"
username = "channel_username"
forward_to_sync = true

sync_channel = "sync_channel_username"
bale_token = "your_bale_bot_token"
bale_chat_id = 123456789
```

## How It Works

1. **Telegram Userbot**: Uses `grammers` (MTProto) to connect as a user account
2. **Channel Monitoring**: Listens for `Update::NewMessage` from monitored channels
3. **Deduplication**: Tracks processed message IDs in memory (last 10,000)
4. **Forwarding**: 
   - To Bale: Uses Bot API with file_ids
   - To Sync Channel: Uses `forward_messages` + header

## No Conflict with Cloudflare Worker

- Cloudflare Worker: Handles Bale commands + Telegram Bot API webhook (public channels only)
- Railway Monitor: Handles MTProto userbot for private channels + sync channel forwarding

They use different Telegram tokens:
- Cloudflare: Bot token `9297696333:AAFlkRVZOl3Y5JSg18evxnr_jjjYo42B7Uk`
- Railway: User account (phone + session)

## Monitoring

Check logs in Railway dashboard:
```bash
railway logs
```

## Health Check

The service doesn't expose HTTP by default. For health checks, you could add a simple HTTP server in the future.