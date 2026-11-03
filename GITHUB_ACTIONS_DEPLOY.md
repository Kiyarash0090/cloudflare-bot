# 🚀 GitHub Actions - Auto Deploy به Cloudflare Workers

این راهنما برای deploy اتوماتیک پروژه به Cloudflare Workers بدون نیاز به نصب Rust روی سیستم شخصی است.

---

## مزایا

- ✅ نیازی به نصب Rust روی سیستمت نیست
- ✅ هر بار که push کنی، اتوماتیک deploy میشه
- ✅ Build و Deploy توی cloud انجام میشه
- ✅ تمیز و حرفه‌ای

---

## مرحله 1: آماده‌سازی پروژه

### 1.1. ساخت `.github/workflows` directory

```bash
cd C:\Users\kiarash\Desktop\cod\v1.1.4\cloudflare-bot
mkdir -p .github/workflows
```

### 1.2. ساخت فایل workflow

فایل `.github/workflows/deploy.yml` رو با این محتوا بساز:

```yaml
name: Deploy to Cloudflare Workers

on:
  push:
    branches:
      - main
  workflow_dispatch:  # برای deploy دستی

jobs:
  deploy:
    runs-on: ubuntu-latest
    name: Deploy

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-unknown-unknown
          override: true

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache target directory
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-target-${{ hashFiles('**/Cargo.lock') }}

      - name: Build
        run: |
          cargo build --release --target wasm32-unknown-unknown --workspace

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install Wrangler
        run: npm install -g wrangler

      - name: Deploy to Cloudflare Workers
        run: wrangler deploy
        env:
          CLOUDFLARE_API_TOKEN: ${{ secrets.CLOUDFLARE_API_TOKEN }}
```

---

## مرحله 2: ساخت Cloudflare API Token

### 2.1. Login به Cloudflare Dashboard

برو به: https://dash.cloudflare.com/profile/api-tokens

### 2.2. ساخت Token جدید

1. کلیک روی **"Create Token"**
2. از template **"Edit Cloudflare Workers"** استفاده کن
3. یا **"Create Custom Token"** رو بزن و این دسترسی‌ها رو بده:

**Permissions:**
```
Account - Workers Scripts - Edit
Account - Account Settings - Read
Account - Workers KV Storage - Edit
Account - D1 - Edit
```

**Account Resources:**
```
Include - Your Account - All accounts
```

### 2.3. کپی کردن Token

بعد از ساخت، Token رو کپی کن (فقط یکبار نشون داده میشه!)

---

## مرحله 3: Setup GitHub Secrets

### 3.1. رفتن به Settings

1. برو به repository خودت در GitHub
2. **Settings** → **Secrets and variables** → **Actions**

### 3.2. افزودن Secret

1. کلیک روی **"New repository secret"**
2. Name: `CLOUDFLARE_API_TOKEN`
3. Value: Token که کپی کردی
4. **"Add secret"**

---

## مرحله 4: Setup Cloudflare Resources (یکبار)

قبل از اولین deploy، باید resources رو بسازی. این کار رو **فقط یکبار** باید انجام بدی.

### 4.1. نصب Wrangler (فقط برای setup)

```bash
npm install -g wrangler
```

### 4.2. Login به Cloudflare

```bash
wrangler login
```

### 4.3. ساخت D1 Database

```bash
cd C:\Users\kiarash\Desktop\cod\v1.1.4\cloudflare-bot

wrangler d1 create telegram_bale_db
```

کپی کن `database_id` و توی `wrangler.toml` بذار:

```toml
[[d1_databases]]
binding = "DB"
database_name = "telegram_bale_db"
database_id = "xxxx-xxxx-xxxx-xxxx"  # اینجا
```

### 4.4. اجرای Migration

```bash
wrangler d1 execute telegram_bale_db --file=./migrations/0001_initial_schema.sql
```

### 4.5. ساخت KV Namespace

```bash
wrangler kv:namespace create "bot_kv"
```

کپی کن `id` و توی `wrangler.toml` بذار:

```toml
[[kv_namespaces]]
binding = "KV"
id = "xxxxxxxxxxxxxxxxxxxx"  # اینجا
```

### 4.6. Set کردن Secrets

```bash
wrangler secret put BALE_TOKEN
wrangler secret put TELEGRAM_API_ID
wrangler secret put TELEGRAM_API_HASH
```

**نکته مهم:** Secrets توی Cloudflare ذخیره میشن، نه GitHub!

---

## مرحله 5: Push به GitHub

### 5.1. Init Git (اگه نکردی)

```bash
cd C:\Users\kiarash\Desktop\cod\v1.1.4\cloudflare-bot

git init
git add .
git commit -m "Initial commit - Telegram Bale Bot"
```

### 5.2. افزودن Remote

```bash
# repository خودت رو بساز در GitHub
# بعد:
git remote add origin https://github.com/YOUR_USERNAME/telegram-bale-bot.git
git branch -M main
git push -u origin main
```

---

## مرحله 6: تماشای Deploy

### 6.1. رفتن به Actions Tab

بعد از push، برو به:
```
https://github.com/YOUR_USERNAME/telegram-bale-bot/actions
```

### 6.2. دیدن Progress

باید یک workflow جدید ببینی که داره اجرا میشه:
- ✅ Checkout
- ✅ Setup Rust
- ✅ Build
- ✅ Deploy

حدود **5-10 دقیقه** طول می‌کشه (اولین بار بیشتره)

### 6.3. چک کردن Success

وقتی تموم شد، باید ✅ سبز ببینی.

---

## مرحله 7: Setup Webhook

بعد از deploy موفق، webhook رو set کن:

```bash
curl -X POST "https://tapi.bale.ai/bot<BALE_TOKEN>/setWebhook" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://telegram-bale-bot.YOUR_SUBDOMAIN.workers.dev/bale/webhook"}'
```

---

## 🔄 Workflow بعدی

از این به بعد:

```bash
# تغییرات رو بده
git add .
git commit -m "Update handlers"
git push

# GitHub Actions خودکار:
# 1. Build می‌کنه
# 2. Deploy می‌کنه
# 3. بهت اطلاع میده
```

---

## 📊 Deploy دستی (Manual)

اگه خواستی بدون push deploy کنی:

1. برو به **Actions** tab
2. کلیک روی **"Deploy to Cloudflare Workers"**
3. **"Run workflow"** → **"Run workflow"**

---

## 🐛 Troubleshooting

### ❌ Error: "API token authentication failed"

**راه‌حل:**
1. Token رو دوباره بساز
2. دسترسی‌ها رو چک کن
3. توی GitHub Secrets دوباره set کن

### ❌ Error: "D1 database not found"

**راه‌حل:**
1. مطمئن شو `database_id` توی `wrangler.toml` درسته
2. چک کن database وجود داره: `wrangler d1 list`

### ❌ Error: "Build failed"

**راه‌حل:**
- Logs رو توی GitHub Actions ببین
- احتمالاً یک syntax error در کد هست

---

## 📁 ساختار نهایی پروژه

```
cloudflare-bot/
├── .github/
│   └── workflows/
│       └── deploy.yml          ← GitHub Actions
├── crates/
│   ├── shared/
│   ├── main-bot/
│   ├── session-bot/
│   └── telegram-poller/
├── migrations/
├── wrangler.toml               ← با IDs پر شده
├── Cargo.toml
└── README.md
```

---

## ✅ چک لیست Setup

قبل از push:

- [ ] `.github/workflows/deploy.yml` ساخته شده
- [ ] Cloudflare API Token ساخته شده
- [ ] GitHub Secret `CLOUDFLARE_API_TOKEN` set شده
- [ ] D1 Database ساخته شده و ID در `wrangler.toml`
- [ ] KV Namespace ساخته شده و ID در `wrangler.toml`
- [ ] Secrets در Cloudflare set شدن (`wrangler secret put`)
- [ ] Git init و remote add انجام شده

---

## 🎯 مزایای این روش

✅ **بدون نصب:** هیچ چیزی روی سیستم نصب نمی‌کنی (جز git!)  
✅ **اتوماتیک:** هر push → deploy  
✅ **حرفه‌ای:** CI/CD pipeline کامل  
✅ **سریع:** بعد از اولین بار، cache میشه  
✅ **امن:** API Token توی GitHub secrets  

---

## 💡 نکته مهم

**Secrets (BALE_TOKEN, etc.) فقط در Cloudflare set میشن، نه GitHub!**

این secrets برای runtime هستن. GitHub فقط `CLOUDFLARE_API_TOKEN` رو نیاز داره.

---

**آماده‌ای شروع کنیم؟** 🚀

من می‌تونم فایل `deploy.yml` رو الان برات بسازم!
