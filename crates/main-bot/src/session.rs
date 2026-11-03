use worker::*;

pub fn get_session_ui() -> Response {
    let html = r#"
<!DOCTYPE html>
<html dir="rtl" lang="fa">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ذخیره Session تلگرام</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            justify-content: center;
            align-items: center;
            padding: 20px;
        }
        .container {
            background: white;
            border-radius: 20px;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
            max-width: 600px;
            width: 100%;
            padding: 40px;
        }
        h1 {
            color: #333;
            margin-bottom: 10px;
            font-size: 28px;
        }
        .subtitle {
            color: #666;
            margin-bottom: 30px;
            font-size: 14px;
        }
        .info-box {
            background: #f8f9fa;
            border-right: 4px solid #667eea;
            padding: 20px;
            margin-bottom: 30px;
            border-radius: 8px;
        }
        .info-box h3 {
            color: #667eea;
            margin-bottom: 15px;
            font-size: 18px;
        }
        .info-box ol {
            padding-right: 20px;
            color: #555;
            line-height: 1.8;
        }
        .info-box code {
            background: #e9ecef;
            padding: 2px 8px;
            border-radius: 4px;
            font-family: 'Courier New', monospace;
            color: #d63384;
            font-size: 13px;
        }
        .form-group {
            margin-bottom: 25px;
        }
        label {
            display: block;
            margin-bottom: 8px;
            color: #333;
            font-weight: 600;
            font-size: 14px;
        }
        input, textarea {
            width: 100%;
            padding: 12px 15px;
            border: 2px solid #e0e0e0;
            border-radius: 8px;
            font-size: 14px;
            transition: border-color 0.3s;
            font-family: inherit;
        }
        textarea {
            min-height: 120px;
            resize: vertical;
            font-family: 'Courier New', monospace;
        }
        input:focus, textarea:focus {
            outline: none;
            border-color: #667eea;
        }
        button {
            width: 100%;
            padding: 14px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            border: none;
            border-radius: 8px;
            font-size: 16px;
            font-weight: 600;
            cursor: pointer;
            transition: transform 0.2s, box-shadow 0.2s;
        }
        button:hover {
            transform: translateY(-2px);
            box-shadow: 0 10px 20px rgba(102, 126, 234, 0.4);
        }
        button:active {
            transform: translateY(0);
        }
        .message {
            padding: 15px;
            border-radius: 8px;
            margin-bottom: 20px;
            display: none;
        }
        .message.success {
            background: #d4edda;
            color: #155724;
            border: 1px solid #c3e6cb;
        }
        .message.error {
            background: #f8d7da;
            color: #721c24;
            border: 1px solid #f5c6cb;
        }
        .message.show {
            display: block;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>💬 ذخیره Session تلگرام</h1>
        <p class="subtitle">برای استفاده از ربات، ابتدا Session تلگرام خود را ذخیره کنید</p>

        <div class="info-box">
            <h3>📝 چگونه Session بسازیم؟</h3>
            <ol>
                <li>Python و <code>telethon</code> را نصب کنید: <code>pip install telethon</code></li>
                <li>کد زیر را در یک فایل Python ذخیره و اجرا کنید:</li>
            </ol>
            <pre style="background: #2d2d2d; color: #f8f8f2; padding: 15px; border-radius: 8px; margin: 15px 0; overflow-x: auto; direction: ltr; text-align: left;"><code>from telethon.sync import TelegramClient
from telethon.sessions import StringSession

api_id = 26820590
api_hash = "76c39222f20cc6138bad5f0ce9760a2c"

with TelegramClient(StringSession(), api_id, api_hash) as client:
    print("Session String:")
    print(client.session.save())</code></pre>
            <ol start="3">
                <li>شماره تلفن و کد تایید تلگرام را وارد کنید</li>
                <li>Session String چاپ شده را کپی کرده و در فرم زیر وارد کنید</li>
            </ol>
        </div>

        <div id="message" class="message"></div>

        <form id="sessionForm">
            <div class="form-group">
                <label for="userId">🆔 شناسه کاربری تلگرام شما (User ID)</label>
                <input type="number" id="userId" name="userId" placeholder="مثال: 5585381768" required>
            </div>

            <div class="form-group">
                <label for="sessionString">🔑 Session String تلگرام</label>
                <textarea id="sessionString" name="sessionString" placeholder="Session String خود را اینجا Paste کنید..." required></textarea>
            </div>

            <button type="submit">💾 ذخیره Session</button>
        </form>
    </div>

    <script>
        const form = document.getElementById('sessionForm');
        const messageDiv = document.getElementById('message');

        form.addEventListener('submit', async (e) => {
            e.preventDefault();

            const userId = document.getElementById('userId').value;
            const sessionString = document.getElementById('sessionString').value;

            try {
                const response = await fetch('/session/save', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({
                        user_id: parseInt(userId),
                        session_string: sessionString
                    })
                });

                const result = await response.json();

                if (response.ok) {
                    showMessage('success', '✅ Session با موفقیت ذخیره شد!');
                    form.reset();
                } else {
                    showMessage('error', '❌ خطا: ' + (result.error || 'مشکلی پیش آمد'));
                }
            } catch (error) {
                showMessage('error', '❌ خطا در ارتباط با سرور');
            }
        });

        function showMessage(type, text) {
            messageDiv.className = 'message ' + type + ' show';
            messageDiv.textContent = text;
            setTimeout(() => {
                messageDiv.className = 'message';
            }, 5000);
        }
    </script>
</body>
</html>
    "#;

    Response::from_html(html).unwrap()
}

pub async fn save_session(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    #[derive(serde::Deserialize)]
    struct SessionRequest {
        user_id: i64,
        session_string: String,
    }

    let body: SessionRequest = req.json().await?;

    let db = ctx.env.d1("DB")?;
    let now = chrono::Utc::now().timestamp();

    // Convert session string to base64
    let session_bytes = body.session_string.as_bytes();
    let session_b64 = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        session_bytes
    );

    db.prepare(
        "INSERT INTO telegram_sessions (user_id, session_data, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(user_id) DO UPDATE SET
           session_data = excluded.session_data,
           updated_at = excluded.updated_at"
    )
    .bind(&[
        body.user_id.into(),
        session_b64.into(),
        now.into(),
        now.into(),
    ])?
    .run()
    .await?;

    Response::from_json(&serde_json::json!({
        "success": true,
        "message": "Session saved successfully"
    }))
}

pub async fn get_session(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let url = req.url()?;
    let user_id: i64 = url
        .query_pairs()
        .find(|(key, _)| key == "user_id")
        .and_then(|(_, value)| value.parse().ok())
        .ok_or_else(|| worker::Error::RustError("user_id parameter required".to_string()))?;

    let db = ctx.env.d1("DB")?;

    let result = db
        .prepare("SELECT session_data FROM telegram_sessions WHERE user_id = ?1")
        .bind(&[user_id.into()])?
        .first::<String>(Some("session_data"))
        .await?
        .ok_or_else(|| worker::Error::RustError("Session not found".to_string()))?;

    // Decode base64 session
    let session_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        result
    )
    .map_err(|e| worker::Error::RustError(format!("Base64 decode error: {}", e)))?;

    let session_string = String::from_utf8(session_bytes)
        .map_err(|e| worker::Error::RustError(format!("UTF-8 decode error: {}", e)))?;

    Response::from_json(&serde_json::json!({
        "user_id": user_id,
        "session_string": session_string
    }))
}
