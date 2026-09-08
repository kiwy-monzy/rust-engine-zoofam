use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};

use auth::Claims;
use models::{NewPermission, UpdatePermission};

use crate::json::ValidatedJson;
use crate::middleware::ApiResult;
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/permissions",
    tag = "permissions",
    responses(
        (status = 200, description = "List of permissions"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<serde_json::Value>> {
    claims.require("permissions", "read")?;
    let permissions = controller::permissions::list(&state.pool)?;
    Ok(Json(serde_json::json!({ "permissions": permissions })))
}

#[utoipa::path(
    post,
    path = "/api/v1/permissions",
    tag = "permissions",
    request_body = NewPermission,
    responses(
        (status = 201, description = "Permission created"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    ValidatedJson(input): ValidatedJson<NewPermission>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    claims.require("permissions", "write")?;
    let permission = controller::permissions::create(&state.pool, input)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "permission": permission })),
    ))
}

#[utoipa::path(
    patch,
    path = "/api/v1/permissions/{id}",
    tag = "permissions",
    params(
        ("id" = i32, Path, description = "Permission ID")
    ),
    request_body = UpdatePermission,
    responses(
        (status = 200, description = "Permission updated"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Permission not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
    ValidatedJson(input): ValidatedJson<UpdatePermission>,
) -> ApiResult<Json<serde_json::Value>> {
    claims.require("permissions", "write")?;
    let permission = controller::permissions::update(&state.pool, id, input)?;
    Ok(Json(serde_json::json!({ "permission": permission })))
}

#[utoipa::path(
    delete,
    path = "/api/v1/permissions/{id}",
    tag = "permissions",
    params(
        ("id" = i32, Path, description = "Permission ID")
    ),
    responses(
        (status = 204, description = "Permission deleted"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Permission not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
) -> ApiResult<StatusCode> {
    claims.require("permissions", "write")?;
    controller::permissions::delete(&state.pool, id)?;
    Ok(StatusCode::NO_CONTENT)
}
