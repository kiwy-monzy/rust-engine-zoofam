use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use diesel::result::Error as DieselError;
use serde_json::json;

use auth::{AuthError, Claims};

use crate::state::AppState;

pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

impl From<AuthError> for ApiError {
    fn from(e: AuthError) -> Self {
        let status = match e {
            AuthError::Missing | AuthError::Invalid | AuthError::Expired => {
                StatusCode::UNAUTHORIZED
            }
            AuthError::BadCredentials => StatusCode::UNAUTHORIZED,
            AuthError::Disabled | AuthError::Forbidden(_) => StatusCode::FORBIDDEN,
            AuthError::Hashing(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        ApiError::new(status, e.to_string())
    }
}

impl From<controller::Error> for ApiError {
    fn from(e: controller::Error) -> Self {
        use applewallet::WalletError as W;
        use controller::Error as E;
        match e {
            E::Auth(inner) => inner.into(),
            E::NotFound(_) => ApiError::new(StatusCode::NOT_FOUND, e.to_string()),
            E::Conflict(_) => ApiError::new(StatusCode::CONFLICT, e.to_string()),
            E::Invalid(_) => ApiError::new(StatusCode::BAD_REQUEST, e.to_string()),
            E::Forbidden(_) => ApiError::new(StatusCode::FORBIDDEN, e.to_string()),
            E::SessionEnded(_) => ApiError::new(StatusCode::UNAUTHORIZED, e.to_string()),
            E::PaymentFailed(_) => ApiError::new(StatusCode::BAD_REQUEST, e.to_string()),
            E::Clickpesa(_) => ApiError::new(StatusCode::BAD_REQUEST, e.to_string()),
            E::Wallet(W::UnknownSerial(_)) => ApiError::new(StatusCode::NOT_FOUND, e.to_string()),
            E::Wallet(W::Keys(_) | W::OpenSsl(_)) => ApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "wallet signing is not configured on this server",
            ),
            E::Wallet(W::Apns(inner)) => {
                tracing::warn!("wallet APNs push failed: {inner}");
                ApiError::new(
                    StatusCode::BAD_GATEWAY,
                    "the pass update could not be pushed to Apple",
                )
            }
            E::Wallet(_) => ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            E::Db(inner) => {
                let detail = format!("database: {inner}");
                eprintln!("[routes] {detail}");
                tracing::error!("{detail}");
                ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, detail)
            }
        }
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

impl From<DieselError> for ApiError {
    fn from(e: DieselError) -> Self {
        tracing::error!("database error: {e}");
        ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "database error")
    }
}

impl From<maps::MapError> for ApiError {
    fn from(e: maps::MapError) -> Self {
        use maps::MapError as E;
        match e {
            E::LayerNotFound(_)
            | E::SourceNotFound(_)
            | E::FeatureNotFound(_)
            | E::StyleNotFound(_) => ApiError::new(StatusCode::NOT_FOUND, e.to_string()),
            E::Unauthorized(_) => ApiError::new(StatusCode::FORBIDDEN, e.to_string()),
            E::Database(inner) => {
                tracing::error!("maps database error: {inner}");
                ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "something went wrong")
            }
            E::Configuration(_) => ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "map services are not configured correctly",
            ),
            _ => ApiError::new(StatusCode::BAD_REQUEST, e.to_string()),
        }
    }
}

pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // 1) Bearer header (CLI, mobile, server-to-server).
    // 2) `gw_access` cookie (browser SPA).
    let token = extract_access_token(request.headers())
        .ok_or_else(|| ApiError::from(AuthError::Missing))?;
    let claims = state.jwt.verify_access(&token)?;

    // 3) The signature and expiry are good. Now the part a stateless token
    //    cannot do on its own: confirm the session is still live — the
    //    account active, the token not signed out, the version not bumped by
    //    a sign-out-everywhere, the refresh family still present.
    //    This is one indexed read per request, the price of revocation.
    controller::sessions::validate(&state.pool, &claims).map_err(ApiError::from)?;

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}

fn extract_access_token(headers: &axum::http::HeaderMap) -> Option<String> {
    if let Some(h) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
        if let Some(stripped) = h
            .strip_prefix("Bearer ")
            .or_else(|| h.strip_prefix("bearer "))
        {
            let s = stripped.trim();
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }
    if let Some(c) = headers.get("cookie").and_then(|v| v.to_str().ok()) {
        for pair in c.split(';') {
            let pair = pair.trim();
            if let Some(rest) = pair.strip_prefix("gw_access=") {
                if !rest.is_empty() {
                    return Some(rest.to_string());
                }
            }
        }
    }
    None
}

pub fn claims(request: &Request) -> Result<&Claims, ApiError> {
    request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| ApiError::from(AuthError::Missing))
}

/// Middleware to require a specific permission
/// This checks if the user's role has the required permission
pub fn require_permission(
    required_permission: &'static str,
) -> impl Fn(Request, Next) -> futures::future::BoxFuture<'static, Result<Response, ApiError>> + Clone
{
    move |request: Request, next: Next| {
        let permission = required_permission;
        Box::pin(async move {
            let claims = claims(&request)?;

            // Extract the module and action from the permission string (e.g., "maps:read")
            let parts: Vec<&str> = permission.split(':').collect();
            if parts.len() != 2 {
                return Err(ApiError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Invalid permission format",
                ));
            }

            let (module, action) = (parts[0], parts[1]);

            // Check if user has the required permission
            // This would typically query the database, but for now we'll do a simple check
            // In production, this should cache role permissions for performance
            let has_permission = check_user_permission(&claims.user_id(), module, action).await?;

            if !has_permission {
                return Err(ApiError::new(
                    StatusCode::FORBIDDEN,
                    format!("Permission '{}' required", permission),
                ));
            }

            Ok(next.run(request).await)
        })
    }
}

async fn check_user_permission(
    user_id: &Option<uuid::Uuid>,
    module: &str,
    action: &str,
) -> Result<bool, ApiError> {
    let uid = match user_id {
        Some(id) => id.to_string(),
        None => return Ok(false),
    };

    let db_url = db::database_url();
    let pool = db::create_pool_from(&db_url)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, format!("db pool: {e}")))?;
    let mut conn = db::conn(&pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, format!("db conn: {e}")))?;

    use diesel::dsl::sql_query;
    use diesel::prelude::*;

    #[derive(QueryableByName)]
    struct PermCheck {
        #[diesel(sql_type = diesel::sql_types::Integer)]
        _ok: i32,
    }

    let row: Option<PermCheck> = sql_query(
        "SELECT 1 AS _ok FROM gateway_user_roles ur \
         JOIN gateway_role_permissions rp ON rp.role_id = ur.role_id \
         JOIN gateway_permissions p ON p.id = rp.permission_id \
         WHERE ur.user_id = ?1 AND p.module = ?2 AND p.action = ?3 \
         LIMIT 1",
    )
    .bind::<diesel::sql_types::Text, _>(&uid)
    .bind::<diesel::sql_types::Text, _>(module)
    .bind::<diesel::sql_types::Text, _>(action)
    .get_result::<PermCheck>(&mut conn)
    .optional()
    .map_err(|e| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("permission query: {e}"),
        )
    })?;

    Ok(row.is_some())
}
