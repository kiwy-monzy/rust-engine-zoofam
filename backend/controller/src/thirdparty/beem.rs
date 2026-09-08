use async_trait::async_trait;
use serde::Deserialize;

use super::{NotificationProvider, USSDProvider};

#[derive(Clone)]
pub struct BeemAfricaProvider {
    api_key: String,
    sender_id: String,
    base_url: String,
}

impl BeemAfricaProvider {
    pub fn new(api_key: String, sender_id: String) -> Self {
        Self {
            api_key,
            sender_id,
            base_url: "https://api.beem.software".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct BeemSmsResponse {
    #[serde(rename = "status")]
    status: String,
    #[serde(rename = "messageId")]
    message_id: Option<String>,
    #[serde(rename = "responseCode")]
    response_code: Option<String>,
    #[serde(rename = "responseText")]
    response_text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BeemUssdResponse {
    #[serde(rename = "status")]
    status: String,
    #[serde(rename = "ussdSessionId")]
    ussd_session_id: Option<String>,
    #[serde(rename = "message")]
    message: Option<String>,
}

#[async_trait]
impl NotificationProvider for BeemAfricaProvider {
    fn name(&self) -> &str {
        "beem-africa"
    }
    
    async fn send_sms(&self, to: &str, message: &str) -> Result<String, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/public/v1/sms/send", self.base_url);
        
        let response = client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "source": self.sender_id,
                "destination": to,
                "message": message,
            }))
            .send()
            .await
            .map_err(|e| format!("network error: {}", e))?;
        
        let status = response.status();
        let body = response.text().await.map_err(|e| format!("read error: {}", e))?;
        
        if !status.is_success() {
            return Err(format!("API error {}: {}", status, body));
        }
        
        let data: BeemSmsResponse = serde_json::from_str(&body)
            .map_err(|e| format!("parse error: {}", e))?;
        
        if data.status == "Success" || data.response_code == Some("000".to_string()) {
            Ok(data.message_id.unwrap_or_default())
        } else {
            Err(data.response_text.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }
    
    async fn send_bulk_sms(&self, recipients: Vec<&str>, message: &str) -> Result<Vec<String>, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/public/v1/sms/sendBulk", self.base_url);
        
        let dests: Vec<_> = recipients.into_iter().map(|r| serde_json::json!({
            "destination": r,
        })).collect();
        
        let response = client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "source": self.sender_id,
                "destinations": dests,
                "message": message,
            }))
            .send()
            .await
            .map_err(|e| format!("network error: {}", e))?;
        
        let status = response.status();
        let body = response.text().await.map_err(|e| format!("read error: {}", e))?;
        
        if !status.is_success() {
            return Err(format!("API error {}: {}", status, body));
        }
        
        let data: Vec<BeemSmsResponse> = serde_json::from_str(&body)
            .map_err(|e| format!("parse error: {}", e))?;
        
        Ok(data.into_iter().filter_map(|s| s.message_id).collect())
    }
}

#[async_trait]
impl USSDProvider for BeemAfricaProvider {
    fn name(&self) -> &str {
        "beem-africa"
    }

    async fn initiate_ussd(
        &self,
        phone: &str,
        message: &str,
        _callback_url: Option<&str>,
    ) -> Result<String, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/public/v1/ussd/send", self.base_url);
        
        let response = client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "source": self.sender_id,
                "destination": phone,
                "message": message,
            }))
            .send()
            .await
            .map_err(|e| format!("network error: {}", e))?;
        
        let status = response.status();
        let body = response.text().await.map_err(|e| format!("read error: {}", e))?;
        
        if !status.is_success() {
            return Err(format!("API error {}: {}", status, body));
        }
        
        let data: BeemUssdResponse = serde_json::from_str(&body)
            .map_err(|e| format!("parse error: {}", e))?;
        
        if data.status == "Success" {
            Ok(data.ussd_session_id.unwrap_or_default())
        } else {
            Err(data.message.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }
}
