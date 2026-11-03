# 🚀 PLAN: Migration از Python Bot به Rust + Cloudflare Workers

## 📋 خلاصه وضعیت فعلی

**Bot فعلی (Python):**
- **زبان**: Python (~5,300 خط کد)
- **کتابخانه‌ها**: 
  - `telethon` برای Telegram (MTProto API)
  - `balethon` برای Bale (REST API)
- **معماری**: Long-polling همزمان برای هر دو پلتفرم
- **قابلیت‌های اصلی**:
  - مانیتورینگ 20+ کانال Telegram و forward به Bale
  - مدیریت فایل‌های بزرگ (تا 200MB) با compression و splitting
  - سیستم pending posts با download buttons
  - ارسال دوطرفه Bale ↔ Telegram
  - دستورات مدیریتی از طریق Bale bot

---

## 🎯 اهداف Migration

### ✅ انتخاب‌های معماری (بر اساس پاسخ‌های شما)
- **پلتفرم**: Cloudflare Workers (Free Plan)
- **معماری**: Durable Objects + Workers
- **Storage**: Hybrid (D1 + Workers KV)
- **File Handling**: Streaming (بدون ذخیره‌سازی)
- **Scope**: MVP اول، بعد قابلیت‌های پیشرفته

### 🔒 محدودیت‌های Free Plan
- CPU time: 10ms per request
- Memory: 128MB
- Durable Objects: محدود ولی کافی برای long-polling
- D1: 5GB storage, 100k reads/day
- KV: 100k reads/day, 1k writes/day

---

## 📚 جواب سوال: کتابخانه‌های Telegram و Bale

### ❌ نمیشه از یک کتابخانه برای هر دو استفاده کرد چون:

**Telegram:**
- دو روش ارتباط: REST API (Bot API) یا MTProto (User API)
- برای login به عنوان user نیاز به MTProto داریم
- **کتابخانه‌های Rust**:
  - `grammers` → کامل‌ترین، MTProto native
  - `teloxide` → فقط Bot API (برای bot account)
  - `tbot` → ساده‌تر ولی محدودتر

**Bale:**
- فقط REST API (شبیه Telegram Bot API)
- کتابخانه Rust رسمی نداره
- **راه‌حل**: خودمان HTTP client می‌نویسیم با `reqwest`

### ✅ توصیه نهایی:
```rust
// برای Telegram (user login + monitoring)
grammers = "0.7"  // MTProto برای session و channel access

// برای Bale (bot commands)
reqwest = "0.11"  // ساده HTTP client
serde = "1.0"     // JSON serialization
```

---

## 🏗️ معماری جدید

### 📦 ساختار پروژه

```
cloudflare-bot/
├── Cargo.toml                    # Root workspace
├── README.md                     # مستندات کامل
├── MIGRATION_PLAN.md             # این فایل
│
├── crates/
│   ├── session-bot/              # 🔐 Bot مدیریت session
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── telegram_auth.rs  # Login flow
│   │   │   └── handlers.rs       # Command handlers
│   │   └── Cargo.toml
│   │
│   ├── main-bot/                 # 🤖 Bot اصلی (Bale commands)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── bale_api.rs       # Bale API wrapper
│   │   │   ├── handlers.rs       # Command routing
│   │   │   └── models.rs         # Data structures
│   │   └── Cargo.toml
│   │
│   ├── telegram-poller/          # 📡 Telegram long-polling (Durable Object)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── poller.rs         # Long-polling loop
│   │   │   ├── message_handler.rs
│   │   │   └── session_manager.rs
│   │   └── Cargo.toml
│   │
│   └── shared/                   # 📚 کتابخانه مشترک
│       ├── src/
│       │   ├── lib.rs
│       │   ├── storage/
│       │   │   ├── d1.rs         # D1 queries
│       │   │   └── kv.rs         # KV operations
│       │   ├── models.rs         # Shared types
│       │   └── utils.rs          # Helper functions
│       └── Cargo.toml
│
├── migrations/                   # D1 migrations
│   └── 0001_initial_schema.sql
│
├── sql/                          # SQL schemas
│   └── schema.sql
│
├── wrangler.toml                 # Cloudflare config
└── .env.example                  # Environment template
```

---

## 🔄 Components و مسئولیت‌ها

### 1️⃣ **Session Management Bot** (Durable Object + Worker)

**مسئولیت:**
- Login به Telegram با phone number
- ذخیره session در D1
- فقط به admin ID شما پاسخ میده
- دستورات: `/start`, `/login`, `/code <CODE>`, `/password <PASS>`, `/logout`

**Flow:**
```
User → Telegram Bot → Worker (webhook) → Durable Object
                                       ↓
                              D1 (save session)
```

**Schema D1 (sessions table):**
```sql
CREATE TABLE telegram_sessions (
  id INTEGER PRIMARY KEY,
  user_id INTEGER UNIQUE NOT NULL,
  session_data BLOB NOT NULL,     -- grammers session binary
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
```

### 2️⃣ **Main Bot** (Worker با Webhook - Bale)

**مسئولیت:**
- دریافت دستورات از Bale (webhook)
- CRUD برای channels: `/add_channel`, `/remove_channel`, `/list_channels`
- کنترل monitoring: `/start_monitor`, `/stop_monitor`
- دریافت status: `/status`

**Endpoints:**
- `POST /bale/webhook` → دریافت updates از Bale
- `GET /health` → health check

**Schema D1 (channels table):**
```sql
CREATE TABLE channels (
  id INTEGER PRIMARY KEY,
  telegram_id INTEGER UNIQUE NOT NULL,
  name TEXT NOT NULL,
  username TEXT,
  link TEXT,
  custom_max_size_mb INTEGER,
  zip_max_size_mb INTEGER,
  enabled BOOLEAN DEFAULT 1,
  added_at INTEGER NOT NULL
);
```

### 3️⃣ **Telegram Poller** (Durable Object)

**مسئولیت:**
- Long-polling از Telegram (getUpdates)
- پردازش پیام‌های کانال‌ها
- Forward به Bale با streaming

**چالش Free Plan:**
- نمیتونیم مدام polling کنیم (10ms CPU limit)
- **راه‌حل**: Cron Trigger هر 1 دقیقه wake up میکنه Durable Object رو

**Flow:**
```
Cron (every 1min) → Wake Durable Object
                         ↓
                   Telegram.getUpdates(timeout=50s)
                         ↓
                   Process new messages
                         ↓
                   Forward to Bale (streaming)
```

### 4️⃣ **Storage Strategy**

#### D1 (SQL - Persistent Data):
- `telegram_sessions`: Session data
- `channels`: لیست کانال‌های مانیتور شده
- `pending_posts`: پست‌های pending (Phase 2)

#### Workers KV (Cache - Fast Access):
- `throttle:{user_id}`: Rate limiting
- `error_count:{channel_id}`: Error throttling
- `last_update_id`: آخرین update_id از Telegram

---

## 📝 MVP Scope (Phase 1)

### ✅ قابلیت‌هایی که پیاده می‌کنیم:

#### Session Bot:
- [ ] Login flow با phone/code/password
- [ ] ذخیره session در D1
- [ ] Admin-only access با hardcoded USER_ID

#### Main Bot (Bale):
- [ ] Webhook setup برای Bale
- [ ] دستور `/add_channel <username|id> [+size]`
- [ ] دستور `/remove_channel <username|id>`
- [ ] دستور `/list_channels` (بدون pagination - ساده)
- [ ] دستور `/start_monitor` / `/stop_monitor`
- [ ] دستور `/status` (چند کانال، monitoring on/off)

#### Telegram Poller:
- [ ] Long-polling با Cron trigger (1min interval)
- [ ] پردازش text messages
- [ ] پردازش photos (تا 20MB)
- [ ] پردازش documents (تا 20MB)
- [ ] Forward به Bale با streaming (بدون ذخیره روی disk)
- [ ] Mark as read

#### Storage:
- [ ] D1 setup: sessions, channels tables
- [ ] KV setup: throttling, cache
- [ ] Migration script: import از channels.json فعلی

---

## ❌ قابلیت‌هایی که Phase 2 هستند:

- Compression و splitting فایل‌های بزرگ
- Pending posts system با download buttons
- Bale → Telegram sending
- Media پیشرفته: albums, stickers, animations
- Inline keyboard pagination برای `/list_channels`
- Error throttling پیشرفته
- Dashboard web UI

---

## 🚧 چالش‌ها و راه‌حل‌ها

### 1️⃣ **Free Plan CPU Limit (10ms)**
**مشکل**: نمیتونیم مدام long-polling کنیم  
**راه‌حل**: 
- Cron Trigger هر 1 دقیقه Durable Object رو wake میکنه
- هر wake: 50-second blocking call به `getUpdates`
- Trade-off: تاخیر تا 1 دقیقه برای forward

### 2️⃣ **Session Management**
**مشکل**: Telegram session فایل باینری بزرگیه  
**راه‌حل**: 
- grammers session رو serialize کنیم به BLOB
- ذخیره در D1 (تا 1MB per row کافیه)
- هر بار load → deserialize → use

### 3️⃣ **File Streaming**
**مشکل**: نمیتونیم فایل رو download و ذخیره کنیم (محدودیت memory)  
**راه‌حل**:
```rust
// Stream from Telegram → directly to Bale
let telegram_stream = telegram.download_media_stream(message);
let response = bale.upload_photo_stream(telegram_stream).await?;
```
- محدودیت: فایل‌های بزرگ (>20MB) در MVP نداریم
