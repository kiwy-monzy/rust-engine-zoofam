use axum::{
    response::IntoResponse,
    routing::Router,
};
use utoipa::OpenApi;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

use crate::auth;
use crate::system;
use crate::subscription;
use crate::rbac;
use crate::fleet;
use crate::wallet;
use crate::storage;
use crate::search;
use crate::tile;
use crate::uploads;
use models::subscription as ms;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "MkulimaLink API",
        version = "1.0.0",
        description = "Farmer-to-buyer marketplace for Tanzania. Connects farmers with buyers through mobile app, USSD (feature phones), and SMS.",
        contact(name = "MkulimaLink", email = "support@mkulimalink.tz")
    ),
    paths(
        // Health & System
        system::health,
        system::get_system,
        system::update_system,
        system::get_version,
        system::list_releases,
        system::upload_release,
        system::latest_release,
        system::check_version,
        system::delete_release,
        system::download_release,
        system::list_tickets,
        system::create_ticket,
        system::ticket_recipients,
        system::close_ticket,
        // Auth
        auth::register,
        auth::login,
        auth::refresh,
        auth::logout,
        auth::logout_all,
        auth::me,
        auth::request_password_reset,
        auth::confirm_password_reset,
        auth::get_profile,
        auth::update_profile,
        // Subscription
        subscription::list_plans,
        subscription::create_plan,
        subscription::update_plan,
        subscription::delete_plan,
        subscription::get_organization_subscription,
        subscription::subscribe_organization,
        subscription::cancel_subscription,
        subscription::list_invoices,
        // RBAC
        rbac::controllers::users::list,
        rbac::controllers::users::search,
        rbac::controllers::users::get,
        rbac::controllers::users::create,
        rbac::controllers::users::update,
        rbac::controllers::users::delete,
        rbac::controllers::users::assign_role,
        rbac::controllers::users::revoke_role,
        rbac::controllers::roles::list,
        rbac::controllers::roles::get,
        rbac::controllers::roles::create,
        rbac::controllers::roles::update,
        rbac::controllers::roles::delete,
        rbac::controllers::roles::grant,
        rbac::controllers::roles::revoke,
        rbac::controllers::permissions::list,
        rbac::controllers::permissions::create,
        rbac::controllers::permissions::update,
        rbac::controllers::permissions::delete,
        // Fleet
        fleet::gateway::vault_list,
        fleet::gateway::vault_put,
        fleet::gateway::bolt_list,
        fleet::gateway::bolt_login_start,
        fleet::gateway::bolt_login_confirm,
        fleet::gateway::bolt_status,
        fleet::gateway::marine_cookie,
        fleet::gateway::marine_cookie_set,
        fleet::gateway::tile_handler,
        fleet::gateway::summary,
        // Wallet
        wallet::download_pass,
        wallet::register_device,
        wallet::unregister_device,
        wallet::list_updates,
        wallet::latest_pass,
        wallet::log,
        // Storage
        storage::upload_file_handler,
        storage::list_files_handler,
        storage::get_file_handler,
        storage::delete_file_handler,
        storage::serve_file_handler,
        // Search
        search::search,
        search::reindex,
        // Tile
        tile::tile_handler,
        // Uploads
        uploads::import_geojson_handler,
    ),
    components(
        schemas(
            auth::LoginResponse,
            auth::PermissionItem,
            auth::PermissionGroup,
            models::User,
            models::UserWithRoles,
            models::Role,
            models::RoleWithPermissions,
            models::Permission,
            models::NewPermission,
            models::UpdatePermission,
            models::CreateUser,
            models::UpdateUser,
            models::CreateRole,
            models::UpdateRole,
            models::Credentials,
            models::SystemSetting,
            models::UpdateSystem,
            models::Release,
            models::NewRelease,
            system::UploadReleaseRequest,
            models::SupportTicket,
            models::TicketView,
            models::CreateTicket,
            auth::ProfileUpdateBody,
            auth::ChangePasswordBody,
            auth::ResetRequestBody,
            auth::ResetConfirmBody,
            ms::Plan,
            ms::NewPlan,
            ms::UpdatePlan,
            ms::Subscription,
            ms::NewSubscription,
            ms::Invoice,
            ms::NewInvoice,
            subscription::SubscribeRequest,
            // Fleet schemas
            fleet::gateway::VaultPutBody,
            fleet::gateway::BoltStartBody,
            fleet::gateway::BoltConfirmBody,
            fleet::gateway::MarineCookieBody,
            // Wallet schemas
            wallet::RegistrationBody,
            wallet::LogBody,
            // Storage schemas
            storage::UploadFileResponse,
        )
    ),
    tags(
        (name = "health", description = "Service health check"),
        (name = "auth", description = "Authentication - registration, login, token refresh"),
        (name = "users", description = "User profile management"),
        (name = "rbac", description = "Role-based access control - users, roles, permissions"),
        (name = "system", description = "System settings, releases, support"),
        (name = "maps", description = "Map layers, features, tiles"),
        (name = "fleet", description = "Fleet vehicle tracking"),
        (name = "erp", description = "ERP - products, procurement, inventory, sales"),
        (name = "crm", description = "CRM - leads, opportunities, customers, quotes"),
        (name = "dmc", description = "Destination Management Company - tours, bookings"),
        (name = "marketplace", description = "Service marketplace - organizations, services, bookings"),
        (name = "website", description = "Website templates"),
        (name = "storage", description = "File storage and uploads"),
        (name = "search", description = "Full-text search"),
    )
)]
pub struct ApiDoc;

pub fn openapi_schema() -> utoipa::openapi::OpenApi {
    let mut api = ApiDoc::openapi();
    // Add JWT bearer auth security scheme for Swagger UI "Authorize" button
    if let Some(components) = api.components.as_mut() {
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("Enter JWT access token here"))
                    .build(),
            ),
        );
    }
    // Apply global security requirement
    api.security = Some(vec![utoipa::openapi::security::SecurityRequirement::new(
        "bearer_auth",
        Vec::<String>::new(),
    )]);
    api
}

pub fn swagger_ui() -> Router {
    // Option 1: Default Swagger UI with config
    let config = utoipa_swagger_ui::Config::from("/api-docs/openapi.json")
        .try_it_out_enabled(true)
        .filter(true)
        .show_common_extensions(true)
        .request_snippets_enabled(true)
        .show_extensions(true);

    utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
        .config(config)
        .url("/api-docs/openapi.json", openapi_schema())
        .into()
}

/// Custom Swagger UI with MkulimaLink branding and instructions
pub fn swagger_ui_custom() -> Router {
    use axum::response::Html;
    use axum::routing::get;

    let custom_html = include_str!("swagger.html").to_string();

    Router::new()
        .route("/swagger-custom", get(move || async { Html(custom_html) }))
        .route("/api-docs/openapi.json", get(swagger_openapi))
}

pub async fn swagger_openapi() -> impl IntoResponse {
    axum::Json(openapi_schema())
}
