// System routes - settings, releases, support
mod controllers;

use axum::routing::{delete, get, patch, post};
use axum::Router;

use crate::state::AppState;

/// Build system routes for settings, releases, and support
pub fn routes() -> Router<AppState> {
    Router::new()
        // System settings
        .route(
            "/system",
            get(controllers::get_system).patch(controllers::update_system),
        )
        // Version & releases
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
        .route("/system/releases/:id", delete(controllers::delete_release))
        .route(
            "/system/releases/:id/download",
            get(controllers::download_release),
        )
        // Support tickets
        .route(
            "/support",
            get(controllers::list_tickets).post(controllers::create_ticket),
        )
        .route("/support/users", get(controllers::ticket_recipients))
        .route("/support/:id/close", post(controllers::close_ticket))
}

// Re-export health for use in lib.rs
pub use controllers::health;
// Re-export handlers for openapi.rs
pub use controllers::*;
