pub mod erp;
pub mod crm;
pub mod controllers;
pub mod fleet;
pub mod json;
pub mod maps;
pub mod middleware;
pub mod search;
pub mod state;
pub mod storage;
pub mod tile;
pub mod uploads;
pub mod wallet;

use axum::extract::DefaultBodyLimit;
use axum::routing::{delete, get, patch, post};
use axum::Router;
use tower_http::trace::TraceLayer;

pub use state::AppState;

pub fn public(state: AppState) -> Router {
    Router::new()
        .route("/health", get(controllers::health))
        .route("/auth/register", post(controllers::register))
        .route("/auth/login", post(controllers::login))
        // Public tile endpoints for map clients
        .merge(tile::tile_routes())
        // Public file serving (avatars, etc.)
        .merge(storage::storage_public_routes())
        .with_state(state)
}

pub fn private(state: AppState) -> Router {
    Router::new()
        .route("/auth/me", get(controllers::me))
        .route("/auth/logout", post(controllers::logout))
        .route("/auth/logout-all", post(controllers::logout_all))
        .route(
            "/users",
            get(controllers::list_users).post(controllers::create_user),
        )
        .route(
            "/users/:id",
            get(controllers::get_user)
                .patch(controllers::update_user)
                .delete(controllers::delete_user),
        )
        .route("/users/:id/roles/:role_id", post(controllers::assign_role))
        .route(
            "/users/:id/roles/:role_id",
            delete(controllers::revoke_role),
        )
        .route(
            "/roles",
            get(controllers::list_roles).post(controllers::create_role),
        )
        .route(
            "/roles/:id",
            get(controllers::get_role)
                .patch(controllers::update_role)
                .delete(controllers::delete_role),
        )
        .route(
            "/roles/:id/permissions/:permission_id",
            post(controllers::grant),
        )
        .route(
            "/roles/:id/permissions/:permission_id",
            delete(controllers::revoke),
        )
        .route(
            "/permissions",
            get(controllers::list_permissions).post(controllers::create_permission),
        )
        .route(
            "/permissions/:id",
            patch(controllers::update_permission).delete(controllers::delete_permission),
        )
        // System settings — readable by any signed-in user, writable by admins.
        .route("/system", get(controllers::get_system).patch(controllers::update_system))
        // Version & releases.
        .route("/system/version", get(controllers::get_version))
        .route(
            "/system/releases",
            get(controllers::list_releases).post(controllers::upload_release),
        )
        .route("/system/releases/latest", get(controllers::latest_release))
        .route(
            "/system/releases/check/:version",
            get(controllers::check_version),
        )
        .route(
            "/system/releases/:id",
            delete(controllers::delete_release),
        )
        .route(
            "/system/releases/:id/download",
            get(controllers::download_release),
        )
        // Support tickets.
        .route(
            "/support",
            get(controllers::list_tickets).post(controllers::create_ticket),
        )
        .route("/support/users", get(controllers::ticket_recipients))
        .route("/support/:id/close", post(controllers::close_ticket))
        // Apple Wallet — admin surface (wallet:read / wallet:write)
        .route("/wallet/samples", get(wallet::list_samples))
        .route("/wallet/passes", get(wallet::list_passes))
        .route("/wallet/issue", post(wallet::issue_pass))
        .route(
            "/wallet/passes/:pass_type/:serial",
            get(wallet::get_pass)
                .patch(wallet::update_pass)
                .delete(wallet::delete_pass),
        )
        .route("/wallet/passes/:pass_type/:serial/qr.svg", get(wallet::pass_qr))
        // Map layers and tiles — requires authentication
        .merge(maps::map_routes())
        // GeoJSON import — needs a higher body limit (25 MB)
        .merge(
            uploads::upload_routes()
                .layer(axum::extract::DefaultBodyLimit::max(25 * 1024 * 1024)),
        )
        // User file storage — requires authentication
        .merge(storage::storage_routes())
        // Fleet vehicles and map events
        .merge(fleet::fleet_routes())
        // ERP — 7 sub-modules ledger based
        .merge(erp::erp_routes())
        // CRM → ERP bridge (Quote → Sales Order → Accounting invoice)
        .merge(crm::crm_routes())
        // Search — Tantivy full-text search
        .route("/search", get(search::search))
        .route("/search/reindex", post(search::reindex))
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
    // Every API surface lives under one versioned prefix; the gateway serves
    // the SPA from the filesystem root alongside it.
    let api = Router::new()
        .merge(public(state.clone()))
        .merge(private(state.clone()))
        .merge(wallet_public(state));
    Router::new()
        .nest("/api/v1", api)
        .route("/health", get(controllers::health))
        .route("/sprites/*path", get(serve_sprites))
        .route("/glyphs/*path", get(serve_glyphs))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .layer(TraceLayer::new_for_http())
}

async fn serve_sprites(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> (axum::http::header::HeaderMap, Vec<u8>) {
    let full_path = std::path::PathBuf::from("map_assets/sprites").join(&path);
    let data = tokio::fs::read(&full_path).await.unwrap_or_default();
    let mime: axum::http::header::HeaderValue = if path.ends_with(".json") { "application/json" }
        else if path.ends_with(".png") { "image/png" }
        else { "application/octet-stream" }.parse().unwrap();
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
    headers.insert(axum::http::header::CONTENT_TYPE, "application/x-protobuf".parse().unwrap());
    (headers, data)
}
