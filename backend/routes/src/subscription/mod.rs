//! Subscription & Plans module
//!
//! Manages subscription plans, organization subscriptions, and billing.
//! Organizations subscribe to plans that determine their feature limits.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;

use auth::Claims;
use models::subscription::{NewPlan, Plan, Subscription, UpdatePlan};

use crate::middleware::ApiResult;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct SubscribeRequest {
    pub plan_id: String,
    pub period: String,
}

// ---------------------------------------------------------- Plans --

#[utoipa::path(
    get,
    path = "/api/v1/subscription/plans",
    tag = "subscription",
    responses(
        (status = 200, description = "List of available plans", body = Vec<Plan>),
    )
)]
pub async fn list_plans(State(state): State<AppState>) -> ApiResult<Json<Value>> {
    let plans = controller::subscription::list_plans(&state.pool)?;
    Ok(Json(json!({ "plans": plans })))
}

#[utoipa::path(
    post,
    path = "/api/v1/subscription/plans",
    tag = "subscription",
    request_body = NewPlan,
    responses(
        (status = 201, description = "Plan created", body = Plan),
        (status = 403, description = "Admin access required")
    )
)]
pub async fn create_plan(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<NewPlan>,
) -> ApiResult<(StatusCode, Json<Plan>)> {
    claims.require("subscription", "admin")?;
    let plan = controller::subscription::create_plan(&state.pool, body)?;
    Ok((StatusCode::CREATED, Json(plan)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/subscription/plans/{plan_id}",
    tag = "subscription",
    request_body = UpdatePlan,
    responses(
        (status = 200, description = "Plan updated", body = Plan),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "Plan not found")
    )
)]
pub async fn update_plan(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(plan_id): Path<String>,
    Json(body): Json<UpdatePlan>,
) -> ApiResult<Json<Plan>> {
    claims.require("subscription", "admin")?;
    let plan = controller::subscription::update_plan(&state.pool, &plan_id, body)?;
    Ok(Json(plan))
}

#[utoipa::path(
    delete,
    path = "/api/v1/subscription/plans/{plan_id}",
    tag = "subscription",
    responses(
        (status = 204, description = "Plan deleted"),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "Plan not found")
    )
)]
pub async fn delete_plan(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(plan_id): Path<String>,
) -> ApiResult<StatusCode> {
    claims.require("subscription", "admin")?;
    controller::subscription::delete_plan(&state.pool, &plan_id)?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------- Organization Subscriptions --

#[utoipa::path(
    get,
    path = "/api/v1/subscription/organizations/{org_id}",
    tag = "subscription",
    responses(
        (status = 200, description = "Organization subscription details", body = Subscription),
        (status = 404, description = "Organization not found")
    )
)]
pub async fn get_organization_subscription(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> ApiResult<Json<Subscription>> {
    let sub = controller::subscription::get_organization_subscription(&state.pool, &org_id)?;
    Ok(Json(sub))
}

#[utoipa::path(
    post,
    path = "/api/v1/subscription/organizations/{org_id}/subscribe",
    tag = "subscription",
    request_body = SubscribeRequest,
    responses(
        (status = 201, description = "Subscription created", body = Subscription),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "Organization or plan not found")
    )
)]
pub async fn subscribe_organization(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(org_id): Path<String>,
    Json(body): Json<SubscribeRequest>,
) -> ApiResult<(StatusCode, Json<Subscription>)> {
    claims.require("subscription", "write")?;
    let sub = controller::subscription::subscribe(&state.pool, &org_id, &body.plan_id, &body.period)?;
    Ok((StatusCode::CREATED, Json(sub)))
}

#[utoipa::path(
    post,
    path = "/api/v1/subscription/organizations/{org_id}/cancel",
    tag = "subscription",
    responses(
        (status = 200, description = "Subscription cancelled"),
        (status = 403, description = "Admin access required"),
        (status = 404, description = "Subscription not found")
    )
)]
pub async fn cancel_subscription(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(org_id): Path<String>,
) -> ApiResult<Json<Value>> {
    claims.require("subscription", "write")?;
    controller::subscription::cancel(&state.pool, &org_id)?;
    Ok(Json(json!({ "status": "cancelled" })))
}

// ---------------------------------------------------------- Invoices --

#[utoipa::path(
    get,
    path = "/api/v1/subscription/organizations/{org_id}/invoices",
    tag = "subscription",
    responses(
        (status = 200, description = "List of invoices"),
        (status = 404, description = "Organization not found")
    )
)]
pub async fn list_invoices(
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> ApiResult<Json<Value>> {
    let invoices = controller::subscription::list_invoices(&state.pool, &org_id)?;
    Ok(Json(json!({ "invoices": invoices })))
}

// ---------------------------------------------------------- Router --

pub fn subscription_routes() -> axum::Router<AppState> {
    use axum::routing::{delete, get, post, patch};
    axum::Router::new()
        .route("/subscription/plans", get(list_plans).post(create_plan))
        .route("/subscription/plans/:plan_id", patch(update_plan).delete(delete_plan))
        .route("/subscription/organizations/:org_id", get(get_organization_subscription))
        .route("/subscription/organizations/:org_id/subscribe", post(subscribe_organization))
        .route("/subscription/organizations/:org_id/cancel", post(cancel_subscription))
        .route("/subscription/organizations/:org_id/invoices", get(list_invoices))
}
