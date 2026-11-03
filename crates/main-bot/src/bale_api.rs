use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::{BaleApiResponse, BaleMessage, Error, Result};

const BALE_API_BASE: &str = "https://tapi.bale.ai";

/// Bale Bot API client
pub struct BaleBot {
    token: String,
    client: Client,
    chat_id: i64,
}

impl BaleBot {
    pub fn new(token: String, chat_id: i64) -> Self {
        Self {
            token,
            client: Client::new(),
            chat_id,
        }
    }

    fn api_url(&self, method: &str) -> String {
        format!("{}/bot{}/{}", BALE_API_BASE, self.token, method)
    }

    // ==================== Messaging ====================

    pub async fn send_message(&self, text: &str) -> Result<BaleMessage> {
        self.send_message_to(self.chat_id, text).await
    }

    pub async fn send_message_to(&self, chat_id: i64, text: &str) -> Result<BaleMessage> {
        let response = self
            .client
            .post(self.api_url("sendMessage"))
            .json(&json!({
                "chat_id": chat_id,
                "text": text,
                "parse_mode": "Markdown",
            }))
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        let result: BaleApiResponse<BaleMessage> = response
            .json()
            .await
            .map_err(|e| Error::Bale(e.to_string()))?;

        if result.ok {
            result.result.ok_or_else(|| Error::Bale("No result in response".to_string()))
        } else {
            Err(Error::Bale(
                result.description.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    pub async fn reply_to(&self, message_id: i64, text: &str) -> Result<BaleMessage> {
        let response = self
            .client
            .post(self.api_url("sendMessage"))
            .json(&json!({
                "chat_id": self.chat_id,
                "text": text,
                "reply_to_message_id": message_id,
                "parse_mode": "Markdown",
            }))
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        let result: BaleApiResponse<BaleMessage> = response
            .json()
            .await
            .map_err(|e| Error::Bale(e.to_string()))?;

        if result.ok {
            result.result.ok_or_else(|| Error::Bale("No result in response".to_string()))
        } else {
            Err(Error::Bale(
                result.description.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    pub async fn edit_message(&self, message_id: i64, text: &str) -> Result<()> {
        let response = self
            .client
            .post(self.api_url("editMessageText"))
            .json(&json!({
                "chat_id": self.chat_id,
                "message_id": message_id,
                "text": text,
                "parse_mode": "Markdown",
            }))
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        let result: BaleApiResponse<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| Error::Bale(e.to_string()))?;

        if result.ok {
            Ok(())
        } else {
            Err(Error::Bale(
                result.description.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    pub async fn delete_message(&self, message_id: i64) -> Result<()> {
        let response = self
            .client
            .post(self.api_url("deleteMessage"))
            .json(&json!({
                "chat_id": self.chat_id,
                "message_id": message_id,
            }))
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        let result: BaleApiResponse<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| Error::Bale(e.to_string()))?;

        if result.ok {
            Ok(())
        } else {
            Err(Error::Bale(
                result.description.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    // ==================== Media ====================

    pub async fn send_photo(&self, photo_url: &str, caption: Option<&str>) -> Result<BaleMessage> {
        let mut payload = json!({
            "chat_id": self.chat_id,
            "photo": photo_url,
        });

        if let Some(cap) = caption {
            payload["caption"] = json!(cap);
            payload["parse_mode"] = json!("Markdown");
        }

        let response = self
            .client
            .post(self.api_url("sendPhoto"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        let result: BaleApiResponse<BaleMessage> = response
            .json()
            .await
            .map_err(|e| Error::Bale(e.to_string()))?;

        if result.ok {
            result.result.ok_or_else(|| Error::Bale("No result in response".to_string()))
        } else {
            Err(Error::Bale(
                result.description.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    pub async fn send_document(&self, document_url: &str, caption: Option<&str>) -> Result<BaleMessage> {
        let mut payload = json!({
            "chat_id": self.chat_id,
            "document": document_url,
        });

        if let Some(cap) = caption {
            payload["caption"] = json!(cap);
            payload["parse_mode"] = json!("Markdown");
        }

        let response = self
            .client
            .post(self.api_url("sendDocument"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        let result: BaleApiResponse<BaleMessage> = response
            .json()
            .await
            .map_err(|e| Error::Bale(e.to_string()))?;

        if result.ok {
            result.result.ok_or_else(|| Error::Bale("No result in response".to_string()))
        } else {
            Err(Error::Bale(
                result.description.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    pub async fn send_video(&self, video_url: &str, caption: Option<&str>) -> Result<BaleMessage> {
        let mut payload = json!({
            "chat_id": self.chat_id,
            "video": video_url,
        });

        if let Some(cap) = caption {
            payload["caption"] = json!(cap);
            payload["parse_mode"] = json!("Markdown");
        }

        let response = self
            .client
            .post(self.api_url("sendVideo"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        let result: BaleApiResponse<BaleMessage> = response
            .json()
            .await
            .map_err(|e| Error::Bale(e.to_string()))?;

        if result.ok {
            result.result.ok_or_else(|| Error::Bale("No result in response".to_string()))
        } else {
            Err(Error::Bale(
                result.description.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    // ==================== Webhook ====================

    pub async fn set_webhook(&self, url: &str) -> Result<()> {
        let response = self
            .client
            .post(self.api_url("setWebhook"))
            .json(&json!({
                "url": url,
            }))
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        let result: BaleApiResponse<bool> = response
            .json()
            .await
            .map_err(|e| Error::Bale(e.to_string()))?;

        if result.ok {
            Ok(())
        } else {
            Err(Error::Bale(
                result.description.unwrap_or_else(|| "Failed to set webhook".to_string())
            ))
        }
    }

    pub async fn delete_webhook(&self) -> Result<()> {
        let response = self
            .client
            .post(self.api_url("deleteWebhook"))
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;

        let result: BaleApiResponse<bool> = response
            .json()
            .await
            .map_err(|e| Error::Bale(e.to_string()))?;

        if result.ok {
            Ok(())
        } else {
            Err(Error::Bale(
                result.description.unwrap_or_else(|| "Failed to delete webhook".to_string())
            ))
        }
    }

    // ==================== Helpers ====================

    pub fn is_admin(&self, user_id: i64, admin_id: i64) -> bool {
        user_id == admin_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_url() {
        let bot = BaleBot::new("test_token".to_string(), 123456);
        assert_eq!(
            bot.api_url("sendMessage"),
            "https://tapi.bale.ai/bottest_token/sendMessage"
        );
    }
}
