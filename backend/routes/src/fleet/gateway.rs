//! Fleet gateway controller — Bolt login, vault, in-memory tile serving.
//! No DB tables are written from here. The poller (in `gateway-fleet`) keeps
//! its live cache in process memory and serves it via `gateway_fleet::tiles`.

use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use chrono::Utc;
use serde::Deserialize;
use utoipa::ToSchema;

use auth::Claims;
use fleet::bolt::service as boltsvc;

use crate::middleware::{ApiError, ApiResult};
use crate::state::AppState;

const DAR_LAT: f64 = -6.83;
const DAR_LNG: f64 = 39.30;
const DAR_TZ: &str = "Africa/Dar_es_Salaam";

fn require(claims: &Claims, module: &str, action: &str) -> Result<(), ApiError> {
    claims
        .require(module, action)
        .map_err(|e| ApiError::new(StatusCode::FORBIDDEN, e.to_string()))
}

// ---------------------------------------------------------------- vault --

#[derive(Deserialize, ToSchema)]
pub struct VaultPutBody {
    service: String,
    #[serde(default = "credential_kind")]
    kind: String,
    name: String,
    #[serde(default)]
    secret: String,
    #[serde(default)]
    meta: serde_json::Value,
}
fn credential_kind() -> String {
    "credential".to_string()
}
fn default_channel() -> String {
    "sms".to_string()
}

#[utoipa::path(
    get,
    path = "/api/v1/fleet/vault",
    tag = "fleet",
    responses(
        (status = 200, description = "List of vault entries"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(("bearer_auth" = []))
)]
pub async fn vault_list(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<serde_json::Value>> {
    require(&claims, "fleet", "admin")?;
    let pool = state.pool.clone();
    let entries = tokio::task::spawn_blocking(move || gateway_fleet::db_ext::vault_list(&pool))
        .await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(
        serde_json::json!({ "success": true, "vault": entries }),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/fleet/vault",
    tag = "fleet",
    request_body = VaultPutBody,
    responses(
        (status = 200, description = "Credential stored"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(("bearer_auth" = []))
)]
pub async fn vault_put(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<VaultPutBody>,
) -> ApiResult<Json<serde_json::Value>> {
    require(&claims, "fleet", "admin")?;
    if input.service.is_empty() || input.name.is_empty() {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "service and name are required",
        ));
    }
    let meta = if input.meta.is_object() {
        input.meta.clone()
    } else {
        serde_json::json!({})
    };
    let pool = state.pool.clone();
    let svc = input.service.clone();
    let name = input.name.clone();
    let secret = input.secret.clone();
    let kind = input.kind.clone();
    let meta2 = meta.clone();
    tokio::task::spawn_blocking(move || {
        gateway_fleet::db_ext::vault_put(&pool, &svc, &kind, &name, &secret, &meta2)
    })
    .await
    .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ---------------------------------------------------------------- bolt vehicles (in-memory) --

#[utoipa::path(
    get,
    path = "/api/v1/fleet/bolt",
    tag = "fleet",
    responses(
        (status = 200, description = "List of Bolt vehicles"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(("bearer_auth" = []))
)]
pub async fn bolt_list(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<serde_json::Value>> {
    require(&claims, "fleet", "read")?;
    let snap = gateway_fleet::load_live();
    let v = snap.as_ref().map(|s| s.bolt.clone()).unwrap_or_default();
    let _ = state; // suppress unused warning
    Ok(Json(serde_json::json!({
        "success": true,
        "count": v.len(),
        "vehicles": v,
        "cached": snap.is_some(),
    })))
}

// ---------------------------------------------------------------- bolt login --

#[derive(Deserialize, ToSchema)]
pub struct BoltStartBody {
    phone: String,
    #[serde(default = "default_channel")]
    channel: String,
    #[serde(default)]
    lat: Option<f64>,
    #[serde(default)]
    lng: Option<f64>,
    #[serde(default)]
    country: String,
    #[serde(default)]
    tz: String,
}
#[derive(Deserialize, ToSchema)]
pub struct BoltConfirmBody {
    otp: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/fleet/bolt/login/start",
    tag = "fleet",
    request_body = BoltStartBody,
    responses(
        (status = 200, description = "OTP code sent"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 502, description = "Bolt service error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn bolt_login_start(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<BoltStartBody>,
) -> ApiResult<Json<serde_json::Value>> {
    require(&claims, "fleet", "admin")?;
    if input.phone.trim().is_empty() {
        return Err(ApiError::new(StatusCode::BAD_REQUEST, "phone is required"));
    }
    let (lat, lng) = (input.lat.unwrap_or(DAR_LAT), input.lng.unwrap_or(DAR_LNG));
    let country = if input.country.is_empty() {
        "TZ".into()
    } else {
        input.country.clone()
    };
    let tz = if input.tz.is_empty() {
        DAR_TZ.into()
    } else {
        input.tz.clone()
    };
    let device = fleet::bolt::Device {
        uuid: uuid::Uuid::new_v4().to_string(),
        lat,
        lng,
        country: country.clone(),
        timezone: tz.clone(),
    };
    match boltsvc::start(&input.phone, &device, &input.channel).await {
        Ok(session) => {
            let secret = serde_json::to_string(&session).unwrap_or_default();
            let meta = serde_json::json!({ "lat": lat, "lng": lng, "country": country, "tz": tz, "phone": input.phone, "device_uuid": device.uuid });
            let pool = state.pool.clone();
            tokio::task::spawn_blocking(move || {
                gateway_fleet::db_ext::vault_put(
                    &pool, "bolt", "session", "pending", &secret, &meta,
                )
            })
            .await
            .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            Ok(Json(
                serde_json::json!({ "success": true, "channel": input.channel, "message": "Code sent — enter it to finish." }),
            ))
        }
        Err(e) => Err(ApiError::new(StatusCode::BAD_GATEWAY, format!("Bolt: {e}"))),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/fleet/bolt/login/confirm",
    tag = "fleet",
    request_body = BoltConfirmBody,
    responses(
        (status = 200, description = "Login confirmed"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 409, description = "No pending login"),
        (status = 502, description = "Bolt service error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn bolt_login_confirm(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<BoltConfirmBody>,
) -> ApiResult<Json<serde_json::Value>> {
    require(&claims, "fleet", "admin")?;
    let pending: Option<(String, serde_json::Value)> = {
        let pool = state.pool.clone();
        tokio::task::spawn_blocking(move || {
            gateway_fleet::db_ext::vault_get(&pool, "bolt", "pending")
        })
        .await
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };
    let (secret, meta) = match pending {
        Some(v) => v,
        None => {
            return Err(ApiError::new(
                StatusCode::CONFLICT,
                "No pending login — start first",
            ))
        }
    };
    let pending_session: fleet::bolt::Session = serde_json::from_str(&secret)
        .map_err(|_| ApiError::new(StatusCode::CONFLICT, "No valid pending login — start again"))?;
    let lat = meta.get("lat").and_then(|v| v.as_f64()).unwrap_or(DAR_LAT);
    let lng = meta.get("lng").and_then(|v| v.as_f64()).unwrap_or(DAR_LNG);
    let country = meta
        .get("country")
        .and_then(|v| v.as_str())
        .unwrap_or("TZ")
        .to_string();
    let tz = meta
        .get("tz")
        .and_then(|v| v.as_str())
        .unwrap_or(DAR_TZ)
        .to_string();
    let device = fleet::bolt::Device {
        uuid: meta
            .get("device_uuid")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        lat,
        lng,
        country,
        timezone: tz,
    };
    match boltsvc::confirm(&pending_session, &input.otp, &device).await {
        Ok(session) => {
            let secret = serde_json::to_string(&session).unwrap_or_default();
            let sess_meta = serde_json::json!({ "phone": session.phone, "logged_in_at": Utc::now().to_rfc3339() });
            let pool = state.pool.clone();
            tokio::task::spawn_blocking(move || {
                gateway_fleet::db_ext::vault_put(
                    &pool, "bolt", "session", "session", &secret, &sess_meta,
                )
            })
            .await
            .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            Ok(Json(
                serde_json::json!({ "success": true, "message": "Bolt session stored." }),
            ))
        }
        Err(e) => Err(ApiError::new(StatusCode::BAD_GATEWAY, format!("Bolt: {e}"))),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/fleet/bolt/status",
    tag = "fleet",
    responses(
        (status = 200, description = "Bolt login status"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(("bearer_auth" = []))
)]
pub async fn bolt_status(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<serde_json::Value>> {
    require(&claims, "fleet", "read")?;
    let pool = state.pool.clone();
    let v = tokio::task::spawn_blocking(move || {
        gateway_fleet::db_ext::vault_get(&pool, "bolt", "session")
    })
    .await
    .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let logged_in = v.is_some();
    let meta = v.map(|(_, m)| m);
    Ok(Json(
        serde_json::json!({ "success": true, "logged_in": logged_in, "meta": meta }),
    ))
}

// ---------------------------------------------------------------- marine cookie --

#[derive(Deserialize, ToSchema)]
pub struct MarineCookieBody {
    #[serde(default)]
    cookie: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/fleet/marine/cookie",
    tag = "fleet",
    responses(
        (status = 200, description = "Marine cookie status"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(("bearer_auth" = []))
)]
pub async fn marine_cookie(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<serde_json::Value>> {
    require(&claims, "fleet", "read")?;
    let pool = state.pool.clone();
    let cookie: Option<String> = tokio::task::spawn_blocking(move || {
        gateway_fleet::db_ext::vault_secret(&pool, "marinetraffic", "cf_cookie")
    })
    .await
    .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    // Also check the live poller cache so the user can see what is actually
    // being used to render marine data on the map.
    let snap = gateway_fleet::load_live();
    let live_marine = snap.as_ref().map(|s| s.marine.len()).unwrap_or(0);
    Ok(Json(serde_json::json!({
        "success": true,
        "configured": cookie.is_some(),
        "cookie": cookie,
        "live_marine_count": live_marine,
    })))
}

#[utoipa::path(
    post,
    path = "/api/v1/fleet/marine/cookie",
    tag = "fleet",
    request_body = MarineCookieBody,
    responses(
        (status = 200, description = "Cookie updated"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(("bearer_auth" = []))
)]
pub async fn marine_cookie_set(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(input): Json<MarineCookieBody>,
) -> ApiResult<Json<serde_json::Value>> {
    require(&claims, "fleet", "admin")?;
    let pool = state.pool.clone();
    let secret = input.cookie.clone();
    let meta = serde_json::json!({ "updated_at": Utc::now().to_rfc3339() });
    tokio::task::spawn_blocking(move || {
        gateway_fleet::db_ext::vault_put(
            &pool,
            "marinetraffic",
            "cookie",
            "cf_cookie",
            &secret,
            &meta,
        )
    })
    .await
    .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ---------------------------------------------------------------- tiles --

/// Serve MVT (default) or GeoJSON (`.json` suffix) tiles straight from the
/// in-memory poller cache. `source` is one of `bolt`, `marine`, `flights`.
#[utoipa::path(
    get,
    path = "/api/v1/fleet/tiles/{source}/{z}/{x}/{y}",
    tag = "fleet",
    params(
        ("source" = String, Path, description = "Tile source (bolt, marine, flights)"),
        ("z" = u32, Path, description = "Zoom level"),
        ("x" = u32, Path, description = "Tile X coordinate"),
        ("y" = String, Path, description = "Tile Y coordinate (append .json for GeoJSON)")
    ),
    responses(
        (status = 200, description = "Tile data (MVT or GeoJSON)"),
        (status = 400, description = "Invalid tile coordinates"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(("bearer_auth" = []))
)]
pub async fn tile_handler(
    Extension(claims): Extension<Claims>,
    Path((source, z, x, y_path)): Path<(String, u32, u32, String)>,
) -> ApiResult<Response> {
    require(&claims, "fleet", "read")?;
    let wants_json = y_path.ends_with(".json");
    let y: u32 = y_path
        .trim_end_matches(".json")
        .parse()
        .map_err(|_| ApiError::new(StatusCode::BAD_REQUEST, format!("invalid tile y: {y_path}")))?;

    if !["bolt", "marine", "flights"].contains(&source.as_str()) {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            format!("unknown source: {source}"),
        ));
    }

    if wants_json {
        let fc = gateway_fleet::tiles::geojson_for_tile(&source, z, x, y)
            .unwrap_or_else(|| serde_json::json!({ "type": "FeatureCollection", "features": [] }));
        return Ok((
            [(header::CONTENT_TYPE, "application/geo+json")],
            fc.to_string(),
        )
            .into_response());
    }

    let bytes = gateway_fleet::tiles::mvt_for_tile(&source, z, x, y)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let etag = format!("\"{}-{}-{}-{}\"", source, z, x, y);
    Ok((
        [
            (
                header::CONTENT_TYPE,
                "application/x-protobuf;type=mapbox-vector",
            ),
            (header::CACHE_CONTROL, "public, max-age=10"),
            (header::ETAG, etag.as_str()),
        ],
        bytes,
    )
        .into_response())
}

// ---------------------------------------------------------------- summary --

#[utoipa::path(
    get,
    path = "/api/v1/fleet/summary",
    tag = "fleet",
    responses(
        (status = 200, description = "Fleet live summary"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(("bearer_auth" = []))
)]
pub async fn summary(Extension(claims): Extension<Claims>) -> ApiResult<Json<serde_json::Value>> {
    require(&claims, "fleet", "read")?;
    if gateway_fleet::load_live().is_none() {
        return Ok(Json(serde_json::json!({
            "success": true,
            "cached": false,
            "counts": { "bolt": 0, "marine": 0, "flights": 0, "total": 0 },
            "sources": [],
            "cells": { "type": "FeatureCollection", "features": [] },
        })));
    }
    let mut value = gateway_fleet::tiles::live_summary().unwrap_or(serde_json::json!({}));
    if let Some(obj) = value.as_object_mut() {
        obj.insert("cached".to_string(), serde_json::Value::Bool(true));
    }
    Ok(Json(value))
}
