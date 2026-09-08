use async_trait::async_trait;
use serde::Deserialize;

use super::{NotificationProvider, USSDProvider};

#[derive(Clone)]
pub struct NotiftyProvider {
    api_key: String,
    sender_id: String,
    base_url: String,
}

impl NotiftyProvider {
    pub fn new(api_key: String, sender_id: String) -> Self {
        Self {
            api_key,
            sender_id,
            base_url: "https://app.notifty.co/api/v1".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct NotiftySmsResponse {
    #[serde(rename = "status")]
    status: String,
    #[serde(rename = "message_id")]
    message_id: Option<String>,
    #[serde(rename = "error")]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NotiftyUssdResponse {
    #[serde(rename = "status")]
    status: String,
    #[serde(rename = "session_id")]
    session_id: Option<String>,
    #[serde(rename = "error")]
    error: Option<String>,
}

#[async_trait]
impl NotificationProvider for NotiftyProvider {
    fn name(&self) -> &str {
        "notifty"
    }
    
    async fn send_sms(&self, to: &str, message: &str) -> Result<String, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/sms/send", self.base_url);
        
        let response = client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "sender": self.sender_id,
                "to": to,
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
        
        let data: NotiftySmsResponse = serde_json::from_str(&body)
            .map_err(|e| format!("parse error: {}", e))?;
        
        if data.status == "success" || data.status == "pending" {
            Ok(data.message_id.unwrap_or_else(|| "sent".to_string()))
        } else {
            Err(data.error.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }
    
    async fn send_bulk_sms(&self, recipients: Vec<&str>, message: &str) -> Result<Vec<String>, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/sms/send/bulk", self.base_url);
        
        let response = client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "sender": self.sender_id,
                "to": recipients,
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
        
        let data: Vec<NotiftySmsResponse> = serde_json::from_str(&body)
            .map_err(|e| format!("parse error: {}", e))?;
        
        Ok(data.into_iter().filter_map(|s| s.message_id).collect())
    }
}

#[async_trait]
impl USSDProvider for NotiftyProvider {
    fn name(&self) -> &str {
        "notifty"
    }

    async fn initiate_ussd(
        &self,
        phone: &str,
        message: &str,
        _callback_url: Option<&str>,
    ) -> Result<String, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/ussd/send", self.base_url);
        
        let response = client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "sender": self.sender_id,
                "to": phone,
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
        
        let data: NotiftyUssdResponse = serde_json::from_str(&body)
            .map_err(|e| format!("parse error: {}", e))?;
        
        if data.status == "success" {
            Ok(data.session_id.unwrap_or_else(|| "initiated".to_string()))
        } else {
            Err(data.error.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }
}
