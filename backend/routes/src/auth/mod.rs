//! Auth surface: register, login (access + refresh + session row), refresh,
//! logout (single session), logout-all, me, password change, password reset
//! request + confirm, profile, sessions, avatars. All permission checks are
//! enforced server-side; the frontend `RequirePerm` is only a UI hint.

use axum::extract::{ConnectInfo, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::{Extension, Json};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::SocketAddr;
use uuid::Uuid;
use utoipa::ToSchema;

use auth::{
    hash_password, hash_reset_token, verify_password, AccessToken, AuthError, Claims, RefreshToken,
};
use models::{session::Session, profile::UpdateProfile as ModelsUpdateProfile, UserFile, UserWithRoles};

use crate::json::ValidatedJson;
use crate::middleware::{ApiError, ApiResult};
use crate::state::AppState;

const REFRESH_HEADER: &str = "x-refresh-token";
const COOKIE_ACCESS: &str = "gw_access";
const COOKIE_REFRESH: &str = "gw_refresh";

#[derive(Debug, Serialize, ToSchema)]
pub struct PermissionItem {
    pub module: String,
    pub action: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PermissionGroup {
    pub module: String,
    pub actions: Vec<String>,
}

fn group_permissions(perms: &[String]) -> Vec<PermissionGroup> {
    use std::collections::BTreeMap;
    let mut grouped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for p in perms {
        let mut parts = p.splitn(2, ':');
        if let (Some(module), Some(action)) = (parts.next(), parts.next()) {
            grouped
                .entry(module.to_string())
                .or_default()
                .push(action.to_string());
        }
    }
    grouped
        .into_iter()
        .map(|(module, mut actions)| {
            actions.sort_unstable();
            actions.dedup();
            PermissionGroup { module, actions }
        })
        .collect()
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    refresh_expires_in: i64,
    user: UserWithRoles,
    roles: Vec<String>,
    permissions: Vec<PermissionGroup>,
}

fn user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

fn client_ip(headers: &HeaderMap, addr: Option<&SocketAddr>) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
        .or_else(|| addr.map(|a| a.ip().to_string()))
}

fn device_label(ua: Option<&str>) -> Option<String> {
    let ua = ua?;
    // Cheap browser/OS detector. We don't need a full UA parser — the raw
    // user-agent is preserved on the row.
    let browser = if ua.contains("Chrome/") && !ua.contains("Edg/") {
        "Chrome"
    } else if ua.contains("Edg/") {
        "Edge"
    } else if ua.contains("Firefox/") {
        "Firefox"
    } else if ua.contains("Safari/") && !ua.contains("Chrome/") {
        "Safari"
    } else {
        "Browser"
    };
    let os = if ua.contains("Windows") {
        "Windows"
    } else if ua.contains("Mac OS X") || ua.contains("Macintosh") {
        "macOS"
    } else if ua.contains("Android") {
        "Android"
    } else if ua.contains("iPhone") || ua.contains("iPad") {
        "iOS"
    } else if ua.contains("Linux") {
        "Linux"
    } else {
        "Unknown OS"
    };
    Some(format!("{browser} · {os}"))
}

fn write_cookies(headers: &mut HeaderMap, access: &str, refresh: &str, secure: bool) {
    let access_cookie = format!(
        "{}={}; Path=/; HttpOnly; SameSite=Lax{}",
        COOKIE_ACCESS,
        access,
        if secure { "; Secure" } else { "" }
    );
    let refresh_cookie = format!(
        "{}={}; Path=/; HttpOnly; SameSite=Lax{}",
        COOKIE_REFRESH,
        refresh,
        if secure { "; Secure" } else { "" }
    );
    headers.append("set-cookie", access_cookie.parse().unwrap());
    headers.append("set-cookie", refresh_cookie.parse().unwrap());
}

fn clear_cookies(headers: &mut HeaderMap) {
    for c in [
        format!(
            "{}=; Path=/; HttpOnly; Max-Age=0; SameSite=Lax",
            COOKIE_ACCESS
        ),
        format!(
            "{}=; Path=/; HttpOnly; Max-Age=0; SameSite=Lax",
            COOKIE_REFRESH
        ),
    ] {
        headers.append("set-cookie", c.parse().unwrap());
    }
}

fn extract_refresh(headers: &HeaderMap) -> Option<String> {
    if let Some(h) = headers.get(REFRESH_HEADER).and_then(|v| v.to_str().ok()) {
        if !h.is_empty() {
            return Some(h.to_string());
        }
    }
    if let Some(c) = headers.get("cookie").and_then(|v| v.to_str().ok()) {
        for pair in c.split(';') {
            let pair = pair.trim();
            if let Some(rest) = pair.strip_prefix(&format!("{COOKIE_REFRESH}=")) {
                if !rest.is_empty() {
                    return Some(rest.to_string());
                }
            }
        }
    }
    None
}

fn secure_cookies(headers: &HeaderMap) -> bool {
    headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.eq_ignore_ascii_case("https"))
        .unwrap_or(false)
}

fn now_secs() -> i64 {
    Utc::now().timestamp()
}

async fn issue_login(
    state: &AppState,
    user: &models::User,
    user_agent: Option<String>,
    ip: Option<String>,
) -> ApiResult<(LoginResponse, AccessToken, RefreshToken, String, i64, i64)> {
    let roles = controller::users::role_names(&state.pool, &user.id)?;
    let perms = controller::users::permission_keys(&state.pool, &user.id)?;
    let (access, refresh, _access_claims, _refresh_claims, sid) = state
        .jwt
        .issue_pair(
            &user.id,
            &user.email,
            roles.clone(),
            perms.clone(),
            user.token_version,
            None,
        )
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let refresh_hash = hash_reset_token(&refresh.0);
    let expires_at = Utc::now().naive_utc() + state.jwt.refresh_ttl();
    if let Err(e) = controller::sessions::create(
        &state.pool,
        &sid,
        &user.id,
        &refresh_hash,
        device_label(user_agent.as_deref()).as_deref(),
        user_agent.as_deref(),
        ip.as_deref(),
        expires_at,
    ) {
        eprintln!(
            "[auth::login] session create FAILED: user_id={} sid={} err={}",
            &user.id, &sid, e
        );
        return Err(ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("session create failed: {e}"),
        ));
    }
    let user_with_roles = controller::users::get(&state.pool, &user.id)?;
    let access_secs = state.jwt.access_ttl().num_seconds();
    let refresh_secs = state.jwt.refresh_ttl().num_seconds();
    Ok((
        LoginResponse {
            access_token: access.0.clone(),
            refresh_token: refresh.0.clone(),
            expires_in: access_secs,
            refresh_expires_in: refresh_secs,
            user: user_with_roles,
            roles: roles.clone(),
            permissions: group_permissions(&perms),
        },
        access,
        refresh,
        sid,
        access_secs,
        refresh_secs,
    ))
}

// ---------------------------------------------------------- register --

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    tag = "auth",
    request_body = CreateUser,
    responses(
        (status = 201, description = "User registered successfully"),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "Email already exists")
    )
)]
pub async fn register(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    ValidatedJson(input): ValidatedJson<models::CreateUser>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let _ = (addr, &headers);
    let user = controller::users::create(&state.pool, input)?;
    Ok((StatusCode::CREATED, Json(json!({ "user": user }))))
}

// ---------------------------------------------------------- login --

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "auth",
    request_body = Credentials,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    ValidatedJson(input): ValidatedJson<models::Credentials>,
) -> ApiResult<impl IntoResponse> {
    login_inner(state, addr, headers, input).await
}

async fn login_inner(
    state: AppState,
    addr: SocketAddr,
    headers: HeaderMap,
    input: models::Credentials,
) -> ApiResult<impl IntoResponse> {
    let user = controller::users::find_by_email(&state.pool, &input.email)
        .map_err(|_| ApiError::from(AuthError::BadCredentials))?;
    if !user.is_active {
        return Err(ApiError::from(AuthError::Disabled));
    }
    verify_password(&input.password, &user.password_hash)?;

    let ua = user_agent(&headers);
    let ip = client_ip(&headers, Some(&addr));
    let (resp, _access, _refresh, _sid, _access_secs, _refresh_secs) =
        issue_login(&state, &user, ua, ip).await?;

    let mut out_headers = HeaderMap::new();
    write_cookies(
        &mut out_headers,
        &resp.access_token,
        &resp.refresh_token,
        secure_cookies(&headers),
    );

    let body = Json(json!({
        "access_token": resp.access_token,
        "refresh_token": resp.refresh_token,
        "expires_in": resp.expires_in,
        "refresh_expires_in": resp.refresh_expires_in,
        "user": resp.user,
        "roles": resp.roles,
        "permissions": resp.permissions,
    }));
    Ok((StatusCode::OK, out_headers, body))
}

// ---------------------------------------------------------- refresh --

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    tag = "auth",
    responses(
        (status = 200, description = "Tokens refreshed", body = LoginResponse),
        (status = 401, description = "Invalid refresh token")
    )
)]
pub async fn refresh(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    let raw = extract_refresh(&headers).ok_or_else(|| ApiError::from(AuthError::Missing))?;
    let claims = state.jwt.verify_refresh(&raw).map_err(ApiError::from)?;
    let hash = hash_reset_token(&raw);
    let session = controller::sessions::validate_refresh(&state.pool, &claims, &hash)?;

    // Rotate: every successful refresh mints a brand new refresh token,
    // invalidating the old one. If anyone replays the old token, the row's
    // hash no longer matches and validation fails.
    let user = controller::users::find_by_id(&state.pool, &session.user_id)
        .map_err(|_| ApiError::from(AuthError::Invalid))?;
    if !user.is_active {
        return Err(ApiError::from(AuthError::Disabled));
    }
    let roles = controller::users::role_names(&state.pool, &user.id)?;
    let perms_keys = controller::users::permission_keys(&state.pool, &user.id)?;
    let parent = Some(session.id.clone());
    let (access, refresh, _access_claims, _refresh_claims, _new_sid) = state
        .jwt
        .issue_pair(
            &user.id,
            &user.email,
            roles.clone(),
            perms_keys.clone(),
            user.token_version,
            parent,
        )
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let new_hash = hash_reset_token(&refresh.0);
    controller::sessions::rotate_refresh_hash(&state.pool, &session.id, &new_hash)?;
    let ip = client_ip(&headers, Some(&addr));
    let ua = user_agent(&headers);
    controller::sessions::touch(&state.pool, &session.id, ip.as_deref(), ua.as_deref())?;

    let mut out_headers = HeaderMap::new();
    write_cookies(
        &mut out_headers,
        &access.0,
        &refresh.0,
        secure_cookies(&headers),
    );

    let body = Json(json!({
        "access_token": access.0,
        "refresh_token": refresh.0,
        "expires_in": state.jwt.access_ttl().num_seconds(),
        "refresh_expires_in": state.jwt.refresh_ttl().num_seconds(),
        "user": controller::users::get(&state.pool, &user.id)?,
        "roles": roles,
        "permissions": group_permissions(&perms_keys),
    }));
    Ok((StatusCode::OK, out_headers, body))
}

// ---------------------------------------------------------- logout --

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "auth",
    responses(
        (status = 204, description = "Logged out successfully"),
        (status = 401, description = "Not authenticated")
    )
)]
pub async fn logout(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    controller::sessions::revoke(&state.pool, &claims)?;
    let mut out_headers = HeaderMap::new();
    clear_cookies(&mut out_headers);
    let _ = headers;
    Ok((StatusCode::NO_CONTENT, out_headers))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout-all",
    tag = "auth",
    responses(
        (status = 204, description = "All sessions revoked"),
        (status = 401, description = "Not authenticated")
    )
)]
pub async fn logout_all(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<StatusCode> {
    controller::sessions::revoke_all(&state.pool, &claims.sub)?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------- me --

#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    tag = "auth",
    responses(
        (status = 200, description = "Current user profile", body = UserWithRoles),
        (status = 401, description = "Not authenticated")
    )
)]
pub async fn me(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    let u = controller::users::get(&state.pool, &claims.sub)?;
    let perms = controller::users::permission_keys(&state.pool, &claims.sub)?;
    Ok(Json(json!({
        "id": claims.sub,
        "email": claims.email,
        "display_name": u.user.display_name,
        "avatar_url": u.user.avatar_url,
        "first_name": u.user.first_name,
        "middle_name": u.user.middle_name,
        "last_name": u.user.last_name,
        "username": u.user.username,
        "is_active": u.user.is_active,
        "roles": claims.roles,
        "permissions": group_permissions(&perms),
        "expires_at": claims.exp,
        "session_id": claims.sid,
    })))
}

// ---------------------------------------------------------- profile --

#[derive(Deserialize, ToSchema)]
pub struct ProfileUpdateBody {
    pub display_name: Option<String>,
    pub phone: Option<String>,
    pub location: Option<String>,
    pub bio: Option<String>,
    pub avatar_file_id: Option<Option<i32>>,
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/profile",
    tag = "auth",
    responses(
        (status = 200, description = "User profile with avatars"),
        (status = 401, description = "Not authenticated")
    )
)]
pub async fn get_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    claims.require("profile", "read")?;
    let u = controller::users::get(&state.pool, &claims.sub)?;
    let p = controller::profile::get(&state.pool, &claims.sub)?;
    let avatars = controller::profile::list_avatars(&state.pool, &claims.sub)?;
    let current_avatar = avatars
        .iter()
        .find(|a| Some(a.id) == p.avatar_file_id)
        .cloned();
    Ok(Json(json!({
        "user": {
            "id": u.user.id,
            "email": u.user.email,
            "display_name": u.user.display_name,
            "avatar_url": u.user.avatar_url,
            "first_name": u.user.first_name,
            "middle_name": u.user.middle_name,
            "last_name": u.user.last_name,
            "username": u.user.username,
        },
        "profile": p,
        "avatars": avatars,
        "current_avatar": current_avatar,
    })))
}

#[utoipa::path(
    patch,
    path = "/api/v1/auth/profile",
    tag = "auth",
    request_body = ProfileUpdateBody,
    responses(
        (status = 200, description = "Profile updated"),
        (status = 401, description = "Not authenticated"),
        (status = 403, description = "Insufficient permissions")
    )
)]
pub async fn update_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    ValidatedJson(body): ValidatedJson<ProfileUpdateBody>,
) -> ApiResult<Json<Value>> {
    claims.require("profile", "write")?;
    let patch = ModelsUpdateProfile {
        display_name: body.display_name.clone(),
        phone: body.phone.clone(),
        location: body.location.clone(),
        bio: body.bio.clone(),
        avatar_file_id: body.avatar_file_id,
    };
    let p = controller::profile::upsert(&state.pool, &claims.sub, &patch)?;

    // Also update gateway_users name fields if provided.
    let has_user_fields = body.first_name.is_some()
        || body.middle_name.is_some()
        || body.last_name.is_some()
        || body.username.is_some();
    if has_user_fields {
        let user_patch = models::UpdateUser {
            display_name: body.display_name.clone(),
            avatar_url: None,
            is_active: None,
            first_name: body.first_name.clone(),
            middle_name: body.middle_name.clone(),
            last_name: body.last_name.clone(),
            username: body.username.clone(),
        };
        let _ = controller::users::update(&state.pool, &claims.sub, user_patch);
        // Rebuild display_name from first + middle + last.
        let _ = controller::profile::sync_display_name(&state.pool, &claims.sub);
    }

    Ok(Json(json!({ "profile": p })))
}

pub async fn set_avatar(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(file_id): Path<i32>,
) -> ApiResult<Json<Value>> {
    claims.require("profile", "write")?;
    let p = controller::profile::set_avatar(&state.pool, &claims.sub, file_id)?;
    Ok(Json(json!({ "profile": p })))
}

pub async fn list_avatars(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    claims.require("profile", "read")?;
    let avatars = controller::profile::list_avatars(&state.pool, &claims.sub)?;
    Ok(Json(json!({ "avatars": avatars })))
}

pub async fn upload_avatar(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    mut multipart: axum::extract::Multipart,
) -> ApiResult<Json<Value>> {
    claims.require("profile", "write")?;
    let mut file: Option<(String, Vec<u8>, Option<String>)> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, format!("multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let filename = field.file_name().unwrap_or("avatar").to_string();
            let mime = field.content_type().map(|m| m.to_string());
            let data = field
                .bytes()
                .await
                .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, format!("read error: {e}")))?;
            file = Some((filename, data.to_vec(), mime));
        }
    }
    let (filename, data, mime) =
        file.ok_or_else(|| ApiError::new(StatusCode::BAD_REQUEST, "missing file field"))?;

    // Persist through the storage crate so the same upload pipeline is used.
    let stored = state.controllers().store_user_file(
        &claims.sub,
        "profile_avatars",
        &filename,
        mime.as_deref().unwrap_or("application/octet-stream"),
        &data,
    )?;
    let file = UserFile {
        id: stored.id,
        user_id: stored.user_id,
        filename: stored.filename,
        original_name: stored.original_name,
        mime_type: stored.mime_type,
        size_bytes: stored.size_bytes,
        storage_path: stored.storage_path,
        collection: stored.collection,
        is_public: stored.is_public,
        created_at: stored.created_at,
        updated_at: stored.updated_at,
    };
    Ok(Json(json!({ "file": file })))
}

// ---------------------------------------------------------- change password --

#[derive(Deserialize, ToSchema)]
pub struct ChangePasswordBody {
    pub current: String,
    pub new_password: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/profile/password",
    tag = "auth",
    request_body = ChangePasswordBody,
    responses(
        (status = 204, description = "Password changed"),
        (status = 401, description = "Not authenticated or wrong current password"),
        (status = 403, description = "Insufficient permissions")
    )
)]
pub async fn change_password(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    ValidatedJson(body): ValidatedJson<ChangePasswordBody>,
) -> ApiResult<StatusCode> {
    claims.require("profile", "write")?;
    let user = controller::users::find_by_id(&state.pool, &claims.sub)
        .map_err(|_| ApiError::from(AuthError::Invalid))?;
    verify_password(&body.current, &user.password_hash)?;
    let new_hash = hash_password(&body.new_password)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    controller::users::update_password(&state.pool, &claims.sub, &new_hash)?;
    // Bumping the user's token_version already happened in update_password;
    // revoke every live session so old cookies stop working.
    controller::sessions::revoke_all(&state.pool, &claims.sub)?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------- password reset --

#[derive(Deserialize, ToSchema)]
pub struct ResetRequestBody {
    pub email: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/password-reset/request",
    tag = "auth",
    request_body = ResetRequestBody,
    responses(
        (status = 200, description = "Reset email sent (or dev_token returned in dev mode)")
    )
)]
pub async fn request_password_reset(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    ValidatedJson(body): ValidatedJson<ResetRequestBody>,
) -> ApiResult<Json<Value>> {
    // Always 200 to avoid email enumeration.
    let _ = (addr, &headers);
    let user = match controller::users::find_by_email(&state.pool, &body.email) {
        Ok(u) if u.is_active => Some(u),
        _ => None,
    };
    let mut returned_token: Option<String> = None;
    if let Some(u) = user {
        let raw = Uuid::new_v4().to_string();
        let token_hash = hash_reset_token(&raw);
        let id = Uuid::new_v4().to_string();
        let expires_at = Utc::now().naive_utc() + Duration::hours(1);
        controller::sessions::create_reset(&state.pool, &id, &u.id, &token_hash, expires_at)?;

        if let Some(mailer) = controller::mailer::current() {
            let link = format!("{}/reset-password?token={}", mailer.public_base_url(), raw);
            if let Err(e) = mailer.send_reset_password(&u.email, &u.display_name, &raw) {
                tracing::warn!(error = %e, "failed to send reset email");
            }
            let _ = link;
        } else {
            tracing::info!(
                "[reset] SMTP not configured; raw token for {} = {}",
                u.email,
                raw
            );
            returned_token = Some(raw);
        }
    }
    let mut body = json!({ "ok": true });
    if let Some(tok) = returned_token {
        body["dev_token"] = json!(tok);
    }
    Ok(Json(body))
}

#[derive(Deserialize, ToSchema)]
pub struct ResetConfirmBody {
    pub token: String,
    pub new_password: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/password-reset/confirm",
    tag = "auth",
    request_body = ResetConfirmBody,
    responses(
        (status = 204, description = "Password reset successful"),
        (status = 400, description = "Invalid or expired token")
    )
)]
pub async fn confirm_password_reset(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<ResetConfirmBody>,
) -> ApiResult<StatusCode> {
    let hash = hash_reset_token(&body.token);
    let row = controller::sessions::find_reset_by_hash(&state.pool, &hash)?
        .ok_or_else(|| ApiError::from(AuthError::Invalid))?;
    if row.used_at.is_some() {
        return Err(ApiError::from(AuthError::Invalid));
    }
    let now = Utc::now().naive_utc();
    if row.expires_at < now {
        return Err(ApiError::from(AuthError::Expired));
    }
    let new_hash = hash_password(&body.new_password)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    controller::users::update_password(&state.pool, &row.user_id, &new_hash)?;
    controller::sessions::consume_reset(&state.pool, &row.id)?;
    // Force every session to log in again.
    controller::sessions::revoke_all(&state.pool, &row.user_id)?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------- sessions --

pub async fn list_sessions(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    claims.require("profile", "read")?;
    let sessions = controller::sessions::list_for_user(&state.pool, &claims.sub)?;
    let current_sid = claims.sid.clone();
    let items: Vec<Value> = sessions
        .into_iter()
        .map(|s: Session| {
            json!({
                "id": s.id,
                "user_id": s.user_id,
                "device": s.device,
                "user_agent": s.user_agent,
                "ip": s.ip,
                "created_at": s.created_at,
                "last_seen_at": s.last_seen_at,
                "expires_at": s.expires_at,
                "revoked_at": s.revoked_at,
                "current": s.id == current_sid,
            })
        })
        .collect();
    Ok(Json(json!({ "sessions": items })))
}

pub async fn revoke_session(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    claims.require("profile", "write")?;
    controller::sessions::revoke_by_id(&state.pool, &claims.sub, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// We need this for `register_and_login` in tests (kept around for compatibility).
#[allow(dead_code)]
pub fn current_time() -> i64 {
    now_secs()
}
