use crate::{Channel, Error, MonitoringState, Result, TelegramSession};
use worker::D1Database;

/// D1 storage operations
pub struct D1Storage {
    db: D1Database,
}

impl D1Storage {
    pub fn new(db: D1Database) -> Self {
        Self { db }
    }

    // ==================== Session Operations ====================

    pub async fn save_session(&self, user_id: i64, session_data: &[u8]) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        
        self.db
            .prepare(
                "INSERT INTO telegram_sessions (user_id, session_data, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(user_id) DO UPDATE SET
                   session_data = excluded.session_data,
                   updated_at = excluded.updated_at"
            )
            .bind(&[
                user_id.into(),
                session_data.into(),
                now.into(),
                now.into(),
            ])?
            .run()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_session(&self, user_id: i64) -> Result<Option<TelegramSession>> {
        let result = self.db
            .prepare("SELECT id, user_id, session_data, created_at, updated_at FROM telegram_sessions WHERE user_id = ?1")
            .bind(&[user_id.into()])?
            .first::<TelegramSession>(None)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(result)
    }

    pub async fn delete_session(&self, user_id: i64) -> Result<()> {
        self.db
            .prepare("DELETE FROM telegram_sessions WHERE user_id = ?1")
            .bind(&[user_id.into()])?
            .run()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    // ==================== Channel Operations ====================

    pub async fn add_channel(&self, channel: &Channel) -> Result<i64> {
        let now = chrono::Utc::now().timestamp();
        
        let result = self.db
            .prepare(
                "INSERT INTO channels (telegram_id, name, username, link, custom_max_size_mb, zip_max_size_mb, enabled, added_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 RETURNING id"
            )
            .bind(&[
                channel.telegram_id.into(),
                channel.name.clone().into(),
                channel.username.clone().into(),
                channel.link.clone().into(),
                channel.custom_max_size_mb.into(),
                channel.zip_max_size_mb.into(),
                (channel.enabled as i64).into(),
                now.into(),
                now.into(),
            ])?
            .first::<i64>(Some("id"))
            .await
            .map_err(|e| Error::Database(e.to_string()))?
            .ok_or_else(|| Error::Database("Failed to get inserted channel ID".to_string()))?;

        Ok(result)
    }

    pub async fn remove_channel(&self, telegram_id: i64) -> Result<()> {
        self.db
            .prepare("DELETE FROM channels WHERE telegram_id = ?1")
            .bind(&[telegram_id.into()])?
            .run()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_channel(&self, telegram_id: i64) -> Result<Option<Channel>> {
        let result = self.db
            .prepare("SELECT * FROM channels WHERE telegram_id = ?1")
            .bind(&[telegram_id.into()])?
            .first::<Channel>(None)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(result)
    }

    pub async fn get_all_channels(&self) -> Result<Vec<Channel>> {
        let result = self.db
            .prepare("SELECT * FROM channels ORDER BY name")
            .all()
            .await
            .map_err(|e| Error::Database(e.to_string()))?
            .results::<Channel>()
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(result)
    }

    pub async fn get_enabled_channels(&self) -> Result<Vec<Channel>> {
        let result = self.db
            .prepare("SELECT * FROM channels WHERE enabled = 1 ORDER BY name")
            .all()
            .await
            .map_err(|e| Error::Database(e.to_string()))?
            .results::<Channel>()
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(result)
    }

    pub async fn update_channel(&self, channel: &Channel) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        
        self.db
            .prepare(
                "UPDATE channels SET
                   name = ?1,
                   username = ?2,
                   link = ?3,
                   custom_max_size_mb = ?4,
                   zip_max_size_mb = ?5,
                   enabled = ?6,
                   updated_at = ?7
                 WHERE telegram_id = ?8"
            )
            .bind(&[
                channel.name.clone().into(),
                channel.username.clone().into(),
                channel.link.clone().into(),
                channel.custom_max_size_mb.into(),
                channel.zip_max_size_mb.into(),
                (channel.enabled as i64).into(),
                now.into(),
                channel.telegram_id.into(),
            ])?
            .run()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    // ==================== Monitoring State Operations ====================

    pub async fn get_monitoring_state(&self) -> Result<MonitoringState> {
        let result = self.db
            .prepare("SELECT * FROM monitoring_state WHERE id = 1")
            .first::<MonitoringState>(None)
            .await
            .map_err(|e| Error::Database(e.to_string()))?
            .ok_or_else(|| Error::Database("Monitoring state not found".to_string()))?;

        Ok(result)
    }

    pub async fn start_monitoring(&self) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        
        self.db
            .prepare("UPDATE monitoring_state SET is_running = 1, started_at = ?1 WHERE id = 1")
            .bind(&[now.into()])?
            .run()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn stop_monitoring(&self) -> Result<()> {
        let now = chrono::Utc::now().timestamp();
        
        self.db
            .prepare("UPDATE monitoring_state SET is_running = 0, stopped_at = ?1 WHERE id = 1")
            .bind(&[now.into()])?
            .run()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn update_last_update_id(&self, update_id: i64) -> Result<()> {
        self.db
            .prepare("UPDATE monitoring_state SET last_update_id = ?1 WHERE id = 1")
            .bind(&[update_id.into()])?
            .run()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }
}
