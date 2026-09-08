use reqwest::Client;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::error::ClickPesaError;

const TOKEN_TTL_SECS: u64 = 3300;

#[derive(Debug, Clone)]
pub struct ClickPesaClient {
    pub api_key: String,
    pub client_id: String,
    pub checksum_key: Option<String>,
    pub base_url: String,
    pub http_client: Client,
    access_token: Arc<RwLock<Option<String>>>,
    token_expires_at: Arc<RwLock<Option<Instant>>>,
}

#[derive(Debug, serde::Deserialize)]
struct TokenResponse {
    token: String,
}

impl ClickPesaClient {
    pub fn new(api_key: String, client_id: String, checksum_key: Option<String>, base_url: String) -> Self {
        Self {
            api_key,
            client_id,
            checksum_key,
            base_url,
            http_client: Client::new(),
            access_token: Arc::new(RwLock::new(None)),
            token_expires_at: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get_access_token(&self) -> Result<String, ClickPesaError> {
        {
            let token = self.access_token.read().await;
            let expires_at = self.token_expires_at.read().await;
            if let (Some(ref t), Some(ref exp)) = (&*token, &*expires_at) {
                if Instant::now() < *exp {
                    return Ok(t.clone());
                }
            }
        }

        let url = format!("{}/third-parties/generate-token", self.base_url);
        
        let response = self.http_client
            .post(&url)
            .header("client-id", &self.client_id)
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| ClickPesaError::Network(e.to_string()))?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if status == 401 {
            return Err(ClickPesaError::Authentication("Invalid client-id or api-key".to_string()));
        }
        if status == 403 {
            return Err(ClickPesaError::Authentication("Forbidden".to_string()));
        }
        if !status.is_success() {
            return Err(ClickPesaError::Api(format!("Status: {}, Body: {}", status, body)));
        }

        let token_data: TokenResponse = serde_json::from_str(&body)
            .map_err(|e| ClickPesaError::Parse(format!("Token parse error: {} - body: {}", e, body)))?;

        let mut token = self.access_token.write().await;
        let mut expires_at = self.token_expires_at.write().await;
        *token = Some(token_data.token.clone());
        *expires_at = Some(Instant::now() + Duration::from_secs(TOKEN_TTL_SECS));

        Ok(token_data.token)
    }

    pub async fn request(&self, method: &str, path: &str, body: Option<serde_json::Value>, params: Option<Vec<(String, String)>>) -> Result<serde_json::Value, ClickPesaError> {
        let token = self.get_access_token().await?;
        let url = format!("{}{}", self.base_url, path);
        
        let req_method = reqwest::Method::from_bytes(method.to_uppercase().as_bytes())
            .map_err(|_| ClickPesaError::Api(format!("Invalid HTTP method: {}", method)))?;
        
        let mut req_builder = self.http_client
            .request(req_method, &url)
            .header("Authorization", token)
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(30));

        if let Some(p) = params {
            for (k, v) in p {
                req_builder = req_builder.query(&[(&k, &v)]);
            }
        }

        if let Some(b) = body {
            req_builder = req_builder.json(&b);
        }

        let response = req_builder.send().await.map_err(|e| ClickPesaError::Network(e.to_string()))?;
        
        let status = response.status();
        let body_str = response.text().await.unwrap_or_default();

        if status.as_u16() >= 400 {
            return Err(ClickPesaError::Api(format!("Status: {}, Body: {}", status, body_str)));
        }

        if body_str.is_empty() {
            return Ok(serde_json::json!({"status": "success"}));
        }

        serde_json::from_str(&body_str).map_err(|e| ClickPesaError::Parse(format!("{}: {}", e, body_str)))
    }

    pub async fn get(&self, path: &str, params: Option<Vec<(String, String)>>) -> Result<serde_json::Value, ClickPesaError> {
        self.request("GET", path, None, params).await
    }

    pub async fn post(&self, path: &str, body: serde_json::Value) -> Result<serde_json::Value, ClickPesaError> {
        self.request("POST", path, Some(body), None).await
    }

    pub async fn patch(&self, path: &str, body: serde_json::Value) -> Result<serde_json::Value, ClickPesaError> {
        self.request("PATCH", path, Some(body), None).await
    }

    pub async fn delete(&self, path: &str) -> Result<serde_json::Value, ClickPesaError> {
        self.request("DELETE", path, None, None).await
    }
}