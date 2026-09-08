//! Beem Africa SMS client — sends SMS via https://docs.beem.africa/

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BeemError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("not configured — set BEEM_API_KEY and BEEM_SECRET_KEY")]
    NotConfigured,
    #[error("api error: {0}")]
    Api(String),
}

#[derive(Clone)]
pub struct BeemAfrica {
    api_key: String,
    secret_key: String,
    sender_id: String,
    url: String,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct SmsRequest<'a> {
    source_addr: &'a str,
    encoding: &'a str,
    message: &'a str,
    recipients: Vec<Recipient<'a>>,
}

#[derive(Debug, Serialize)]
struct Recipient<'a> {
    recipient_id: &'a str,
    dest_addr: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct SmsResponse {
    #[serde(rename = "request_id")]
    pub request_id: Option<String>,
    #[serde(rename = "success_count")]
    pub success_count: Option<u32>,
    #[serde(rename = "failed_count")]
    pub failed_count: Option<u32>,
    #[serde(rename = "pending_count")]
    pub pending_count: Option<u32>,
}

impl BeemAfrica {
    pub fn new() -> Self {
        Self {
            api_key: std::env::var("BEEM_API_KEY").unwrap_or_default(),
            secret_key: std::env::var("BEEM_SECRET_KEY").unwrap_or_default(),
            sender_id: std::env::var("BEEM_SENDER_ID").unwrap_or_else(|_| "GW".into()),
            url: std::env::var("BEEM_SMS_URL")
                .unwrap_or_else(|_| "https://apisms.beem.africa/v1/send".into()),
            client: reqwest::Client::new(),
        }
    }

    pub fn configured(&self) -> bool {
        !self.api_key.is_empty() && !self.secret_key.is_empty()
    }

    fn auth_header(&self) -> String {
        let creds = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            format!("{}:{}", self.api_key, self.secret_key),
        );
        format!("Basic {}", creds)
    }

    pub async fn send(&self, phone_msisdn: &str, message: &str) -> Result<SmsResponse, BeemError> {
        if !self.configured() {
            tracing::warn!("Beem Africa not configured — SMS to {} skipped", phone_msisdn);
            return Err(BeemError::NotConfigured);
        }

        let payload = SmsRequest {
            source_addr: &self.sender_id,
            encoding: "0",
            message,
            recipients: vec![Recipient {
                recipient_id: "1",
                dest_addr: phone_msisdn.trim_start_matches('+'),
            }],
        };

        tracing::debug!("Beem SMS to {}: {}", phone_msisdn, message);

        let resp = self
            .client
            .post(&self.url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            tracing::error!("Beem API error {}: {}", status, body);
            return Err(BeemError::Api(format!("{}: {}", status, body)));
        }

        let result: SmsResponse = serde_json::from_str(&body)
            .map_err(|e| BeemError::Api(format!("failed to parse response: {} — {}", e, body)))?;

        tracing::info!(
            "Beem SMS sent to {} — request_id={:?}",
            phone_msisdn,
            result.request_id
        );

        Ok(result)
    }
}

impl Default for BeemAfrica {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_not_configured() {
        // Force empty env
        std::env::remove_var("BEEM_API_KEY");
        std::env::remove_var("BEEM_SECRET_KEY");
        let beem = BeemAfrica::new();
        assert!(!beem.configured());
    }
}
