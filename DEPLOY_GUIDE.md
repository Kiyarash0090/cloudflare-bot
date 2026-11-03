# 🚀 راهنمای کامل Deploy در Cloudflare Workers

این راهنمای گام به گام برای deploy کردن Telegram-Bale Bot در Cloudflare Workers است.

---

## پیش‌نیازها

قبل از شروع، مطمئن شوید این‌ها رو دارید:

- [ ] حساب Cloudflare (رایگان کافیه): https://dash.cloudflare.com/sign-up
- [ ] Node.js نصب باشه (برای Wrangler CLI)
- [ ] Rust نصب باشه
- [ ] Git Bash یا Terminal
- [ ] Token ربات Bale
- [ ] API ID و API Hash از Telegram

---

## مرحله 1: نصب ابزارها

### 1.1. چک کردن Node.js

```bash
node --version
# باید نسخه 16 یا بالاتر باشه
```

اگر نداری، از اینجا دانلود کن: https://nodejs.org/

### 1.2. نصب Wrangler CLI

```bash
npm install -g wrangler

# چک کن که نصب شد
wrangler --version
```

### 1.3. چک کردن Rust

```bash
rustc --version
cargo --version
```

اگر نداری:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## مرحله 2: Login به Cloudflare

### 2.1. Login کردن

```bash
wrangler login
```

این دستور یک مرورگر باز می‌کنه و باید authorize کنی.

### 2.2. چک کردن Account

```bash
wrangler whoami
```

باید account و Account ID رو نشون بده.

---

## مرحله 3: ساخت Resources در Cloudflare

### 3.1. ساخت D1 Database

```bash
cd C:\Users\kiarash\Desktop\cod\v1.1.4\cloudflare-bot

wrangler d1 create telegram_bale_db
```

**خروجی شبیه این میشه:**
```
✅ Successfully created DB 'telegram_bale_db'

[[d1_databases]]
binding = "DB"
database_name = "telegram_bale_db"
database_id = "xxxx-xxxx-xxxx-xxxx-xxxx"
```

**مهم:** `database_id` رو کپی کن!

### 3.2. Update کردن wrangler.toml

فایل `wrangler.toml` رو باز کن و `database_id` رو جایگزین کن:

```toml
[[d1_databases]]
binding = "DB"
database_name = "telegram_bale_db"
database_id = "xxxx-xxxx-xxxx-xxxx-xxxx"  # اینجا رو عوض کن
```

### 3.3. اجرای Migration

```bash
wrangler d1 execute telegram_bale_db --file=./migrations/0001_initial_schema.sql
```

باید بگه: `✅ Successfully executed SQL`

**چک کردن جداول:**
```bash
wrangler d1 execute telegram_bale_db --command="SELECT name FROM sqlite_master WHERE type='table'"
```

باید 4 جدول رو نشون بده: `telegram_sessions`, `channels`, `monitoring_state`, `pending_posts`

### 3.4. ساخت KV Namespace

```bash
wrangler kv:namespace create "bot_kv"
```

**خروجی:**
```
✅ Success!
Add the following to your wrangler.toml:

[[kv_namespaces]]
binding = "KV"
id = "xxxxxxxxxxxxxxxxxxxxxxxxxxxx"
```

**مهم:** `id` رو کپی کن!

### 3.5. Update کردن wrangler.toml

```toml
[[kv_namespaces]]
binding = "KV"
id = "xxxxxxxxxxxxxxxxxxxxxxxxxxxx"  # اینجا رو عوض کن
```

---

## مرحله 4: تنظیم Secrets

حالا باید secret های حساس رو set کنیم.

### 4.1. Bale Token

```bash
wrangler secret put BALE_TOKEN
```

وقتی پرسید، token ربات Bale رو وارد کن و Enter بزن.

### 4.2. Telegram API ID

```bash
wrangler secret put TELEGRAM_API_ID
```

API ID رو از https://my.telegram.org بگیر و وارد کن.

### 4.3. Telegram API Hash

```bash
wrangler secret put TELEGRAM_API_HASH
```

API Hash رو وارد کن.

### 4.4. Session Bot Token (بعداً)

این رو فعلاً نیاز نیست، بعداً که Session Bot رو پیاده کردیم set می‌کنیم.

```bash
# فعلاً skip کن
# wrangler secret put SESSION_BOT_TOKEN
```

### 4.5. چک کردن Secrets

```bash
wrangler secret list
```

باید این‌ها رو نشون بده:
- BALE_TOKEN
- TELEGRAM_API_ID
- TELEGRAM_API_HASH

---

## مرحله 5: Build و Deploy

### 5.1. Build کردن پروژه

```bash
cd C:\Users\kiarash\Desktop\cod\v1.1.4\cloudflare-bot

cargo build --release --target wasm32-unknown-unknown
```

**نکته:** اگر error داد که wasm32 target نداری:
```bash
rustup target add wasm32-unknown-unknown
```

### 5.2. Deploy کردن

```bash
wrangler deploy
```

**خروجی موفق:**
```
✨ Built successfully
✨ Uploading...
✨ Deployed telegram-bale-bot
   https://telegram-bale-bot.your-subdomain.workers.dev
```

**مهم:** URL رو کپی کن! این URL worker شماست.

### 5.3. تست کردن Deploy

```bash
curl https://your-worker-url.workers.dev/health
```

باید `OK` برگردونه.

---

## مرحله 6: Setup Webhook برای Bale

حالا باید به Bale بگیم پیام‌ها رو به worker ما بفرسته.

### 6.1. Set کردن Webhook

**فرمت دستور:**
```bash
curl -X POST "https://tapi.bale.ai/bot<BALE_TOKEN>/setWebhook" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://your-worker-url.workers.dev/bale/webhook"}'
```

**مثال واقعی:**
```bash
# Token خودت رو جایگزین کن
curl -X POST "https://tapi.bale.ai/bot569330731:HLEcgoVy-yTIQHxwSolEGYuYoWSfmeoHBz8/setWebhook" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://telegram-bale-bot.kiarash.workers.dev/bale/webhook"}'
```

**خروجی موفق:**
```json
{"ok":true,"result":true,"description":"Webhook was set"}
```

### 6.2. چک کردن Webhook

```bash
curl "https://tapi.bale.ai/bot<BALE_TOKEN>/getWebhookInfo"
```

باید URL worker شما رو نشون بده.

---

## مرحله 7: تست کردن Bot

### 7.1. باز کردن Bale

1. تو Bale برو به ربات خودت
2. پیام `/start` رو بفرست

**باید جواب بده:**
```
🤖 خوش آمدید!

این ربات کانال‌های تلگرام را مانیتور کرده و پست‌ها را به بله ارسال می‌کند.

دستورات موجود:
/add_channel - افزودن کانال
/remove_channel - حذف کانال
...
```

### 7.2. تست دستورات

```
/help
/status
/list_channels
```

همه باید کار کنن!

### 7.3. افزودن کانال تستی

```
/add_channel @test_channel +100
```

باید بگه کانال اضافه شد.

---

## مرحله 8: Monitoring و Logs

### 8.1. دیدن Logs زنده

```bash
wrangler tail
```

این تمام logs رو real-time نشون میده.

### 8.2. دیدن Logs در Dashboard

1. برو به: https://dash.cloudflare.com
2. Workers & Pages → telegram-bale-bot
3. Logs

### 8.3. چک کردن Metrics

در همون صفحه، tab "Metrics" رو ببین:
- Request count
- Errors
- CPU time
- Success rate

---

## مرحله 9: Import کردن Channels فعلی (اختیاری)

اگر می‌خوای کانال‌های موجود توی Python bot رو import کنی:

### 9.1. ساخت SQL از JSON

```bash
cd C:\Users\kiarash\Desktop\cod\v1.1.4

# این script رو بنویس (یا من می‌نویسم)
node scripts/import_channels.js
```

یا می‌تونیم یک script Python بنویسیم که channels.json رو به SQL تبدیل کنه.

می‌خوای این script رو بنویسم؟

---

## مرحله 10: Troubleshooting

### ❌ Error: "No access to D1"

**راه‌حل:**
```bash
# دوباره binding رو check کن
wrangler d1 list

# اگر database نیست، دوباره بساز
wrangler d1 create telegram_bale_db
```

### ❌ Error: "KV namespace not found"

**راه‌حل:**
```bash
wrangler kv:namespace list
# ID رو copy کن و توی wrangler.toml بزار
```

### ❌ Error: "Unauthorized webhook"

**راه‌حل:**
```bash
# Secret رو دوباره set کن
wrangler secret put BALE_TOKEN

# Worker رو redeploy کن
wrangler deploy
```

### ❌ Error: "Build failed"

**راه‌حل:**
```bash
# Clean کن و دوباره build کن
cargo clean
cargo build --release --target wasm32-unknown-unknown
```

### ❌ Bot جواب نمیده

**چک کن:**
1. Webhook درست set شده؟
```bash
curl "https://tapi.bale.ai/bot<TOKEN>/getWebhookInfo"
```

2. Logs چی میگه؟
```bash
wrangler tail
```

3. Worker بالاست؟
```bash
curl https://your-worker.workers.dev/health
```

---

## مرحله 11: Update کردن Worker

وقتی تغییری دادی:

```bash
# Build
cargo build --release --target wasm32-unknown-unknown

# Deploy
wrangler deploy
```

---

## 📊 چک لیست نهایی

قبل از اینکه بگی تموم شد، این‌ها رو چک کن:

- [ ] `wrangler whoami` account رو نشون میده
- [ ] `wrangler d1 list` دیتابیس رو نشون میده
- [ ] `wrangler kv:namespace list` KV رو نشون میده
- [ ] `wrangler secret list` secrets رو نشون میده
- [ ] Worker URL کار می‌کنه: `curl https://your-url/health`
- [ ] Webhook set شده: `getWebhookInfo`
- [ ] Bot توی Bale به `/start` جواب میده
- [ ] `/status` وضعیت رو نشون میده
- [ ] `/list_channels` کار می‌کنه (حتی اگر خالی باشه)

---

## 🎯 وضعیت بعد از Deploy

### ✅ چیزهایی که کار می‌کنن:
- Main Bot با تمام دستورات
- Database (D1) برای ذخیره channels
- KV برای rate limiting
- Webhook از Bale

### ❌ چیزهایی که هنوز کار نمی‌کنن:
- Login به Telegram (نیاز به Session Bot)
- Forward واقعی پیام‌ها (نیاز به Telegram Poller)
- Cron trigger (فعلاً disable است)

---

## 💰 هزینه (Free Plan)

با Free Plan میتونی:
- 100,000 request/day
- 10ms CPU time per request
- 5GB D1 storage
- 100k KV reads/day

برای این bot، رایگان کافیه! 🎉

---

## 📞 کمک بیشتر

اگر به مشکل خوردی:

1. Logs رو چک کن: `wrangler tail`
2. Error رو کپی کن و بپرس
3. Cloudflare Docs: https://developers.cloudflare.com/workers/

---

## 🚀 مرحله بعدی

بعد از Deploy موفق، می‌تونیم:
1. Session Bot رو پیاده کنیم (برای login)
2. Telegram Poller رو بسازیم (برای forward)
3. Cron trigger رو فعال کنیم

---

**آماده‌ای شروع کنی؟** 🎯
