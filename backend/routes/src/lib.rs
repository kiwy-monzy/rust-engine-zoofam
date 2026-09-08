// Route modules - organized by domain
pub mod auth;
pub mod rbac;
pub mod subscription;
pub mod system;
pub mod maps;
pub mod erp;
pub mod crm;
pub mod dmc;
pub mod marketplace;
pub mod website;
pub mod storage;
pub mod fleet;
pub mod wallet;
pub mod search;
pub mod tile;
pub mod uploads;
pub mod thirdparty;

// Core modules
pub mod json;
pub mod middleware;
pub mod openapi;
pub mod state;

use axum::extract::DefaultBodyLimit;
use axum::routing::{delete, get, patch, post};
use axum::Router;
use tower_http::trace::TraceLayer;

pub use state::AppState;

pub fn public(state: AppState) -> Router {
    Router::new()
        .route("/health", get(system::health))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/refresh", post(auth::refresh))
        .route(
            "/auth/password-reset/request",
            post(auth::request_password_reset),
        )
        .route(
            "/auth/password-reset/confirm",
            post(auth::confirm_password_reset),
        )
        // Public tile endpoints for map clients
        .merge(tile::tile_routes())
        // Public file serving (avatars, etc.)
        .merge(storage::storage_public_routes())
        // Third-party public webhooks (no auth required)
        .merge(thirdparty::public::public_routes())
        .with_state(state)
}

pub fn private(state: AppState) -> Router {
    Router::new()
        // Auth — session management
        .route("/auth/me", get(auth::me))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/logout-all", post(auth::logout_all))
        // Profile + sessions + avatars
        .route(
            "/auth/profile",
            get(auth::get_profile).patch(auth::update_profile),
        )
        .route("/auth/profile/avatar", post(auth::upload_avatar))
        .route("/auth/profile/avatar/pick", post(auth::set_avatar))
        .route("/auth/profile/avatars", get(auth::list_avatars))
        .route("/auth/profile/password", post(auth::change_password))
        .route("/auth/profile/sessions", get(auth::list_sessions))
        .route("/auth/profile/sessions/:id", delete(auth::revoke_session))
        // RBAC — Users, Roles, Permissions
        .merge(rbac::routes())
        // System settings, releases, support
        .merge(system::routes())
        // Map layers, tiles and events — requires authentication
        .merge(maps::map_routes())
        // GeoJSON import — needs a higher body limit (25 MB)
        .merge(
            uploads::upload_routes().layer(axum::extract::DefaultBodyLimit::max(25 * 1024 * 1024)),
        )
        // User file storage — requires authentication
        .merge(storage::storage_routes())
        // Fleet vehicles and map events
        .merge(fleet::fleet_routes())
        // ERP — 7 sub-modules ledger based
        .merge(erp::erp_routes())
        // CRM → ERP bridge (Quote → Sales Order → Accounting invoice)
        .merge(crm::crm_routes())
        // DMC — Destination Management Company / tour operator
        .merge(dmc::dmc_routes())
        // Service Marketplace (Fundi)
        .merge(marketplace::marketplace_routes())
        // Website — Template 0 (Knowlia) with hero/slideshow/downloads/links/products/cart/bookings
        .merge(website::website_routes())
        // Third-party integrations (Clickpesa, BeemAfrica, Notifty)
        .merge(thirdparty::admin::routes())
        .merge(thirdparty::clickpesa::routes())
        .merge(thirdparty::beem::routes())
        .merge(thirdparty::notifty::routes())
        // Search — Tantivy full-text search
        .route("/search", get(search::search))
        .route("/search/reindex", post(search::reindex))
        // Subscription & Plans — manage organization subscriptions
        .merge(subscription::subscription_routes())
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::require_auth,
        ))
        .with_state(state)
}

/// iPhone-facing wallet endpoints. No user JWT: the download link is what a
/// QR encodes, and the pass-update web service authenticates with each pass's
/// `Authorization: ApplePass <token>` credential instead.
pub fn wallet_public(state: AppState) -> Router {
    Router::new()
        .route(
            "/wallet/pass/:pass_type/:serial_pkpass",
            get(wallet::download_pass),
        )
        .route(
            "/wallet/v1/devices/:device_id/registrations/:pass_type/:serial",
            post(wallet::register_device).delete(wallet::unregister_device),
        )
        .route(
            "/wallet/v1/devices/:device_id/registrations/:pass_type",
            get(wallet::list_updates),
        )
        .route(
            "/wallet/v1/passes/:pass_type/:serial",
            get(wallet::latest_pass),
        )
        .route("/wallet/v1/log", post(wallet::log))
        .with_state(state)
}

pub fn app(state: AppState) -> Router {
    let api = Router::new()
        .merge(public(state.clone()))
        .merge(private(state.clone()))
        .merge(wallet_public(state));
    Router::new()
        .nest("/api/v1", api)
        .route("/health", get(system::health))
        .route("/routes", get(list_routes))
        .route("/sprites/*path", get(serve_sprites))
        .route("/glyphs/*path", get(serve_glyphs))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .layer(TraceLayer::new_for_http())
}

async fn list_routes() -> impl axum::response::IntoResponse {
    let routes = &[
        // Public routes
        ("GET", "/api/v1/health"),
        ("POST", "/api/v1/auth/register"),
        ("POST", "/api/v1/auth/login"),
        ("POST", "/api/v1/auth/refresh"),
        ("POST", "/api/v1/auth/password-reset/request"),
        ("POST", "/api/v1/auth/password-reset/confirm"),
        // Wallet public
        ("GET", "/api/v1/wallet/pass/:pass_type/:serial_pkpass"),
        (
            "POST",
            "/api/v1/wallet/v1/devices/:device_id/registrations/:pass_type/:serial",
        ),
        (
            "DELETE",
            "/api/v1/wallet/v1/devices/:device_id/registrations/:pass_type/:serial",
        ),
        (
            "GET",
            "/api/v1/wallet/v1/devices/:device_id/registrations/:pass_type",
        ),
        ("GET", "/api/v1/wallet/v1/passes/:pass_type/:serial"),
        ("POST", "/api/v1/wallet/v1/log"),
        // Private routes (JWT required)
        ("GET", "/api/v1/auth/me"),
        ("POST", "/api/v1/auth/logout"),
        ("POST", "/api/v1/auth/logout-all"),
        ("GET", "/api/v1/auth/profile"),
        ("PATCH", "/api/v1/auth/profile"),
        ("POST", "/api/v1/auth/profile/avatar"),
        ("POST", "/api/v1/auth/profile/avatar/pick"),
        ("GET", "/api/v1/auth/profile/avatars"),
        ("POST", "/api/v1/auth/profile/password"),
        ("GET", "/api/v1/auth/profile/sessions"),
        ("DELETE", "/api/v1/auth/profile/sessions/:id"),
        // RBAC - Users
        ("GET", "/api/v1/users"),
        ("POST", "/api/v1/users"),
        ("GET", "/api/v1/users/search"),
        ("GET", "/api/v1/users/:id"),
        ("PATCH", "/api/v1/users/:id"),
        ("DELETE", "/api/v1/users/:id"),
        ("POST", "/api/v1/users/:id/roles/:role_id"),
        ("DELETE", "/api/v1/users/:id/roles/:role_id"),
        // RBAC - Roles
        ("GET", "/api/v1/roles"),
        ("POST", "/api/v1/roles"),
        ("GET", "/api/v1/roles/:id"),
        ("PATCH", "/api/v1/roles/:id"),
        ("DELETE", "/api/v1/roles/:id"),
        ("POST", "/api/v1/roles/:id/permissions/:permission_id"),
        ("DELETE", "/api/v1/roles/:id/permissions/:permission_id"),
        // RBAC - Permissions
        ("GET", "/api/v1/permissions"),
        ("POST", "/api/v1/permissions"),
        ("PATCH", "/api/v1/permissions/:id"),
        ("DELETE", "/api/v1/permissions/:id"),
        // System
        ("GET", "/api/v1/system"),
        ("PATCH", "/api/v1/system"),
        ("GET", "/api/v1/system/version"),
        ("GET", "/api/v1/system/releases"),
        ("POST", "/api/v1/system/releases"),
        ("GET", "/api/v1/system/releases/latest"),
        ("GET", "/api/v1/system/releases/check/:version"),
        ("DELETE", "/api/v1/system/releases/:id"),
        ("GET", "/api/v1/system/releases/:id/download"),
        // Support
        ("GET", "/api/v1/support"),
        ("POST", "/api/v1/support"),
        ("GET", "/api/v1/support/users"),
        ("POST", "/api/v1/support/:id/close"),
        // Maps, Uploads, Storage, Fleet, ERP, CRM, DMC, Marketplace, Website, Search
        // (merged routes - see respective modules)
        ("GET", "/api/v1/maps/*"),
        ("POST", "/api/v1/uploads/*"),
        ("GET", "/api/v1/storage/*"),
        ("GET", "/api/v1/fleet/*"),
        ("GET", "/api/v1/erp/*"),
        ("GET", "/api/v1/crm/*"),
        ("GET", "/api/v1/dmc/*"),
        ("GET", "/api/v1/marketplace/*"),
        ("GET", "/api/v1/website/*"),
        ("GET", "/api/v1/search"),
        ("POST", "/api/v1/search/reindex"),
    ];

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head><title>API Routes</title></head>
<body>
<h1>API Routes ({})</h1>
<table border="1" cellpadding="5">
<tr><th>Method</th><th>Path</th></tr>
{}
</table>
</body>
</html>"#,
        routes.len(),
        routes
            .iter()
            .map(|(m, p)| format!("<tr><td>{}</td><td><code>{}</code></td></tr>", m, p))
            .collect::<Vec<_>>()
            .join("\n")
    );

    axum::response::Html(html)
}

async fn serve_sprites(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> (axum::http::header::HeaderMap, Vec<u8>) {
    let full_path = std::path::PathBuf::from("map_assets/sprites").join(&path);
    let data = tokio::fs::read(&full_path).await.unwrap_or_default();
    let mime: axum::http::header::HeaderValue = if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".png") {
        "image/png"
    } else {
        "application/octet-stream"
    }
    .parse()
    .unwrap();
    let mut headers = axum::http::header::HeaderMap::new();
    headers.insert(axum::http::header::CONTENT_TYPE, mime);
    (headers, data)
}

async fn serve_glyphs(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> (axum::http::header::HeaderMap, Vec<u8>) {
    let full_path = std::path::PathBuf::from("map_assets/glyphs").join(&path);
    let data = tokio::fs::read(&full_path).await.unwrap_or_default();
    let mut headers = axum::http::header::HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        "application/x-protobuf".parse().unwrap(),
    );
    (headers, data)
}
