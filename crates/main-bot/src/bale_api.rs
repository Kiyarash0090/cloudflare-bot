use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::*;

#[derive(Debug, Clone)]
pub struct BaleApi {
    token: String,
    base_url: String,
}

impl BaleApi {
    pub fn new(token: String) -> Self {
        Self {
            token: token.clone(),
            base_url: format!("https://tapi.bale.ai/bot{}", token),
        }
    }

    // Send a text message
    pub async fn send_message(
        &self,
        chat_id: i64,
        text: &str,
        parse_mode: Option<&str>,
    ) -> Result<BaleMessageResponse> {
        let url = format!("{}/sendMessage", self.base_url);

        let body = json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": parse_mode.unwrap_or("Markdown"),
        });

        self.make_request(&url, body).await
    }

    // Send message to a specific chat
    pub async fn send_message_to(
        &self,
        chat_id: i64,
        text: &str,
    ) -> Result<BaleMessageResponse> {
        self.send_message(chat_id, text, Some("Markdown")).await
    }

    // Reply to a message
    pub async fn reply_to(
        &self,
        chat_id: i64,
        reply_to_message_id: i64,
        text: &str,
    ) -> Result<BaleMessageResponse> {
        let url = format!("{}/sendMessage", self.base_url);

        let body = json!({
            "chat_id": chat_id,
            "text": text,
            "reply_to_message_id": reply_to_message_id,
            "parse_mode": "Markdown",
        });

        self.make_request(&url, body).await
    }

    // Edit message text
    pub async fn edit_message(
        &self,
        chat_id: i64,
        message_id: i64,
        text: &str,
    ) -> Result<BaleMessageResponse> {
        let url = format!("{}/editMessageText", self.base_url);

        let body = json!({
            "chat_id": chat_id,
            "message_id": message_id,
            "text": text,
            "parse_mode": "Markdown",
        });

        self.make_request(&url, body).await
    }

    // Delete message
    pub async fn delete_message(&self, chat_id: i64, message_id: i64) -> Result<bool> {
        let url = format!("{}/deleteMessage", self.base_url);

        let body = json!({
            "chat_id": chat_id,
            "message_id": message_id,
        });

        let response: BaleApiResponse<bool> = self.make_request(&url, body).await?;
        Ok(response.result.unwrap_or(false))
    }

    // Send photo
    pub async fn send_photo(
        &self,
        chat_id: i64,
        photo_url: &str,
        caption: Option<&str>,
    ) -> Result<BaleMessageResponse> {
        let url = format!("{}/sendPhoto", self.base_url);

        let mut body = json!({
            "chat_id": chat_id,
            "photo": photo_url,
        });

        if let Some(cap) = caption {
            body["caption"] = json!(cap);
            body["parse_mode"] = json!("Markdown");
        }

        self.make_request(&url, body).await
    }

    // Send document
    pub async fn send_document(
        &self,
        chat_id: i64,
        document_url: &str,
        caption: Option<&str>,
    ) -> Result<BaleMessageResponse> {
        let url = format!("{}/sendDocument", self.base_url);

        let mut body = json!({
            "chat_id": chat_id,
            "document": document_url,
        });

        if let Some(cap) = caption {
            body["caption"] = json!(cap);
            body["parse_mode"] = json!("Markdown");
        }

        self.make_request(&url, body).await
    }

    // Send video
    pub async fn send_video(
        &self,
        chat_id: i64,
        video_url: &str,
        caption: Option<&str>,
    ) -> Result<BaleMessageResponse> {
        let url = format!("{}/sendVideo", self.base_url);

        let mut body = json!({
            "chat_id": chat_id,
            "video": video_url,
        });

        if let Some(cap) = caption {
            body["caption"] = json!(cap);
            body["parse_mode"] = json!("Markdown");
        }

        self.make_request(&url, body).await
    }

    // Set webhook
    pub async fn set_webhook(&self, webhook_url: &str) -> Result<bool> {
        let url = format!("{}/setWebhook", self.base_url);

        let body = json!({
            "url": webhook_url,
        });

        let response: BaleApiResponse<bool> = self.make_request(&url, body).await?;
        Ok(response.result.unwrap_or(false))
    }

    // Delete webhook
    pub async fn delete_webhook(&self) -> Result<bool> {
        let url = format!("{}/deleteWebhook", self.base_url);
        let body = json!({});

        let response: BaleApiResponse<bool> = self.make_request(&url, body).await?;
        Ok(response.result.unwrap_or(false))
    }

    // Get bot info
    pub async fn get_me(&self) -> Result<BaleUser> {
        let url = format!("{}/getMe", self.base_url);
        let body = json!({});

        self.make_request(&url, body).await
    }

    // Helper to check if user is admin
    pub fn is_admin(&self, user_id: i64, admin_id: i64) -> bool {
        user_id == admin_id
    }

    // Send photo by file_id (Telegram file_id compatible)
    pub async fn send_photo_by_file_id(
        &self,
        chat_id: i64,
        file_id: &str,
        caption: Option<&str>,
        parse_mode: Option<&str>,
    ) -> Result<BaleMessageResponse> {
        let url = format!("{}/sendPhoto", self.base_url);

        let mut body = json!({
            "chat_id": chat_id,
            "photo": file_id,
        });

        if let Some(cap) = caption {
            body["caption"] = json!(cap);
            body["parse_mode"] = json!(parse_mode.unwrap_or("Markdown"));
        }

        self.make_request(&url, body).await
    }

    // Send document by file_id
    pub async fn send_document_by_file_id(
        &self,
        chat_id: i64,
        file_id: &str,
        file_name: Option<&str>,
        caption: Option<&str>,
        parse_mode: Option<&str>,
    ) -> Result<BaleMessageResponse> {
        let url = format!("{}/sendDocument", self.base_url);

        let mut body = json!({
            "chat_id": chat_id,
            "document": file_id,
        });

        if let Some(name) = file_name {
            body["filename"] = json!(name);
        }

        if let Some(cap) = caption {
            body["caption"] = json!(cap);
            body["parse_mode"] = json!(parse_mode.unwrap_or("Markdown"));
        }

        self.make_request(&url, body).await
    }

    // Send video by file_id
    pub async fn send_video_by_file_id(
        &self,
        chat_id: i64,
        file_id: &str,
        caption: Option<&str>,
        parse_mode: Option<&str>,
    ) -> Result<BaleMessageResponse> {
        let url = format!("{}/sendVideo", self.base_url);

        let mut body = json!({
            "chat_id": chat_id,
            "video": file_id,
        });

        if let Some(cap) = caption {
            body["caption"] = json!(cap);
            body["parse_mode"] = json!(parse_mode.unwrap_or("Markdown"));
        }

        self.make_request(&url, body).await
    }

    // Send animation by file_id
    pub async fn send_animation_by_file_id(
        &self,
        chat_id: i64,
        file_id: &str,
        caption: Option<&str>,
        parse_mode: Option<&str>,
    ) -> Result<BaleMessageResponse> {
        let url = format!("{}/sendAnimation", self.base_url);

        let mut body = json!({
            "chat_id": chat_id,
            "animation": file_id,
        });

        if let Some(cap) = caption {
            body["caption"] = json!(cap);
            body["parse_mode"] = json!(parse_mode.unwrap_or("Markdown"));
        }

        self.make_request(&url, body).await
    }

    // Send audio by file_id
    pub async fn send_audio_by_file_id(
        &self,
        chat_id: i64,
        file_id: &str,
        caption: Option<&str>,
        parse_mode: Option<&str>,
    ) -> Result<BaleMessageResponse> {
        let url = format!("{}/sendAudio", self.base_url);

        let mut body = json!({
            "chat_id": chat_id,
            "audio": file_id,
        });

        if let Some(cap) = caption {
            body["caption"] = json!(cap);
            body["parse_mode"] = json!(parse_mode.unwrap_or("Markdown"));
        }

        self.make_request(&url, body).await
    }

    // Send voice by file_id
    pub async fn send_voice_by_file_id(
        &self,
        chat_id: i64,
        file_id: &str,
        caption: Option<&str>,
        parse_mode: Option<&str>,
    ) -> Result<BaleMessageResponse> {
        let url = format!("{}/sendVoice", self.base_url);

        let mut body = json!({
            "chat_id": chat_id,
            "voice": file_id,
        });

        if let Some(cap) = caption {
            body["caption"] = json!(cap);
            body["parse_mode"] = json!(parse_mode.unwrap_or("Markdown"));
        }

        self.make_request(&url, body).await
    }

    // Generic request maker
    async fn make_request<T: for<'de> Deserialize<'de>>(
        &self,
        url: &str,
        body: serde_json::Value,
    ) -> Result<T> {
        let headers = Headers::new();
        headers.set("Content-Type", "application/json")?;

        let mut init = RequestInit::new();
        init.with_method(Method::Post)
            .with_headers(headers)
            .with_body(Some(body.to_string().into()));

        let request = Request::new_with_init(url, &init)?;

        let mut response = Fetch::Request(request).send().await?;
        let json_response: BaleApiResponse<T> = response.json().await?;

        if json_response.ok {
            json_response
                .result
                .ok_or_else(|| Error::RustError("Empty result from Bale API".to_string()))
        } else {
            Err(Error::RustError(format!(
                "Bale API error: {}",
                json_response.description.unwrap_or_else(|| "Unknown error".to_string())
            )))
        }
    }
}

// Response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaleApiResponse<T> {
    pub ok: bool,
    pub result: Option<T>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaleMessageResponse {
    pub message_id: i64,
    pub from: Option<BaleUser>,
    pub chat: BaleChat,
    pub text: Option<String>,
    pub caption: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaleUser {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub is_bot: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaleChat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
    pub title: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaleUpdate {
    pub update_id: i64,
    pub message: Option<BaleMessageResponse>,
}
