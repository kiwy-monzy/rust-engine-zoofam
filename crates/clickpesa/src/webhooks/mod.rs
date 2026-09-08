use serde::{Deserialize, Serialize};
use crate::error::ClickPesaError;

#[derive(Debug, Clone)]
pub struct Webhooks;

impl Webhooks {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    pub id: String,
    pub event_type: String,
    pub event_version: String,
    pub create_time: String,
    pub resource_type: String,
    pub resource: serde_json::Value,
    pub summary: String,
}

pub fn verify_webhook_signature(
    payload: &[u8],
    headers: &WebhookHeaders,
    webhook_secret: &str,
) -> Result<bool, ClickPesaError> {
    let signature = headers
        .clickpesa_signature
        .as_ref()
        .or_else(|| headers.x_clickpesa_signature.as_ref())
        .ok_or_else(|| ClickPesaError::WebhookVerification("Missing signature header".to_string()))?;

    let timestamp = headers
        .clickpesa_transmission_time
        .as_ref()
        .or_else(|| headers.x_clickpesa_transmission_time.as_ref())
        .ok_or_else(|| ClickPesaError::WebhookVerification("Missing timestamp header".to_string()))?;

    let expected_signature = compute_signature(payload, timestamp, webhook_secret)?;
    
    Ok(signature == &expected_signature)
}

fn compute_signature(payload: &[u8], timestamp: &str, secret: &str) -> Result<String, ClickPesaError> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    
    type HmacSha256 = Hmac<Sha256>;
    
    let message = format!("{}{}", timestamp, String::from_utf8_lossy(payload));
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| ClickPesaError::WebhookVerification(e.to_string()))?;
    mac.update(message.as_bytes());
    let result = mac.finalize().into_bytes();
    Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &result))
}

#[derive(Debug, Clone)]
pub struct WebhookHeaders {
    pub clickpesa_signature: Option<String>,
    pub clickpesa_transmission_time: Option<String>,
    pub x_clickpesa_signature: Option<String>,
    pub x_clickpesa_transmission_time: Option<String>,
}

impl WebhookHeaders {
    pub fn from_headers(headers: &reqwest::header::HeaderMap) -> Self {
        Self {
            clickpesa_signature: headers.get("clickpesa-signature").and_then(|h| h.to_str().ok()).map(String::from),
            clickpesa_transmission_time: headers.get("clickpesa-transmission-time").and_then(|h| h.to_str().ok()).map(String::from),
            x_clickpesa_signature: headers.get("x-clickpesa-signature").and_then(|h| h.to_str().ok()).map(String::from),
            x_clickpesa_transmission_time: headers.get("x-clickpesa-transmission-time").and_then(|h| h.to_str().ok()).map(String::from),
        }
    }
}
