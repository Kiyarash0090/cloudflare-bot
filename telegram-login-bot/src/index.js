/**
 * Telegram Session Storage Bot
 * Accepts session strings from users and saves them to D1
 */

export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    if (url.pathname === '/telegram/webhook' && request.method === 'POST') {
      return handleTelegramWebhook(request, env);
    }

    if (url.pathname === '/health' && request.method === 'GET') {
      return new Response(JSON.stringify({ status: 'ok', service: 'telegram-session-bot' }), {
        headers: { 'Content-Type': 'application/json' }
      });
    }

    return new Response('Not Found', { status: 404 });
  }
};

async function handleTelegramWebhook(request, env) {
  try {
    const update = await request.json();

    // Only respond to admin user
    const adminUserId = parseInt(env.ADMIN_USER_ID);
    const userId = update.message?.from?.id;

    if (!userId || userId !== adminUserId) {
      return new Response('OK', { status: 200 });
    }

    if (update.message) {
      await handleMessage(update.message, env);
    }

    return new Response('OK', { status: 200 });
  } catch (error) {
    console.error('Error handling webhook:', error);
    return new Response('OK', { status: 200 });
  }
}

async function handleMessage(message, env) {
  const chatId = message.chat.id;
  const text = message.text?.trim();
  const userId = message.from.id;

  // Get current state
  const stateKey = `session_state:${userId}`;
  const state = await env.KV.get(stateKey, 'json');

  // Handle commands
  if (text === '/start' || text === '/login') {
    const instructions = `🔐 *ذخیره Session تلگرام*

برای استفاده از ربات، Session تلگرام خود را ذخیره کنید.

📝 *مراحل:*

1️⃣ Python و telethon را نصب کنید:
\`\`\`
pip install telethon
\`\`\`

2️⃣ کد زیر را در یک فایل Python ذخیره و اجرا کنید:

\`\`\`python
from telethon.sync import TelegramClient
from telethon.sessions import StringSession

api_id = 26820590
api_hash = "76c39222f20cc6138bad5f0ce9760a2c"

with TelegramClient(StringSession(), api_id, api_hash) as client:
    print("Your Session String:")
    print(client.session.save())
\`\`\`

3️⃣ شماره تلفن، کد تایید و در صورت فعال بودن، رمز دو مرحله‌ای را وارد کنید.

4️⃣ Session String چاپ شده را کپی کنید.

5️⃣ Session String را به این ربات ارسال کنید.

⚠️ *توجه:* Session String شما مانند رمز عبور است. آن را با کسی به اشتراک نگذارید.

برای لغو: /cancel`;

    await sendMessage(env, chatId, instructions, { parse_mode: 'Markdown' });

    // Set state to waiting for session
    await env.KV.put(stateKey, JSON.stringify({
      step: 'waiting_session',
      timestamp: Date.now()
    }), { expirationTtl: 600 });

    return;
  }

  if (text === '/cancel') {
    await env.KV.delete(stateKey);
    await sendMessage(env, chatId, '❌ عملیات لغو شد.');
    return;
  }

  if (text === '/status') {
    const session = await getSessionFromDatabase(env, userId);
    if (session) {
      await sendMessage(env, chatId, '✅ Session شما ذخیره شده است.\n\nبرای ذخیره Session جدید: /login');
    } else {
      await sendMessage(env, chatId, '❌ Session ذخیره نشده است.\n\nبرای ذخیره: /login');
    }
    return;
  }

  // If no state, prompt user to start
  if (!state) {
    await sendMessage(env, chatId, 'لطفاً ابتدا /login را بزنید تا راهنمایی دریافت کنید.');
    return;
  }

  // Handle session string input
  if (state.step === 'waiting_session') {
    await handleSessionString(env, chatId, userId, text);
  }
}

async function handleSessionString(env, chatId, userId, sessionString) {
  const stateKey = `session_state:${userId}`;

  try {
    // Validate session string format (basic check)
    if (!sessionString || sessionString.length < 100) {
      await sendMessage(env, chatId,
        '❌ Session String نامعتبر است.\n\n' +
        'لطفاً مطمئن شوید که Session String کامل را ارسال کرده‌اید.\n\n' +
        'برای دریافت راهنمایی مجدد: /login'
      );
      return;
    }

    await sendMessage(env, chatId, '⏳ در حال ذخیره...');

    // Save to database
    await saveSessionToDatabase(env, userId, sessionString);

    await sendMessage(env, chatId,
      '✅ *Session با موفقیت ذخیره شد!*\n\n' +
      '🎉 اکنون می‌توانید از ربات Bale برای مانیتورینگ کانال‌های تلگرام استفاده کنید.\n\n' +
      'برای بررسی وضعیت: /status',
      { parse_mode: 'Markdown' }
    );

    // Clear state
    await env.KV.delete(stateKey);

  } catch (error) {
    console.error('Error saving session:', error);
    await sendMessage(env, chatId,
      '❌ خطا در ذخیره Session.\n\n' +
      'لطفاً دوباره تلاش کنید: /login'
    );
    await env.KV.delete(stateKey);
  }
}

async function saveSessionToDatabase(env, userId, sessionString) {
  // Encode session as base64
  const sessionBytes = new TextEncoder().encode(sessionString);
  const base64Session = btoa(String.fromCharCode(...sessionBytes));

  const now = Math.floor(Date.now() / 1000);

  await env.DB.prepare(
    `INSERT INTO telegram_sessions (user_id, session_data, created_at, updated_at)
     VALUES (?1, ?2, ?3, ?4)
     ON CONFLICT(user_id) DO UPDATE SET
       session_data = excluded.session_data,
       updated_at = excluded.updated_at`
  )
    .bind(userId, base64Session, now, now)
    .run();
}

async function getSessionFromDatabase(env, userId) {
  try {
    const result = await env.DB.prepare(
      'SELECT session_data FROM telegram_sessions WHERE user_id = ?1'
    )
      .bind(userId)
      .first();

    return result;
  } catch (error) {
    console.error('Error getting session:', error);
    return null;
  }
}

async function sendMessage(env, chatId, text, options = {}) {
  const botToken = env.TELEGRAM_BOT_TOKEN;
  const url = `https://api.telegram.org/bot${botToken}/sendMessage`;

  const body = {
    chat_id: chatId,
    text: text,
    ...options
  };

  await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body)
  });
}
