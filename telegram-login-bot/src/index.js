/**
 * Telegram Login Bot
 * Handles Telegram authentication flow using gramjs
 */

import { TelegramClient } from 'telegram';
import { StringSession } from 'telegram/sessions';

export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    if (url.pathname === '/telegram/webhook' && request.method === 'POST') {
      return handleTelegramWebhook(request, env);
    }

    if (url.pathname === '/health' && request.method === 'GET') {
      return new Response(JSON.stringify({ status: 'ok', service: 'telegram-login-bot' }), {
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
    const userId = update.message?.from?.id || update.callback_query?.from?.id;

    if (userId !== adminUserId) {
      return new Response('OK', { status: 200 });
    }

    if (update.message) {
      await handleMessage(update.message, env);
    } else if (update.callback_query) {
      await handleCallbackQuery(update.callback_query, env);
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
  const stateKey = `login_state:${userId}`;
  const state = await env.KV.get(stateKey, 'json');

  // Handle commands
  if (text === '/start' || text === '/login') {
    await sendMessage(env, chatId,
      '🔐 *ورود به تلگرام*\n\n' +
      'برای ساختن Session تلگرام، شماره موبایل خود را ارسال کنید.\n\n' +
      '📱 فرمت: `+989123456789`',
      { parse_mode: 'Markdown' }
    );

    // Reset state
    await env.KV.put(stateKey, JSON.stringify({ step: 'waiting_phone' }), { expirationTtl: 600 });
    return;
  }

  if (text === '/cancel') {
    await env.KV.delete(stateKey);
    await sendMessage(env, chatId, '❌ عملیات لغو شد.');
    return;
  }

  if (!state) {
    await sendMessage(env, chatId, 'لطفاً ابتدا /login را بزنید.');
    return;
  }

  // Handle login flow based on state
  if (state.step === 'waiting_phone') {
    await handlePhoneNumber(env, chatId, userId, text, state);
  } else if (state.step === 'waiting_code') {
    await handleCode(env, chatId, userId, text, state);
  } else if (state.step === 'waiting_password') {
    await handlePassword(env, chatId, userId, text, state);
  }
}

async function handlePhoneNumber(env, chatId, userId, phone, state) {
  const stateKey = `login_state:${userId}`;

  try {
    // Validate phone format
    if (!phone.startsWith('+')) {
      await sendMessage(env, chatId, '❌ شماره باید با + شروع شود.\nمثال: +989123456789');
      return;
    }

    await sendMessage(env, chatId, '⏳ در حال ارسال کد...');

    // Initialize Telegram client
    const apiId = parseInt(env.TELEGRAM_API_ID);
    const apiHash = env.TELEGRAM_API_HASH;
    const client = new TelegramClient(new StringSession(''), apiId, apiHash, {
      connectionRetries: 5,
    });

    await client.connect();

    // Send code
    const result = await client.sendCode({
      apiId,
      apiHash,
    }, phone);

    await client.disconnect();

    // Save state with phone hash
    await env.KV.put(stateKey, JSON.stringify({
      step: 'waiting_code',
      phone,
      phoneCodeHash: result.phoneCodeHash,
      timestamp: Date.now()
    }), { expirationTtl: 600 });

    await sendMessage(env, chatId,
      '✅ کد تایید ارسال شد!\n\n' +
      '📲 کد 5 رقمی که از تلگرام دریافت کردید را ارسال کنید.\n\n' +
      'برای لغو: /cancel'
    );

  } catch (error) {
    console.error('Error sending code:', error);
    await sendMessage(env, chatId, '❌ خطا در ارسال کد. لطفاً دوباره تلاش کنید: /login');
    await env.KV.delete(stateKey);
  }
}

async function handleCode(env, chatId, userId, code, state) {
  const stateKey = `login_state:${userId}`;

  try {
    await sendMessage(env, chatId, '⏳ در حال تایید...');

    const apiId = parseInt(env.TELEGRAM_API_ID);
    const apiHash = env.TELEGRAM_API_HASH;
    const client = new TelegramClient(new StringSession(''), apiId, apiHash, {
      connectionRetries: 5,
    });

    await client.connect();

    // Try to sign in with code
    try {
      await client.invoke({
        _: 'auth.signIn',
        phoneNumber: state.phone,
        phoneCodeHash: state.phoneCodeHash,
        phoneCode: code,
      });

      // Success! Get session string
      const sessionString = client.session.save();

      await client.disconnect();

      // Save to database
      await saveSessionToDatabase(env, userId, sessionString);

      await sendMessage(env, chatId,
        '✅ *ورود موفقیت‌آمیز!*\n\n' +
        '🎉 Session تلگرام شما ذخیره شد.\n' +
        'اکنون می‌توانید از ربات Bale استفاده کنید.',
        { parse_mode: 'Markdown' }
      );

      await env.KV.delete(stateKey);

    } catch (error) {
      // Check if 2FA is required
      if (error.errorMessage === 'SESSION_PASSWORD_NEEDED') {
        await client.disconnect();

        await env.KV.put(stateKey, JSON.stringify({
          ...state,
          step: 'waiting_password'
        }), { expirationTtl: 600 });

        await sendMessage(env, chatId,
          '🔐 *احراز هویت دو مرحله‌ای*\n\n' +
          'رمز عبور Cloud Password تلگرام خود را وارد کنید.\n\n' +
          'برای لغو: /cancel',
          { parse_mode: 'Markdown' }
        );
        return;
      }

      throw error;
    }

  } catch (error) {
    console.error('Error verifying code:', error);
    await sendMessage(env, chatId,
      '❌ کد اشتباه است یا منقضی شده.\n' +
      'لطفاً دوباره تلاش کنید: /login'
    );
    await env.KV.delete(stateKey);
  }
}

async function handlePassword(env, chatId, userId, password, state) {
  const stateKey = `login_state:${userId}`;

  try {
    await sendMessage(env, chatId, '⏳ در حال تایید رمز...');

    const apiId = parseInt(env.TELEGRAM_API_ID);
    const apiHash = env.TELEGRAM_API_HASH;
    const client = new TelegramClient(new StringSession(''), apiId, apiHash, {
      connectionRetries: 5,
    });

    await client.connect();

    // Sign in with password
    await client.invoke({
      _: 'auth.checkPassword',
      password: {
        _: 'inputCheckPasswordSRP',
        srpId: state.srpId,
        A: state.A,
        M1: password,
      }
    });

    // Success! Get session string
    const sessionString = client.session.save();

    await client.disconnect();

    // Save to database
    await saveSessionToDatabase(env, userId, sessionString);

    await sendMessage(env, chatId,
      '✅ *ورود موفقیت‌آمیز!*\n\n' +
      '🎉 Session تلگرام شما با رمز دو مرحله‌ای ذخیره شد.\n' +
      'اکنون می‌توانید از ربات Bale استفاده کنید.',
      { parse_mode: 'Markdown' }
    );

    await env.KV.delete(stateKey);

  } catch (error) {
    console.error('Error verifying password:', error);
    await sendMessage(env, chatId,
      '❌ رمز اشتباه است.\n' +
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

async function handleCallbackQuery(callbackQuery, env) {
  // Handle button callbacks if needed
  const chatId = callbackQuery.message.chat.id;
  const data = callbackQuery.data;

  await fetch(`https://api.telegram.org/bot${env.TELEGRAM_BOT_TOKEN}/answerCallbackQuery`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ callback_query_id: callbackQuery.id })
  });
}
