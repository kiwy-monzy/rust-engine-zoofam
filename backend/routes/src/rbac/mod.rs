// RBAC routes - users, roles, permissions
pub mod controllers;

use axum::routing::{delete, get, patch, post};
use axum::Router;

use crate::state::AppState;

/// Build RBAC routes for users, roles, and permissions
pub fn routes() -> Router<AppState> {
    Router::new()
        // Users
        .route(
            "/users",
            get(controllers::users::list).post(controllers::users::create),
        )
        .route("/users/search", get(controllers::users::search))
        .route(
            "/users/:id",
            get(controllers::users::get)
                .patch(controllers::users::update)
                .delete(controllers::users::delete),
        )
        .route(
            "/users/:id/roles/:role_id",
            post(controllers::users::assign_role)
                .delete(controllers::users::revoke_role),
        )
        // Roles
        .route(
            "/roles",
            get(controllers::roles::list).post(controllers::roles::create),
        )
        .route(
            "/roles/:id",
            get(controllers::roles::get)
                .patch(controllers::roles::update)
                .delete(controllers::roles::delete),
        )
        .route(
            "/roles/:id/permissions/:permission_id",
            post(controllers::roles::grant).delete(controllers::roles::revoke),
        )
        // Permissions
        .route(
            "/permissions",
            get(controllers::permissions::list).post(controllers::permissions::create),
        )
        .route(
            "/permissions/:id",
            patch(controllers::permissions::update)
                .delete(controllers::permissions::delete),
        )
}
