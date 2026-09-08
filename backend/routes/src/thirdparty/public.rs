use crate::state::AppState;
use axum::{
    http::StatusCode,
    routing::{post},
    Json, Router,
};
use serde_json::{json, Value};

pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/thirdparty/clickpesa/webhook", post(webhook))
}

async fn webhook(Json(payload): Json<serde_json::Value>) -> Result<Json<Value>, (StatusCode, String)> {
    tracing::info!("Clickpesa webhook received: {:?}", payload);
    Ok(Json(json!({ "received": true })))
}
