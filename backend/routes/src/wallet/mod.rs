//! Apple Wallet public web-service handlers.
//! These mirror the pass-update web-service spec that iOS devices call
//! when adding a pass or requesting updates. Auth is per-pass (ApplePass token),
//! not per-user JWT.

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use utoipa::ToSchema;

use applewallet::webservice::DeviceRegistration;

use crate::middleware::{ApiError, ApiResult};
use crate::state::AppState;

const PKPASS_MIME: &str = "application/vnd.apple.pkpass";

fn kit(state: &AppState) -> ApiResult<std::sync::Arc<applewallet::PassKit>> {
    state.wallet.clone().ok_or_else(|| {
        ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "wallet is not configured on this server".to_string(),
        )
    })
}

// GET /wallet/pass/:pass_type/:serial_pkpass  — scan-to-add target
#[utoipa::path(
    get,
    path = "/api/v1/wallet/pass/{pass_type}/{serial_pkpass}",
    tag = "wallet",
    params(
        ("pass_type" = String, Path, description = "Pass type identifier"),
        ("serial_pkpass" = String, Path, description = "Serial number (may include .pkpass suffix)")
    ),
    responses(
        (status = 200, description = "PKPASS file", content_type = "application/vnd.apple.pkpass"),
        (status = 503, description = "Wallet not configured")
    )
)]
pub async fn download_pass(
    State(state): State<AppState>,
    Path((pass_type, serial_pkpass)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let serial = serial_pkpass.trim_end_matches(".pkpass");
    let wallet = kit(&state)?;
    let bytes = controller::dmc::wallet_download(&wallet, &pass_type, serial)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::OK, [(header::CONTENT_TYPE, PKPASS_MIME)], bytes))
}

#[derive(Deserialize, ToSchema)]
pub struct RegistrationBody {
    pub auth_token: Option<String>,
    pub device_library_identifier: String,
    pub push_token: String,
}

// POST /wallet/v1/devices/:device_id/registrations/:pass_type/:serial
#[utoipa::path(
    post,
    path = "/api/v1/wallet/v1/devices/{device_id}/registrations/{pass_type}/{serial}",
    tag = "wallet",
    params(
        ("device_id" = String, Path, description = "Device library identifier"),
        ("pass_type" = String, Path, description = "Pass type identifier"),
        ("serial" = String, Path, description = "Pass serial number")
    ),
    request_body = RegistrationBody,
    responses(
        (status = 201, description = "Device registered"),
        (status = 403, description = "Bad auth token"),
        (status = 503, description = "Wallet not configured")
    )
)]
pub async fn register_device(
    State(state): State<AppState>,
    Path((device_id, pass_type, serial)): Path<(String, String, String)>,
    Json(body): Json<RegistrationBody>,
) -> ApiResult<impl IntoResponse> {
    let wallet = kit(&state)?;
    let token = body.auth_token.as_deref().unwrap_or("");
    let ok = controller::dmc::wallet_verify_auth(&wallet, &pass_type, &serial, token);
    if !ok {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "bad auth token".to_string(),
        ));
    }
    let already =
        controller::dmc::wallet_has_registration(&state.pool, &device_id, &pass_type, &serial);
    wallet.store().register_device(DeviceRegistration {
        device_id: body.device_library_identifier,
        pass_type: pass_type.clone(),
        serial: serial.clone(),
        push_token: body.push_token,
    });
    if already {
        Ok((StatusCode::CONFLICT, Json(serde_json::json!({}))))
    } else {
        Ok((StatusCode::CREATED, Json(serde_json::json!({}))))
    }
}

// DELETE /wallet/v1/devices/:device_id/registrations/:pass_type/:serial
#[utoipa::path(
    delete,
    path = "/api/v1/wallet/v1/devices/{device_id}/registrations/{pass_type}/{serial}",
    tag = "wallet",
    params(
        ("device_id" = String, Path, description = "Device library identifier"),
        ("pass_type" = String, Path, description = "Pass type identifier"),
        ("serial" = String, Path, description = "Pass serial number")
    ),
    responses(
        (status = 204, description = "Device unregistered"),
        (status = 503, description = "Wallet not configured")
    )
)]
pub async fn unregister_device(
    State(state): State<AppState>,
    Path((device_id, pass_type, serial)): Path<(String, String, String)>,
    Json(_body): Json<serde::de::IgnoredAny>,
) -> ApiResult<impl IntoResponse> {
    let wallet = kit(&state)?;
    // Apple says DELETE has no body, but iOS 10+ sends the same registration body.
    // We ignore the body and rely on URL params for pass identity.
    wallet
        .store()
        .unregister_device(&device_id, &pass_type, &serial);
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, utoipa::IntoParams)]
pub struct SerialQuery {
    #[serde(rename = "passesUpdatedSince")]
    pub passes_updated_since: Option<String>,
}

// GET /wallet/v1/devices/:device_id/registrations/:pass_type
#[utoipa::path(
    get,
    path = "/api/v1/wallet/v1/devices/{device_id}/registrations/{pass_type}",
    tag = "wallet",
    params(
        ("device_id" = String, Path, description = "Device library identifier"),
        ("pass_type" = String, Path, description = "Pass type identifier"),
        ("passesUpdatedSince" = Option<String>, Query, description = "Filter by update timestamp")
    ),
    responses(
        (status = 200, description = "List of updated serial numbers"),
        (status = 503, description = "Wallet not configured")
    )
)]
pub async fn list_updates(
    State(state): State<AppState>,
    Path((device_id, pass_type)): Path<(String, String)>,
    Query(q): Query<SerialQuery>,
) -> ApiResult<impl IntoResponse> {
    let wallet = kit(&state)?;
    let (serials, last) = wallet.store().serials_updated_since(
        &device_id,
        &pass_type,
        q.passes_updated_since.as_deref(),
    );
    Ok(Json(
        serde_json::json!({ "serialNumbers": serials, "lastUpdated": last }),
    ))
}

// GET /wallet/v1/passes/:pass_type/:serial
#[utoipa::path(
    get,
    path = "/api/v1/wallet/v1/passes/{pass_type}/{serial}",
    tag = "wallet",
    params(
        ("pass_type" = String, Path, description = "Pass type identifier"),
        ("serial" = String, Path, description = "Pass serial number")
    ),
    responses(
        (status = 200, description = "Latest pass data", content_type = "application/vnd.apple.pkpass"),
        (status = 503, description = "Wallet not configured")
    )
)]
pub async fn latest_pass(
    State(state): State<AppState>,
    Path((pass_type, serial)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let wallet = kit(&state)?;
    let bytes = controller::dmc::wallet_download(&wallet, &pass_type, &serial)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::OK, [(header::CONTENT_TYPE, PKPASS_MIME)], bytes))
}

// POST /wallet/v1/log
#[derive(Deserialize, ToSchema)]
pub struct LogBody {
    pub logs: Option<Vec<serde_json::Value>>,
}
#[utoipa::path(
    post,
    path = "/api/v1/wallet/v1/log",
    tag = "wallet",
    request_body = LogBody,
    responses(
        (status = 200, description = "Logs received")
    )
)]
pub async fn log(
    State(_state): State<AppState>,
    Json(body): Json<LogBody>,
) -> ApiResult<impl IntoResponse> {
    if let Some(logs) = &body.logs {
        for entry in logs {
            tracing::info!(wallet_log = ?entry, "wallet log");
        }
    }
    Ok(Json(serde_json::json!({})))
}
