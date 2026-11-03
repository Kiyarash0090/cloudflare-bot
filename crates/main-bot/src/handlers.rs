use crate::bale_api::{BaleApi, BaleUpdate};
use shared::{Channel, ChannelInput, D1Storage, KVStorage};
use worker::*;

const RATE_LIMIT_SECONDS: i64 = 10;

pub struct BotHandler {
    api: BaleApi,
    storage: D1Storage,
    kv: KVStorage,
    admin_user_id: i64,
}

impl BotHandler {
    pub fn new(api: BaleApi, storage: D1Storage, kv: KVStorage, admin_user_id: i64) -> Self {
        Self {
            api,
            storage,
            kv,
            admin_user_id,
        }
    }

    pub async fn handle_update(&self, update: BaleUpdate) -> Result<()> {
        let message = match update.message {
            Some(msg) => msg,
            None => return Ok(()),
        };

        let user_id = match &message.from {
            Some(user) => user.id,
            None => return Ok(()),
        };

        let chat_id = message.chat.id;
        let text = match &message.text {
            Some(t) => t,
            None => return Ok(()),
        };

        // Check if user is admin
        if !self.api.is_admin(user_id, self.admin_user_id) {
            self.api
                .send_message(chat_id, "⛔ شما دسترسی به این ربات را ندارید.", None)
                .await?;
            return Ok(());
        }

        // Check rate limit
        if self.check_rate_limit(user_id).await? {
            self.api
                .send_message(
                    chat_id,
                    &format!("⏳ لطفاً {} ثانیه صبر کنید.", RATE_LIMIT_SECONDS),
                    None,
                )
                .await?;
            return Ok(());
        }

        // Parse command
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        let command = parts[0];
        let args = &parts[1..];

        // Handle commands
        match command {
            "/start" => self.handle_start(chat_id).await?,
            "/help" => self.handle_help(chat_id).await?,
            "/add_channel" => self.handle_add_channel(chat_id, args).await?,
            "/remove_channel" => self.handle_remove_channel(chat_id, args).await?,
            "/list_channels" => self.handle_list_channels(chat_id).await?,
            "/start_monitor" => self.handle_start_monitor(chat_id).await?,
            "/stop_monitor" => self.handle_stop_monitor(chat_id).await?,
            "/status" => self.handle_status(chat_id).await?,
            _ => {
                self.api
                    .send_message(chat_id, "❓ دستور نامعتبر. از /help استفاده کنید.", None)
                    .await?;
            }
        }

        // Update rate limit
        self.update_rate_limit(user_id).await?;

        Ok(())
    }

    async fn handle_start(&self, chat_id: i64) -> Result<()> {
        let message = "🤖 *خوش آمدید به ربات فوروارد تلگرام به بله*\n\n\
                      این ربات پیام‌های کانال‌های تلگرام را به بله فوروارد می‌کند.\n\n\
                      برای شروع از دستور /help استفاده کنید.";
        self.api.send_message(chat_id, message, None).await?;
        Ok(())
    }

    async fn handle_help(&self, chat_id: i64) -> Result<()> {
        let message = "📚 *راهنمای دستورات*\n\n\
                      *مدیریت کانال‌ها:*\n\
                      `/add_channel @username [+size] [+zipSize]`\n\
                      افزودن کانال تلگرام\n\
                      مثال: `/add_channel @mychannel +100 +zip50`\n\n\
                      `/remove_channel @username`\n\
                      حذف کانال\n\n\
                      `/list_channels`\n\
                      نمایش لیست کانال‌ها\n\n\
                      *مدیریت مانیتورینگ:*\n\
                      `/start_monitor` - شروع مانیتورینگ\n\
                      `/stop_monitor` - توقف مانیتورینگ\n\
                      `/status` - نمایش وضعیت\n\n\
                      *نکات:*\n\
                      • فقط ادمین می‌تواند از ربات استفاده کند\n\
                      • بین دستورات 10 ثانیه فاصله بگذارید\n\
                      • حداکثر حجم فایل پیش‌فرض: 20MB";
        self.api.send_message(chat_id, message, None).await?;
        Ok(())
    }

    async fn handle_add_channel(&self, chat_id: i64, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            self.api
                .send_message(
                    chat_id,
                    "❌ استفاده: `/add_channel @username [+size] [+zipSize]`",
                    None,
                )
                .await?;
            return Ok(());
        }

        let input_str = args.join(" ");
        let input = match ChannelInput::parse(&input_str) {
            Ok(i) => i,
            Err(_) => {
                self.api
                    .send_message(chat_id, "❌ فرمت ورودی نامعتبر است.", None)
                    .await?;
                return Ok(());
            }
        };

        // Create channel
        // Note: telegram_id will be fetched from Telegram API in future
        // For now, use a placeholder that won't conflict with real IDs
        let temp_id = chrono::Utc::now().timestamp();
        let mut channel = Channel::new(temp_id, input.identifier.clone());
        channel.username = Some(input.identifier.clone());
        channel.custom_max_size_mb = input.custom_size;
        channel.zip_max_size_mb = input.zip_size;

        // Save to database
        match self.storage.add_channel(&channel).await {
            Ok(id) => {
                let size_info = if let Some(size) = input.custom_size {
                    format!("\n📦 حداکثر حجم: {}MB", size)
                } else if let Some(zip) = input.zip_size {
                    format!("\n📦 حداکثر حجم (زیپ): {}MB", zip)
                } else {
                    "\n📦 حداکثر حجم: 20MB (پیش‌فرض)".to_string()
                };

                let message = format!(
                    "✅ کانال با موفقیت اضافه شد\n\n\
                     🆔 ID: {}\n\
                     📢 کانال: @{}{}",
                    id, input.identifier, size_info
                );
                self.api.send_message(chat_id, &message, None).await?;
            }
            Err(e) => {
                self.api
                    .send_message(chat_id, &format!("❌ خطا در افزودن کانال: {}", e), None)
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_remove_channel(&self, chat_id: i64, args: &[&str]) -> Result<()> {
        if args.is_empty() {
            self.api
                .send_message(chat_id, "❌ استفاده: `/remove_channel @username`", None)
                .await?;
            return Ok(());
        }

        let _username = args[0].trim_start_matches('@');

        // For now, we need to get channel by username
        // This is a limitation - we'd need to add a get_channel_by_username method
        self.api
            .send_message(
                chat_id,
                "⚠️ برای حذف کانال، ابتدا از /list_channels شناسه کانال را پیدا کنید.",
                None,
            )
            .await?;

        Ok(())
    }

    async fn handle_list_channels(&self, chat_id: i64) -> Result<()> {
        match self.storage.list_channels().await {
            Ok(channels) => {
                if channels.is_empty() {
                    self.api
                        .send_message(chat_id, "📋 هیچ کانالی ثبت نشده است.", None)
                        .await?;
                    return Ok(());
                }

                let mut message = "📋 *لیست کانال‌ها:*\n\n".to_string();
                for (idx, channel) in channels.iter().enumerate() {
                    let status = if channel.enabled { "✅" } else { "❌" };
                    let username = channel.username.as_deref().unwrap_or("نامشخص");
                    let size = channel.max_file_size_mb();

                    message.push_str(&format!(
                        "{}. {} @{}\n   🆔 ID: {}\n   📦 حجم: {}MB\n\n",
                        idx + 1,
                        status,
                        username,
                        channel.id.unwrap_or(0),
                        size
                    ));
                }

                self.api.send_message(chat_id, &message, None).await?;
            }
            Err(e) => {
                self.api
                    .send_message(chat_id, &format!("❌ خطا در دریافت لیست: {}", e), None)
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_start_monitor(&self, chat_id: i64) -> Result<()> {
        match self.storage.start_monitoring().await {
            Ok(_) => {
                self.api
                    .send_message(chat_id, "✅ مانیتورینگ شروع شد.", None)
                    .await?;
            }
            Err(e) => {
                self.api
                    .send_message(chat_id, &format!("❌ خطا در شروع مانیتورینگ: {}", e), None)
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_stop_monitor(&self, chat_id: i64) -> Result<()> {
        match self.storage.stop_monitoring().await {
            Ok(_) => {
                self.api
                    .send_message(chat_id, "⏸️ مانیتورینگ متوقف شد.", None)
                    .await?;
            }
            Err(e) => {
                self.api
                    .send_message(chat_id, &format!("❌ خطا در توقف مانیتورینگ: {}", e), None)
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_status(&self, chat_id: i64) -> Result<()> {
        // Get monitoring state
        let state = match self.storage.get_monitoring_state().await {
            Ok(s) => s,
            Err(e) => {
                self.api
                    .send_message(chat_id, &format!("❌ خطا در دریافت وضعیت: {}", e), None)
                    .await?;
                return Ok(());
            }
        };

        // Get channels count
        let channels = self.storage.list_channels().await.unwrap_or_default();
        let enabled_count = channels.iter().filter(|c| c.enabled).count();

        let status_emoji = if state.is_running { "🟢" } else { "🔴" };
        let status_text = if state.is_running {
            "در حال اجرا"
        } else {
            "متوقف"
        };

        let started_time = state
            .started_at
            .map(|t| shared::utils::format_timestamp(t))
            .unwrap_or_else(|| "—".to_string());

        let message = format!(
            "📊 *وضعیت سیستم*\n\n\
             {} وضعیت: *{}*\n\
             📢 کانال‌های فعال: {} از {}\n\
             🆔 آخرین Update ID: {}\n\
             🕐 زمان شروع: {}\n\n\
             💡 برای مدیریت از دستورات /help استفاده کنید.",
            status_emoji,
            status_text,
            enabled_count,
            channels.len(),
            state.last_update_id,
            started_time
        );

        self.api.send_message(chat_id, &message, None).await?;
        Ok(())
    }

    async fn check_rate_limit(&self, user_id: i64) -> Result<bool> {
        let key = format!("ratelimit:{}", user_id);
        let now = chrono::Utc::now().timestamp();

        if let Some(last_time) = self.kv.get_i64(&key).await? {
            if now - last_time < RATE_LIMIT_SECONDS {
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn update_rate_limit(&self, user_id: i64) -> Result<()> {
        let key = format!("ratelimit:{}", user_id);
        let now = chrono::Utc::now().timestamp();
        self.kv.set_i64(&key, now, Some(60)).await?;
        Ok(())
    }
}
