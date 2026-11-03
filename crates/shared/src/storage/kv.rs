use crate::{Error, Result};
use worker::kv::KvStore;

/// KV storage operations
pub struct KVStorage {
    kv: KvStore,
}

impl KVStorage {
    pub fn new(kv: KvStore) -> Self {
        Self { kv }
    }

    // ==================== Throttling ====================

    pub async fn check_rate_limit(&self, key: &str, ttl_seconds: u64) -> Result<bool> {
        let full_key = format!("throttle:{}", key);
        
        match self.kv.get(&full_key).text().await {
            Ok(Some(_)) => Ok(false), // Rate limited
            Ok(None) => {
                // Set throttle key
                self.kv
                    .put(&full_key, "1")
                    .map_err(|e| Error::KVStorage(e.to_string()))?
                    .expiration_ttl(ttl_seconds)
                    .execute()
                    .await
                    .map_err(|e| Error::KVStorage(e.to_string()))?;
                
                Ok(true) // Allowed
            }
            Err(e) => Err(Error::KVStorage(e.to_string())),
        }
    }

    pub async fn set_throttle(&self, key: &str, ttl_seconds: u64) -> Result<()> {
        let full_key = format!("throttle:{}", key);
        
        self.kv
            .put(&full_key, "1")
            .map_err(|e| Error::KVStorage(e.to_string()))?
            .expiration_ttl(ttl_seconds)
            .execute()
            .await
            .map_err(|e| Error::KVStorage(e.to_string()))?;

        Ok(())
    }

    // ==================== Cache ====================

    pub async fn get_cache<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let full_key = format!("cache:{}", key);
        
        match self.kv.get(&full_key).text().await {
            Ok(Some(data)) => {
                let value: T = serde_json::from_str(&data)?;
                Ok(Some(value))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(Error::KVStorage(e.to_string())),
        }
    }

    pub async fn set_cache<T: serde::Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_seconds: u64,
    ) -> Result<()> {
        let full_key = format!("cache:{}", key);
        let data = serde_json::to_string(value)?;
        
        self.kv
            .put(&full_key, data)
            .map_err(|e| Error::KVStorage(e.to_string()))?
            .expiration_ttl(ttl_seconds)
            .execute()
            .await
            .map_err(|e| Error::KVStorage(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_cache(&self, key: &str) -> Result<()> {
        let full_key = format!("cache:{}", key);
        
        self.kv
            .delete(&full_key)
            .await
            .map_err(|e| Error::KVStorage(e.to_string()))?;

        Ok(())
    }

    // ==================== Generic Operations ====================

    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        self.kv
            .get(key)
            .text()
            .await
            .map_err(|e| Error::KVStorage(e.to_string()))
    }

    pub async fn put(&self, key: &str, value: &str, ttl_seconds: Option<u64>) -> Result<()> {
        let mut builder = self.kv
            .put(key, value)
            .map_err(|e| Error::KVStorage(e.to_string()))?;

        if let Some(ttl) = ttl_seconds {
            builder = builder.expiration_ttl(ttl);
        }

        builder
            .execute()
            .await
            .map_err(|e| Error::KVStorage(e.to_string()))?;

        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        self.kv
            .delete(key)
            .await
            .map_err(|e| Error::KVStorage(e.to_string()))?;

        Ok(())
    }
}
