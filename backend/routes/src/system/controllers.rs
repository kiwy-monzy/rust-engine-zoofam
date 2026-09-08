//! System settings, releases, and support ticket handlers

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde_json::{json, Value};
use utoipa::ToSchema;

use auth::Claims;
use controller::system::SystemSettings;
use models::CreateTicket;

use crate::json::ValidatedJson;
use crate::middleware::{ApiError, ApiResult};
use crate::state::AppState;

// ------------------------------------------------------------------ health --

#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy")
    )
)]
pub async fn health() -> Json<Value> {
    Json(json!({ "ok": true }))
}

// ------------------------------------------------------------------ system --

#[derive(Debug, serde::Serialize)]
struct SystemResponse {
    app_name: String,
    maintenance_mode: String,
    registration_enabled: String,
    version: String,
}

impl From<SystemSettings> for SystemResponse {
    fn from(s: SystemSettings) -> Self {
        Self {
            app_name: s.app_name,
            maintenance_mode: s.maintenance_mode,
            registration_enabled: s.registration_enabled,
            version: s.version,
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/system",
    tag = "system",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Current system settings"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    )
)]
pub async fn get_system(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    let system = controller::system::get(&state.pool)?;
    Ok(Json(json!({ "system": SystemResponse::from(system) })))
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateSystemRequest {
    pub app_name: Option<String>,
    pub maintenance_mode: Option<String>,
    pub registration_enabled: Option<String>,
    pub version: Option<String>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/system",
    tag = "system",
    request_body = UpdateSystemRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "System settings updated"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    )
)]
pub async fn update_system(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    ValidatedJson(input): ValidatedJson<UpdateSystemRequest>,
) -> ApiResult<Json<Value>> {
    claims.require("system", "write")?;

    if let Some(ref name) = input.app_name {
        let name = name.trim();
        if name.is_empty() {
            return Err(ApiError::new(StatusCode::BAD_REQUEST, "app_name cannot be empty"));
        }
        if name.chars().count() > 120 {
            return Err(ApiError::new(StatusCode::BAD_REQUEST, "app_name must be 1-120 characters"));
        }
        controller::system::set_value(&state.pool, "app_name", name)?;
    }

    if let Some(ref mode) = input.maintenance_mode {
        let mode = mode.trim().to_lowercase();
        if mode != "true" && mode != "false" {
            return Err(ApiError::new(StatusCode::BAD_REQUEST, "maintenance_mode must be true or false"));
        }
        controller::system::set_value(&state.pool, "maintenance_mode", &mode)?;
    }

    if let Some(ref reg) = input.registration_enabled {
        let reg = reg.trim().to_lowercase();
        if reg != "true" && reg != "false" {
            return Err(ApiError::new(StatusCode::BAD_REQUEST, "registration_enabled must be true or false"));
        }
        controller::system::set_value(&state.pool, "registration_enabled", &reg)?;
    }

    if let Some(ref version) = input.version {
        let version = version.trim();
        if version.is_empty() || version.chars().count() > 64 {
            return Err(ApiError::new(StatusCode::BAD_REQUEST, "version must be 1-64 characters"));
        }
        controller::system::set_value(&state.pool, "version", version)?;
    }

    let system = controller::system::get(&state.pool)?;
    Ok(Json(json!({ "system": SystemResponse::from(system) })))
}

// ----------------------------------------------------------------- version --

#[utoipa::path(
    get,
    path = "/api/v1/system/version",
    tag = "system",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Application name and version"),
        (status = 401, description = "Not authenticated")
    )
)]
pub async fn get_version(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    let system = controller::system::get(&state.pool)?;
    Ok(Json(json!({
        "version": system.version,
        "name": system.app_name,
    })))
}

// ----------------------------------------------------------------- releases --

#[utoipa::path(
    get,
    path = "/api/v1/system/releases",
    tag = "system",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of releases"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    )
)]
pub async fn list_releases(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> ApiResult<Json<Value>> {
    claims.require("releases", "read")?;
    let releases = if let Some(platform) = params.get("platform") {
        controller::release::list_by_platform(&state.pool, platform)?
    } else {
        controller::release::list(&state.pool)?
    };
    Ok(Json(json!({ "releases": releases })))
}

#[utoipa::path(
    get,
    path = "/api/v1/system/releases/latest",
    tag = "system",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Latest release for the requested platform"),
        (status = 401, description = "Not authenticated")
    )
)]
pub async fn latest_release(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> ApiResult<Json<Value>> {
    let platform = params
        .get("platform")
        .map(|s| s.as_str())
        .unwrap_or("win64");
    let release = controller::release::latest(&state.pool, platform)?;
    match release {
        Some(r) => Ok(Json(json!({
            "version": r.version,
            "platform": r.platform,
            "filename": r.filename,
            "sha256": r.sha256,
            "changelog": r.changelog,
            "download_url": format!("/api/v1/system/releases/{}/download", r.id),
        }))),
        None => Ok(Json(json!({ "version": null }))),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/system/releases/check/{version}",
    tag = "system",
    params(("version" = String, Path,)),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Version check result"),
        (status = 401, description = "Not authenticated")
    )
)]
pub async fn check_version(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(current): Path<String>,
) -> ApiResult<Json<Value>> {
    let current_version = current.trim();
    // Check all platforms for a newer version
    let releases = controller::release::list(&state.pool)?;
    let mut latest: Option<&models::Release> = None;
    for r in &releases {
        if latest.map_or(true, |l: &models::Release| r.version > l.version) {
            latest = Some(r);
        }
    }
    match latest {
        Some(r) if r.version.as_str() > current_version => Ok(Json(json!({
            "current": current_version,
            "latest": r.version,
            "update_available": true,
            "download_url": format!("/api/v1/system/releases/{}/download", r.id),
            "sha256": r.sha256,
            "changelog": r.changelog,
            "platform": r.platform,
        }))),
        _ => Ok(Json(json!({
            "current": current_version,
            "latest": latest.map(|r| &r.version),
            "update_available": false,
        }))),
    }
}

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct UploadReleaseRequest {
    pub version: String,
    pub platform: String,
    #[serde(default)]
    pub changelog: Option<String>,
    pub file: Vec<u8>,
}

#[utoipa::path(
    post,
    path = "/api/v1/system/releases",
    tag = "system",
    request_body(
        content = UploadReleaseRequest,
        content_type = "multipart/form-data",
        description = "Release upload as multipart/form-data with `version`, `platform`, `changelog` and a `file` field"
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Release created"),
        (status = 400, description = "Invalid upload payload"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    )
)]
pub async fn upload_release(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    mut multipart: axum::extract::Multipart,
) -> ApiResult<Json<Value>> {
    claims.require("releases", "write")?;

    let mut version = String::new();
    let mut platform = String::new();
    let mut changelog = String::new();
    let mut file_data: Option<Vec<u8>> = None;
    let mut filename = String::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, &format!("multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "version" => {
                version = field.text().await.unwrap_or_default();
            }
            "platform" => {
                platform = field.text().await.unwrap_or_default();
            }
            "changelog" => {
                changelog = field.text().await.unwrap_or_default();
            }
            "file" => {
                filename = field.file_name().unwrap_or("unknown").to_string();
                let data = field.bytes().await.map_err(|e| {
                    ApiError::new(StatusCode::BAD_REQUEST, &format!("file read error: {e}"))
                })?;
                file_data = Some(data.to_vec());
            }
            _ => {}
        }
    }

    if version.is_empty() || platform.is_empty() {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "version and platform are required",
        ));
    }
    let file_bytes =
        file_data.ok_or_else(|| ApiError::new(StatusCode::BAD_REQUEST, "file is required"))?;

    use sha2::{Digest, Sha256};
    let sha256 = {
        let mut hasher = Sha256::new();
        hasher.update(&file_bytes);
        format!("{:x}", hasher.finalize())
    };

    let file_size = file_bytes.len() as i32;

    // Save file to storage/releases/{platform}/
    let platform_dir = std::path::PathBuf::from("storage")
        .join("releases")
        .join(&platform);
    std::fs::create_dir_all(&platform_dir).map_err(|e| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to create dir: {e}"),
        )
    })?;
    let file_path = platform_dir.join(&filename);
    std::fs::write(&file_path, &file_bytes).map_err(|e| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("failed to write file: {e}"),
        )
    })?;

    let new_release = models::NewRelease {
        version,
        platform,
        filename,
        file_size: Some(file_size),
        sha256: Some(sha256),
        changelog: if changelog.is_empty() {
            None
        } else {
            Some(changelog)
        },
        created_by: None,
    };
    let release = controller::release::create(&state.pool, new_release)?;
    Ok(Json(json!({ "release": release })))
}

#[utoipa::path(
    delete,
    path = "/api/v1/system/releases/{id}",
    tag = "system",
    params(("id" = i32, Path,)),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Release deleted"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Release not found")
    )
)]
pub async fn delete_release(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
) -> ApiResult<StatusCode> {
    claims.require("releases", "write")?;
    // Delete file from disk
    if let Ok(release) = controller::release::get(&state.pool, id) {
        let path = std::path::PathBuf::from("storage")
            .join("releases")
            .join(&release.platform)
            .join(&release.filename);
        let _ = std::fs::remove_file(path);
    }
    controller::release::delete(&state.pool, id)?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/system/releases/{id}/download",
    tag = "system",
    params(("id" = i32, Path,)),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Release binary file", content_type = "application/octet-stream", body = Vec<u8>),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "Release or file not found")
    )
)]
pub async fn download_release(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    use axum::http::header;
    let release = controller::release::get(&state.pool, id)?;
    let path = std::path::PathBuf::from("storage")
        .join("releases")
        .join(&release.platform)
        .join(&release.filename);
    if !path.exists() {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "file not found on disk",
        ));
    }
    controller::release::increment_download(&state.pool, id)?;
    let data = std::fs::read(&path).map_err(|e| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("read error: {e}"),
        )
    })?;
    let disposition = format!("attachment; filename=\"{}\"", release.filename);
    Ok((
        [
            (header::CONTENT_TYPE, "application/octet-stream".to_string()),
            (header::CONTENT_DISPOSITION, disposition),
        ],
        data,
    ))
}

// ----------------------------------------------------------------- support --

#[utoipa::path(
    get,
    path = "/api/v1/support",
    tag = "support",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of support tickets"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    )
)]
pub async fn list_tickets(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    claims.require("support", "read")?;
    let tickets = controller::support::list(&state.pool, &claims.sub)?;
    Ok(Json(json!({ "tickets": tickets })))
}

#[utoipa::path(
    post,
    path = "/api/v1/support",
    tag = "support",
    request_body = CreateTicket,
    security(("bearer_auth" = [])),
    responses(
        (status = 201, description = "Ticket created"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    )
)]
pub async fn create_ticket(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    ValidatedJson(input): ValidatedJson<CreateTicket>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    claims.require("support", "write")?;
    let ticket = controller::support::create(&state.pool, &claims.sub, input)?;
    Ok((StatusCode::CREATED, Json(json!({ "ticket": ticket }))))
}

#[utoipa::path(
    post,
    path = "/api/v1/support/{id}/close",
    tag = "support",
    params(("id" = String, Path,)),
    security(("bearer_auth" = [])),
    responses(
        (status = 204, description = "Ticket closed"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Ticket not found")
    )
)]
pub async fn close_ticket(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    claims.require("support", "write")?;
    let admin = claims.has_role("admin");
    controller::support::close(&state.pool, &claims.sub, admin, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/support/users",
    tag = "support",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of ticket recipients"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    )
)]
pub async fn ticket_recipients(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    claims.require("support", "write")?;
    let users = controller::support::recipients(&state.pool, &claims.sub)?
        .into_iter()
        .map(|(id, name)| json!({ "id": id, "name": name }))
        .collect::<Vec<_>>();
    Ok(Json(json!({ "users": users })))
}
