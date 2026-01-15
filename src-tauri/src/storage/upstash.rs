use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};
use std::env;

#[derive(Clone)]
pub struct UpstashClient {
    client: Client,
    url: String,
    token: String,
}

impl UpstashClient {
    pub fn new() -> Result<Self, String> {
        let url = env::var("UPSTASH_REDIS_REST_URL")
            .map_err(|_| "UPSTASH_REDIS_REST_URL not set".to_string())?;
        let token = env::var("UPSTASH_REDIS_REST_TOKEN")
            .map_err(|_| "UPSTASH_REDIS_REST_TOKEN not set".to_string())?;

        Ok(Self {
            client: Client::new(),
            url,
            token,
        })
    }

    pub async fn set_json<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_secs: u64,
    ) -> Result<(), String> {
        let json = serde_json::to_string(value).map_err(|e| e.to_string())?;

        let response = self
            .client
            .post(format!("{}/setex/{}/{}", self.url, key, ttl_secs))
            .header("Authorization", format!("Bearer {}", self.token))
            .body(json)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Failed to set key: {}", response.status()))
        }
    }

    pub async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, String> {
        let response = self
            .client
            .get(format!("{}/get/{}", self.url, key))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

        if result["result"].is_null() {
            return Ok(None);
        }

        let data_str = result["result"]
            .as_str()
            .ok_or("Invalid response format")?;
        let data: T = serde_json::from_str(data_str).map_err(|e| e.to_string())?;

        Ok(Some(data))
    }

    pub async fn delete(&self, key: &str) -> Result<(), String> {
        self.client
            .post(format!("{}/del/{}", self.url, key))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Store update metadata (no TTL, persistent)
    /// This is used by update upload scripts to store metadata in Upstash Redis
    #[allow(dead_code)]
    pub async fn set_update_metadata<T: Serialize>(
        &self,
        value: &T,
    ) -> Result<(), String> {
        let json = serde_json::to_string(value).map_err(|e| e.to_string())?;

        let response = self
            .client
            .post(format!("{}/set/update:latest", self.url))
            .header("Authorization", format!("Bearer {}", self.token))
            .body(json)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Failed to set update metadata: {}", response.status()))
        }
    }

    /// Get update metadata
    pub async fn get_update_metadata<T: DeserializeOwned>(&self) -> Result<Option<T>, String> {
        self.get_json("update:latest").await
    }
}
