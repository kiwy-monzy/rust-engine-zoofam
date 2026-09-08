use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};

use auth::Claims;
use models::{CreateRole, UpdateRole};

use crate::json::ValidatedJson;
use crate::middleware::ApiResult;
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/roles",
    tag = "roles",
    responses(
        (status = 200, description = "List of roles"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<serde_json::Value>> {
    claims.require("roles", "read")?;
    let roles = controller::roles::list(&state.pool)?;
    Ok(Json(serde_json::json!({ "roles": roles })))
}

#[utoipa::path(
    get,
    path = "/api/v1/roles/{id}",
    tag = "roles",
    params(
        ("id" = i32, Path, description = "Role ID")
    ),
    responses(
        (status = 200, description = "Role details"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Role not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
) -> ApiResult<Json<serde_json::Value>> {
    claims.require("roles", "read")?;
    let role = controller::roles::get(&state.pool, id)?;
    Ok(Json(serde_json::json!({ "role": role })))
}

#[utoipa::path(
    post,
    path = "/api/v1/roles",
    tag = "roles",
    request_body = CreateRole,
    responses(
        (status = 201, description = "Role created"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    ValidatedJson(input): ValidatedJson<CreateRole>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    claims.require("roles", "write")?;
    let role = controller::roles::create(&state.pool, input)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "role": role })),
    ))
}

#[utoipa::path(
    patch,
    path = "/api/v1/roles/{id}",
    tag = "roles",
    params(
        ("id" = i32, Path, description = "Role ID")
    ),
    request_body = UpdateRole,
    responses(
        (status = 200, description = "Role updated"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Role not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
    ValidatedJson(input): ValidatedJson<UpdateRole>,
) -> ApiResult<Json<serde_json::Value>> {
    claims.require("roles", "write")?;
    let role = controller::roles::update(&state.pool, id, input)?;
    Ok(Json(serde_json::json!({ "role": role })))
}

#[utoipa::path(
    delete,
    path = "/api/v1/roles/{id}",
    tag = "roles",
    params(
        ("id" = i32, Path, description = "Role ID")
    ),
    responses(
        (status = 204, description = "Role deleted"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Role not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
) -> ApiResult<StatusCode> {
    claims.require("roles", "write")?;
    controller::roles::delete(&state.pool, id)?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/roles/{id}/permissions/{permission_id}",
    tag = "roles",
    params(
        ("id" = i32, Path, description = "Role ID"),
        ("permission_id" = i32, Path, description = "Permission ID")
    ),
    responses(
        (status = 204, description = "Permission granted"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Role or permission not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn grant(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((id, permission_id)): Path<(i32, i32)>,
) -> ApiResult<StatusCode> {
    claims.require("roles", "write")?;
    controller::roles::grant(&state.pool, id, permission_id)?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/v1/roles/{id}/permissions/{permission_id}",
    tag = "roles",
    params(
        ("id" = i32, Path, description = "Role ID"),
        ("permission_id" = i32, Path, description = "Permission ID")
    ),
    responses(
        (status = 204, description = "Permission revoked"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "Role or permission not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn revoke(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((id, permission_id)): Path<(i32, i32)>,
) -> ApiResult<StatusCode> {
    claims.require("roles", "write")?;
    controller::roles::revoke(&state.pool, id, permission_id)?;
    Ok(StatusCode::NO_CONTENT)
}
