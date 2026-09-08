use crate::middleware::ApiError;
use crate::state::AppState;
use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use controller::thirdparty::{NotiftyProvider, NotificationProvider, USSDProvider};
use serde::Deserialize;
use serde_json::{json, Value};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/thirdparty/notifty/sms/send", post(send_sms))
        .route("/thirdparty/notifty/sms/bulk", post(send_bulk_sms))
        .route("/thirdparty/notifty/ussd/send", post(send_ussd))
}

fn provider() -> Result<NotiftyProvider, ApiError> {
    let api_key = std::env::var("NOTIFTY_API_KEY").map_err(|e| {
        ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, format!("NOTIFTY_API_KEY not set: {}", e))
    })?;
    let sender_id = std::env::var("NOTIFTY_SENDER_ID").unwrap_or_else(|_| "KNOWLIA".to_string());
    Ok(NotiftyProvider::new(api_key, sender_id))
}

#[derive(Deserialize)]
struct SendSmsRequest {
    to: String,
    message: String,
}

async fn send_sms(Json(payload): Json<SendSmsRequest>) -> Result<Json<Value>, ApiError> {
    let p = provider()?;
    let message_id = p
        .send_sms(&payload.to, &payload.message)
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(json!({ "message_id": message_id })))
}

#[derive(Deserialize)]
struct BulkSmsRequest {
    recipients: Vec<String>,
    message: String,
}

async fn send_bulk_sms(Json(payload): Json<BulkSmsRequest>) -> Result<Json<Value>, ApiError> {
    let p = provider()?;
    let recipients: Vec<&str> = payload.recipients.iter().map(|s| s.as_str()).collect();
    let message_ids = p
        .send_bulk_sms(recipients, &payload.message)
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(json!({ "message_ids": message_ids })))
}

#[derive(Deserialize)]
struct UssdRequest {
    phone: String,
    message: String,
    callback_url: Option<String>,
}

async fn send_ussd(Json(payload): Json<UssdRequest>) -> Result<Json<Value>, ApiError> {
    let p = provider()?;
    let session_id = p
        .initiate_ussd(&payload.phone, &payload.message, payload.callback_url.as_deref())
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e))?;
    Ok(Json(json!({ "session_id": session_id })))
}
