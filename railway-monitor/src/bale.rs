use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

#[derive(Clone)]
pub struct BaleClient {
    client: HttpClient,
    token: String,
    base_url: String,
}

#[derive(Debug, Deserialize)]
struct BaleResponse<T> {
    ok: bool,
    result: Option<T>,
    description: Option<String>,
}

impl BaleClient {
    pub fn new(token: String) -> Self {
        let client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: format!("https://tapi.bale.ai/bot{}", token),
            token,
        }
    }

    pub async fn send_message(&self, chat_id: i64, text: &str) -> anyhow::Result<()> {
        let url = format!("{}/sendMessage", self.base_url);
        let body = json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "Markdown"
        });

        let resp: BaleResponse<serde_json::Value> = self.client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        if !resp.ok {
            return Err(anyhow::anyhow!("Bale API error: {:?}", resp.description));
        }

        Ok(())
    }

    pub async fn send_photo(&self, chat_id: i64, file_id: &str, caption: Option<&str>) -> anyhow::Result<()> {
        let url = format!("{}/sendPhoto", self.base_url);
        let mut body = json!({
            "chat_id": chat_id,
            "photo": file_id,
        });

        if let Some(cap) = caption {
            body["caption"] = json!(cap);
            body["parse_mode"] = json!("Markdown");
        }

        let resp: BaleResponse<serde_json::Value> = self.client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        if !resp.ok {
            return Err(anyhow::anyhow!("Bale API error: {:?}", resp.description));
        }

        Ok(())
    }

    pub async fn send_document(&self, chat_id: i64, file_id: &str, file_name: Option<&str>, caption: Option<&str>) -> anyhow::Result<()> {
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
            body["parse_mode"] = json!("Markdown");
        }

        let resp: BaleResponse<serde_json::Value> = self.client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        if !resp.ok {
            return Err(anyhow::anyhow!("Bale API error: {:?}", resp.description));
        }

        Ok(())
    }

    pub async fn send_video(&self, chat_id: i64, file_id: &str, caption: Option<&str>) -> anyhow::Result<()> {
        let url = format!("{}/sendVideo", self.base_url);
        let mut body = json!({
            "chat_id": chat_id,
            "video": file_id,
        });

        if let Some(cap) = caption {
            body["caption"] = json!(cap);
            body["parse_mode"] = json!("Markdown");
        }

        let resp: BaleResponse<serde_json::Value> = self.client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        if !resp.ok {
            return Err(anyhow::anyhow!("Bale API error: {:?}", resp.description));
        }

        Ok(())
    }

    pub async fn forward_message(&self, chat_id: i64, from_chat_id: i64, message_id: i64) -> anyhow::Result<()> {
        let url = format!("{}/forwardMessage", self.base_url);
        let body = json!({
            "chat_id": chat_id,
            "from_chat_id": from_chat_id,
            "message_id": message_id,
        });

        let resp: BaleResponse<serde_json::Value> = self.client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        if !resp.ok {
            return Err(anyhow::anyhow!("Bale API error: {:?}", resp.description));
        }

        Ok(())
    }
}