# Telegram-Bale Bot Migration - Setup Guide

## 🚀 Quick Start

### Prerequisites
- Rust installed: https://rustup.rs/
- Wrangler CLI: `npm install -g wrangler`
- Cloudflare account (free tier works)

### Step 1: Clone and Setup

```bash
cd cloudflare-bot
cp .env.example .env
# Edit .env with your credentials
```

### Step 2: Create Cloudflare Resources

```bash
# Login to Cloudflare
wrangler login

# Create D1 Database
wrangler d1 create telegram_bale_db
# Copy the database_id and update wrangler.toml

# Create KV Namespace
wrangler kv:namespace create "bot_kv"
# Copy the id and update wrangler.toml

# Apply database migrations
wrangler d1 execute telegram_bale_db --file=./migrations/0001_initial_schema.sql
```

### Step 3: Configure Secrets

```bash
wrangler secret put BALE_TOKEN
# Enter your Bale bot token

wrangler secret put TELEGRAM_API_ID
# Enter your API ID from https://my.telegram.org

wrangler secret put TELEGRAM_API_HASH
# Enter your API Hash

wrangler secret put SESSION_BOT_TOKEN
# Enter your Telegram session bot token from @BotFather
```

### Step 4: Build and Deploy

```bash
# Build all crates
cargo build --release

# Deploy to Cloudflare
wrangler deploy
```

### Step 5: Setup Webhooks

After deployment, you'll get a URL like: `https://telegram-bale-bot.your-subdomain.workers.dev`

**Set Bale Webhook:**
```bash
curl -X POST "https://tapi.bale.ai/bot<BALE_TOKEN>/setWebhook" \
  -d "url=https://your-worker-url.workers.dev/bale/webhook"
```

**Set Telegram Session Bot Webhook:**
```bash
curl -X POST "https://api.telegram.org/bot<SESSION_BOT_TOKEN>/setWebhook" \
  -d "url=https://your-worker-url.workers.dev/session-bot/webhook"
```

## 📝 Implementation Status

### ✅ Completed (Phase 1 - Infrastructure)
- [x] Project structure
- [x] Workspace setup
- [x] D1 schema and migrations
- [x] Shared library (models, storage, utils, error handling)
- [x] Bale API wrapper (complete with all methods)
- [x] Main Bot handlers (all commands implemented)
- [x] Documentation (README, MIGRATION_PLAN)
- [x] Configuration files (wrangler.toml, .env.example)

### 🚧 In Progress (Phase 2 - Core Features)
- [ ] Session Management Bot (Telegram auth flow)
- [ ] Telegram Poller (Durable Object with long-polling)
- [ ] Message forwarding logic
- [ ] File streaming implementation

### ⏳ Pending (Phase 3 - Advanced)
- [ ] Compression and splitting
- [ ] Pending posts system
- [ ] Bale → Telegram sending
- [ ] Advanced media types
- [ ] Testing and optimization

## 🎯 What's Working Now

The **Main Bot** is fully functional for:
- `/start` - Welcome message
- `/help` - Command help
- `/add_channel @username [+size|+zipSize]` - Add channels to monitor
- `/remove_channel @username` - Remove channels
- `/list_channels` - List all channels with status
- `/start_monitor` - Enable monitoring
- `/stop_monitor` - Disable monitoring
- `/status` - Show bot status

## 🔨 Next Steps to Complete MVP

1. **Implement Session Bot** (4-5 hours)
   - Telegram authentication flow with grammers
   - Session storage in D1
   - Phone/code/password handlers

2. **Implement Telegram Poller** (6-8 hours)
   - Durable Object setup
   - Long-polling with grammers
   - Message processing
   - Forward to Bale with streaming

3. **Setup Cron Trigger** (1-2 hours)
   - Configure cron in wrangler.toml
   - Wake Durable Object every minute

4. **Testing** (3-4 hours)
   - Local testing with wrangler dev
   - End-to-end flow testing
   - Deploy and monitor

## 📊 Current Files

```
cloudflare-bot/
├── Cargo.toml                 ✅ Workspace config
├── README.md                  ✅ Documentation
├── MIGRATION_PLAN.md          ✅ Full migration plan
├── SETUP.md                   ✅ This file
├── wrangler.toml              ✅ Cloudflare config
├── .env.example               ✅ Environment template
├── .gitignore                 ✅ Git ignore
│
├── crates/
│   ├── shared/                ✅ Complete
│   │   ├── src/
│   │   │   ├── lib.rs         ✅
│   │   │   ├── models.rs      ✅ All data models
│   │   │   ├── error.rs       ✅ Error handling
│   │   │   ├── utils.rs       ✅ Helper functions
│   │   │   └── storage/
│   │   │       ├── d1.rs      ✅ D1 operations
│   │   │       └── kv.rs      ✅ KV operations
│   │   └── Cargo.toml         ✅
│   │
│   ├── main-bot/              ✅ Complete
│   │   ├── src/
│   │   │   ├── lib.rs         ✅ Entry point + routing
│   │   │   ├── bale_api.rs    ✅ Bale API wrapper
│   │   │   └── handlers.rs    ✅ Command handlers
│   │   └── Cargo.toml         ✅
│   │
│   ├── session-bot/           🚧 Placeholder
│   │   ├── src/
│   │   │   ├── lib.rs         🚧 TODO
│   │   │   ├── telegram_auth.rs  🚧 TODO
│   │   │   └── handlers.rs    🚧 TODO
│   │   └── Cargo.toml         ✅
│   │
│   └── telegram-poller/       🚧 Placeholder
│       ├── src/
│       │   ├── lib.rs         🚧 TODO
│       │   ├── poller.rs      🚧 TODO
│       │   └── message_handler.rs  🚧 TODO
│       └── Cargo.toml         ✅
│
├── migrations/
│   └── 0001_initial_schema.sql  ✅ Initial DB schema
│
└── sql/
    └── schema.sql              ✅ Full schema reference
```

## 💡 Usage Examples

Once deployed, use these commands in Bale:

```
/start
# Shows welcome message

/add_channel @rahgozar94725_ip +200
# Adds channel with 200MB custom limit

/add_channel @iXV2ray +zip50
# Adds channel with zip compression

/list_channels
# Shows all added channels

/start_monitor
# Starts forwarding messages

/status
# Shows current status
```

## 🐛 Troubleshooting

### Build Errors
```bash
cargo clean
cargo build
```

### Wrangler Issues
```bash
wrangler logout
wrangler login
```

### Database Issues
```bash
# Check D1
wrangler d1 execute telegram_bale_db --command="SELECT * FROM channels"

# Reset database
wrangler d1 execute telegram_bale_db --file=./migrations/0001_initial_schema.sql
```

## 📚 Resources

- [Cloudflare Workers Docs](https://developers.cloudflare.com/workers/)
- [Wrangler CLI Docs](https://developers.cloudflare.com/workers/wrangler/)
- [D1 Database Docs](https://developers.cloudflare.com/d1/)
- [Workers KV Docs](https://developers.cloudflare.com/kv/)
- [Durable Objects Docs](https://developers.cloudflare.com/durable-objects/)

## 🤝 Contributing

This is a personal migration project. See MIGRATION_PLAN.md for the full roadmap.

---

**Built with ❤️ using Rust and Cloudflare Workers**
