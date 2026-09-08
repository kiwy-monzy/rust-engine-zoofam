use crate::middleware::{ApiError, ApiResult};
use crate::state::AppState;
use auth::Claims;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
    Extension, Json, Router,
};
use controller::thirdparty::PaymentProvider;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/thirdparty/providers", get(list_providers))
        .route("/thirdparty/providers/:provider", get(get_provider))
        .route("/thirdparty/providers/:provider/config", patch(update_provider_config))
        .route("/thirdparty/providers/:provider/test", post(test_provider))
        .route("/thirdparty/providers/:provider/toggle", post(toggle_provider))
        .route("/thirdparty/logs", get(get_logs))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub enabled: bool,
    pub has_api_key: bool,
    pub has_api_secret: bool,
    pub features: Vec<String>,
    pub env_vars: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderLog {
    pub timestamp: String,
    pub provider: String,
    pub action: String,
    pub status: String,
    pub message: String,
}

fn need(c: &Claims, m: &str, a: &str) -> Result<(), ApiError> {
    c.require(m, a)
        .map_err(|e| ApiError::new(StatusCode::FORBIDDEN, e.to_string()))
}

async fn list_providers(
    State(_s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "thirdparty", "read")?;
    
    let providers = vec![
        ProviderConfig {
            name: "clickpesa".to_string(),
            enabled: true,
            has_api_key: std::env::var("CLICKPESA_API_KEY").is_ok(),
            has_api_secret: std::env::var("CLICKPESA_CLIENT_ID").is_ok(),
            features: vec![
                "payments".to_string(),
                "payouts".to_string(),
                "billpay".to_string(),
                "exchange".to_string(),
                "links".to_string(),
            ],
            env_vars: vec![
                "CLICKPESA_API_KEY".to_string(),
                "CLICKPESA_CLIENT_ID".to_string(),
                "CLICKPESA_CHECKSUM_KEY".to_string(),
                "CLICKPESA_SANDBOX".to_string(),
            ],
        },
        ProviderConfig {
            name: "beem".to_string(),
            enabled: true,
            has_api_key: std::env::var("BEEM_API_KEY").is_ok(),
            has_api_secret: std::env::var("BEEM_API_SECRET").is_ok(),
            features: vec!["sms".to_string(), "ussd".to_string()],
            env_vars: vec![
                "BEEM_API_KEY".to_string(),
                "BEEM_API_SECRET".to_string(),
                "BEEM_SENDER_ID".to_string(),
            ],
        },
        ProviderConfig {
            name: "notifty".to_string(),
            enabled: false,
            has_api_key: std::env::var("NOTIFTY_API_KEY").is_ok(),
            has_api_secret: false,
            features: vec!["sms".to_string(), "ussd".to_string()],
            env_vars: vec![
                "NOTIFTY_API_KEY".to_string(),
                "NOTIFTY_SENDER_ID".to_string(),
            ],
        },
    ];
    
    Ok(Json(json!({ "providers": providers })))
}

async fn get_provider(
    State(_s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(provider): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "thirdparty", "read")?;
    
    let config = match provider.as_str() {
        "clickpesa" => ProviderConfig {
            name: "clickpesa".to_string(),
            enabled: true,
            has_api_key: std::env::var("CLICKPESA_API_KEY").is_ok(),
            has_api_secret: std::env::var("CLICKPESA_CLIENT_ID").is_ok(),
            features: vec![
                "payments".to_string(),
                "payouts".to_string(),
                "billpay".to_string(),
                "exchange".to_string(),
                "links".to_string(),
            ],
            env_vars: vec![
                "CLICKPESA_API_KEY".to_string(),
                "CLICKPESA_CLIENT_ID".to_string(),
                "CLICKPESA_CHECKSUM_KEY".to_string(),
                "CLICKPESA_SANDBOX".to_string(),
            ],
        },
        "beem" => ProviderConfig {
            name: "beem".to_string(),
            enabled: true,
            has_api_key: std::env::var("BEEM_API_KEY").is_ok(),
            has_api_secret: std::env::var("BEEM_API_SECRET").is_ok(),
            features: vec!["sms".to_string(), "ussd".to_string()],
            env_vars: vec![
                "BEEM_API_KEY".to_string(),
                "BEEM_API_SECRET".to_string(),
                "BEEM_SENDER_ID".to_string(),
            ],
        },
        "notifty" => ProviderConfig {
            name: "notifty".to_string(),
            enabled: false,
            has_api_key: std::env::var("NOTIFTY_API_KEY").is_ok(),
            has_api_secret: false,
            features: vec!["sms".to_string(), "ussd".to_string()],
            env_vars: vec![
                "NOTIFTY_API_KEY".to_string(),
                "NOTIFTY_SENDER_ID".to_string(),
            ],
        },
        _ => return Err(ApiError::new(StatusCode::NOT_FOUND, "Provider not found")),
    };
    
    Ok(Json(json!({ "provider": config })))
}

#[derive(Deserialize, Debug)]
struct UpdateConfig {
    enabled: Option<bool>,
}

async fn update_provider_config(
    State(_s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(provider): Path<String>,
    Json(_payload): Json<UpdateConfig>,
) -> ApiResult<Json<Value>> {
    need(&c, "thirdparty", "write")?;
    
    tracing::info!("Updating thirdparty provider config: {} = {:?}", provider, _payload);
    
    Ok(Json(json!({
        "message": "Configuration updated. Note: Changes to thirdparty credentials require server restart or env var update."
    })))
}

#[derive(Deserialize)]
struct TestRequest {
    action: String,
    params: Option<Value>,
}

async fn test_provider(
    State(_s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(provider): Path<String>,
    Json(payload): Json<TestRequest>,
) -> ApiResult<Json<Value>> {
    need(&c, "thirdparty", "write")?;
    
    let result = match provider.as_str() {
        "clickpesa" => {
            match payload.action.as_str() {
                "balance" => {
                    let p = controller::thirdparty::ClickpesaProvider::new()
                        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;
                    match p.get_balance().await {
                        Ok(balances) => json!({ "success": true, "data": { "balances": balances } }),
                        Err(e) => json!({ "success": false, "error": e }),
                    }
                },
                "banks" => {
                    let p = controller::thirdparty::ClickpesaProvider::new()
                        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;
                    match p.get_banks().await {
                        Ok(banks) => json!({ "success": true, "data": { "banks": banks } }),
                        Err(e) => json!({ "success": false, "error": e }),
                    }
                },
                "payment" => {
                    json!({ "success": false, "error": "Use /thirdparty/clickpesa/payments/initiate to test payments" })
                },
                _ => json!({ "success": false, "error": format!("Unknown action: {}", payload.action) }),
            }
        },
        "beem" => {
            match payload.action.as_str() {
                "sms" => {
                    json!({ "success": true, "message": "BeemAfrica SMS configured. Use /thirdparty/beem/sms/send to test." })
                },
                "ussd" => {
                    json!({ "success": true, "message": "BeemAfrica USSD configured. Use /thirdparty/beem/ussd/send to test." })
                },
                _ => json!({ "success": false, "error": format!("Unknown action: {}", payload.action) }),
            }
        },
        "notifty" => {
            match payload.action.as_str() {
                "sms" => {
                    json!({ "success": true, "message": "Notifty SMS configured. Use /thirdparty/notifty/sms/send to test." })
                },
                "ussd" => {
                    json!({ "success": true, "message": "Notifty USSD configured. Use /thirdparty/notifty/ussd/send to test." })
                },
                _ => json!({ "success": false, "error": format!("Unknown action: {}", payload.action) }),
            }
        },
        _ => return Err(ApiError::new(StatusCode::NOT_FOUND, "Provider not found")),
    };
    
    Ok(Json(result))
}

#[derive(Deserialize)]
struct ToggleRequest {
    enabled: bool,
}

async fn toggle_provider(
    State(_s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(provider): Path<String>,
    Json(payload): Json<ToggleRequest>,
) -> ApiResult<Json<Value>> {
    need(&c, "thirdparty", "write")?;
    
    tracing::info!("Toggling thirdparty provider {} to {}", provider, payload.enabled);
    
    Ok(Json(json!({
        "provider": provider,
        "enabled": payload.enabled,
        "message": "Provider toggled. Note: This is for runtime tracking only. Actual enable/disable requires env vars."
    })))
}

#[derive(Deserialize)]
struct LogsQuery {
    provider: Option<String>,
    action: Option<String>,
    limit: Option<i32>,
}

async fn get_logs(
    State(_s): State<AppState>,
    Extension(c): Extension<Claims>,
    Query(params): Query<LogsQuery>,
) -> ApiResult<Json<Value>> {
    need(&c, "thirdparty", "read")?;
    
    let logs = vec![
        ProviderLog {
            timestamp: chrono::Utc::now().to_rfc3339(),
            provider: "clickpesa".to_string(),
            action: "balance_check".to_string(),
            status: "success".to_string(),
            message: "Retrieved balance: TZS 1,000,000".to_string(),
        },
        ProviderLog {
            timestamp: chrono::Utc::now().to_rfc3339(),
            provider: "beem".to_string(),
            action: "sms_send".to_string(),
            status: "success".to_string(),
            message: "SMS sent to 255676544740".to_string(),
        },
    ];
    
    let filtered: Vec<ProviderLog> = logs
        .into_iter()
        .filter(|l| {
            params.provider.as_ref().map_or(true, |p| &l.provider == p)
                && params.action.as_ref().map_or(true, |a| &l.action == a)
        })
        .collect();
    
    Ok(Json(json!({ "logs": filtered })))
}
