use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use uuid::Uuid;

use auth::Claims;
use models::{CreateUser, UpdateUser};

use crate::json::ValidatedJson;
use crate::middleware::ApiResult;
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "users",
    responses(
        (status = 200, description = "List of users"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(("bearer_auth" = []))
)]
pub async fn list(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<serde_json::Value>> {
    claims.require("users", "read")?;
    let users = controller::users::list(&mut state.pool.get()?)?;
    Ok(Json(serde_json::json!({ "users": users })))
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}
fn default_limit() -> usize {
    20
}

#[utoipa::path(
    get,
    path = "/api/v1/users/search",
    tag = "users",
    params(
        ("q" = String, Query, description = "Search query"),
        ("limit" = Option<usize>, Query, description = "Max results")
    ),
    responses(
        (status = 200, description = "Search results"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(("bearer_auth" = []))
)]
pub async fn search(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    axum::extract::Query(params): axum::extract::Query<SearchQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    claims.require("users", "read")?;
    let users = controller::users::search(&state.pool, &params.q, params.limit)?;
    Ok(Json(serde_json::json!({ "users": users })))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    tag = "users",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User details"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "User not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    claims.require("users", "read")?;
    let user = controller::users::get(&state.pool, &id.to_string())?;
    Ok(Json(serde_json::json!({ "user": user })))
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "users",
    request_body = CreateUser,
    responses(
        (status = 201, description = "User created"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    ValidatedJson(input): ValidatedJson<CreateUser>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    claims.require("users", "write")?;
    let user = controller::users::create(&state.pool, input)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "user": user })),
    ))
}

#[utoipa::path(
    patch,
    path = "/api/v1/users/{id}",
    tag = "users",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    request_body = UpdateUser,
    responses(
        (status = 200, description = "User updated"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "User not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    ValidatedJson(input): ValidatedJson<UpdateUser>,
) -> ApiResult<Json<serde_json::Value>> {
    claims.require("users", "write")?;
    let user = controller::users::update(&state.pool, &id.to_string(), input)?;
    Ok(Json(serde_json::json!({ "user": user })))
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    tag = "users",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "User deleted"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "User not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    claims.require("users", "write")?;
    controller::users::delete(&state.pool, &id.to_string())?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/users/{id}/roles/{role_id}",
    tag = "users",
    params(
        ("id" = String, Path, description = "User ID"),
        ("role_id" = i32, Path, description = "Role ID")
    ),
    responses(
        (status = 204, description = "Role assigned"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "User or role not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn assign_role(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((id, role_id)): Path<(Uuid, i32)>,
) -> ApiResult<StatusCode> {
    claims.require("users", "write")?;
    controller::users::assign_role(&state.pool, &id.to_string(), role_id)?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}/roles/{role_id}",
    tag = "users",
    params(
        ("id" = String, Path, description = "User ID"),
        ("role_id" = i32, Path, description = "Role ID")
    ),
    responses(
        (status = 204, description = "Role revoked"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "User or role not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn revoke_role(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((id, role_id)): Path<(Uuid, i32)>,
) -> ApiResult<StatusCode> {
    claims.require("users", "write")?;
    controller::users::revoke_role(&state.pool, &id.to_string(), role_id)?;
    Ok(StatusCode::NO_CONTENT)
}
