# Telegram-Bale Bot

Rust-based bot for monitoring Telegram channels and forwarding to Bale, deployed on Cloudflare Workers.

## 🚀 Quick Deploy با GitHub Actions

این پروژه به صورت خودکار از طریق GitHub Actions deploy میشه.

### ✅ آماده‌سازی (یکبار)

1. **Cloudflare API Token بساز:**
   - برو به: https://dash.cloudflare.com/profile/api-tokens
   - "Create Token" → "Edit Cloudflare Workers"
   - Token رو کپی کن

2. **GitHub Secret بساز:**
   - Settings → Secrets → Actions → New repository secret
   - Name: `CLOUDFLARE_API_TOKEN`
   - Value: Token کپی شده

3. **Cloudflare Resources بساز:**
   ```bash
   npm install -g wrangler
   wrangler login
   
   # D1 Database
   wrangler d1 create telegram_bale_db
   # Copy database_id به wrangler.toml
   
   # KV Namespace
   wrangler kv:namespace create "bot_kv"
   # Copy id به wrangler.toml
   
   # Secrets
   wrangler secret put BALE_TOKEN
   wrangler secret put TELEGRAM_API_ID
   wrangler secret put TELEGRAM_API_HASH
   ```

4. **Push به GitHub:**
   ```bash
   git add .
   git commit -m "Initial commit"
   git push
   ```

### 🎯 Deploy اتوماتیک

بعد از setup، هر بار که push کنی:
- ✅ GitHub Actions خودکار build می‌کنه
- ✅ به Cloudflare Workers deploy می‌کنه
- ✅ در 5-10 دقیقه آماده است!

---

## 📚 مستندات

- [راهنمای کامل Deploy با GitHub Actions](./GITHUB_ACTIONS_DEPLOY.md)
- [راهنمای Deploy دستی](./DEPLOY_GUIDE.md)
- [راهنمای Setup](./SETUP.md)
- [پلن Migration](./MIGRATION_PLAN.md)
- [خلاصه پروژه](./PROJECT_SUMMARY.md)

---

## 🤖 دستورات Bot

```
/start          - خوشامدگویی
/help           - راهنما
/add_channel    - افزودن کانال تلگرام
/remove_channel - حذف کانال
/list_channels  - لیست کانال‌ها
/start_monitor  - شروع مانیتورینگ
/stop_monitor   - توقف مانیتورینگ
/status         - وضعیت فعلی
```

---

## 🏗️ معماری

- **Language:** Rust
- **Platform:** Cloudflare Workers (Serverless)
- **Storage:** D1 (SQLite) + Workers KV
- **Telegram API:** grammers (MTProto)
- **Bale API:** Custom REST client

---

## 📊 وضعیت پروژه

- ✅ Infrastructure (100%)
- ✅ Main Bot Commands (100%)
- ✅ Database Schema (100%)
- 🚧 Session Bot (10%)
- 🚧 Telegram Poller (10%)

**Overall:** 70% MVP Complete

---

## 🔧 Development

```bash
# Local build
cargo build --workspace

# Test
cargo test --workspace

# Local development
wrangler dev
```

---

## 📝 License

MIT

---

## 👤 Author

Kiarash

---

**Built with ❤️ using Rust + Cloudflare Workers**
