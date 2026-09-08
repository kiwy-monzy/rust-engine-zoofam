use crate::{
    middleware::{ApiError, ApiResult},
    state::AppState,
};
use auth::Claims;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
    Extension, Json, Router,
};
use models::marketplace::{
    UpdateBooking, UpdateCartItem, UpdateCategory, UpdateOrgMember, UpdateOrganization,
    UpdatePromotion, UpdateService,
};
use serde::Deserialize;
use serde_json::{json, Value};

fn need(c: &Claims, m: &str, a: &str) -> Result<(), ApiError> {
    c.require(m, a)
        .map_err(|e| ApiError::new(StatusCode::FORBIDDEN, e.to_string()))
}

pub fn marketplace_routes() -> Router<AppState> {
    Router::new()
        // Organizations (providers)
        .route(
            "/marketplace/organizations",
            get(list_organizations).post(create_organization),
        )
        .route(
            "/marketplace/organizations/:id",
            get(get_organization)
                .patch(update_organization)
                .delete(delete_organization),
        )
        // Org members
        .route(
            "/marketplace/organizations/:org_id/members",
            get(list_org_members).post(add_org_member),
        )
        .route(
            "/marketplace/org-members/:id",
            patch(update_org_member).delete(remove_org_member),
        )
        // Categories
        .route(
            "/marketplace/categories",
            get(list_categories).post(create_category),
        )
        .route(
            "/marketplace/categories/:id",
            get(get_category)
                .patch(update_category)
                .delete(delete_category),
        )
        // Services
        .route(
            "/marketplace/services",
            get(list_services).post(create_service),
        )
        .route(
            "/marketplace/services/:id",
            get(get_service)
                .patch(update_service)
                .delete(delete_service),
        )
        .route(
            "/marketplace/organizations/:org_id/services",
            get(list_services_by_org),
        )
        // Zones
        .route(
            "/marketplace/organizations/:org_id/zones",
            get(list_zones).post(create_zone),
        )
        .route("/marketplace/zones/:id", delete(delete_zone))
        // Bookings
        .route(
            "/marketplace/bookings",
            get(list_bookings).post(create_booking),
        )
        .route(
            "/marketplace/bookings/:id",
            get(get_booking)
                .patch(update_booking)
                .delete(delete_booking),
        )
        .route(
            "/marketplace/organizations/:org_id/bookings",
            get(list_bookings_by_org),
        )
        .route(
            "/marketplace/customers/:user_id/bookings",
            get(list_bookings_by_customer),
        )
        // Custom requests
        .route(
            "/marketplace/requests",
            get(list_requests).post(create_request),
        )
        .route(
            "/marketplace/requests/:id",
            get(get_request).delete(delete_request),
        )
        // Bids
        .route(
            "/marketplace/requests/:request_id/bids",
            get(list_bids).post(create_bid),
        )
        .route("/marketplace/bids/:id", get(get_bid).delete(delete_bid))
        .route("/marketplace/bids/:id/accept", post(accept_bid))
        // Cart
        .route("/marketplace/cart", post(add_to_cart_item))
        .route(
            "/marketplace/cart/:id",
            get(list_cart)
                .patch(update_cart_item)
                .delete(remove_cart_item),
        )
        .route("/marketplace/cart/user/:user_id", delete(clear_cart_items))
        // Reviews
        .route(
            "/marketplace/organizations/:org_id/reviews",
            get(list_org_reviews).post(create_review),
        )
        // Commissions
        .route(
            "/marketplace/organizations/:org_id/commissions",
            get(list_org_commissions),
        )
        .route("/marketplace/commissions", post(create_commission_entry))
        // Payouts
        .route("/marketplace/payouts", get(list_all_payouts))
        .route(
            "/marketplace/organizations/:org_id/payouts",
            get(list_org_payouts).post(create_payout_entry),
        )
        .route(
            "/marketplace/payouts/:id/complete",
            post(complete_payout_entry),
        )
        // Promotions
        .route(
            "/marketplace/promotions",
            get(list_promotions).post(create_promotion_entry),
        )
        .route(
            "/marketplace/promotions/:id",
            get(get_promotion_entry)
                .patch(update_promotion_entry)
                .delete(delete_promotion_entry),
        )
        .route("/marketplace/promos/validate", post(validate_promo))
        // Notifications
        .route("/marketplace/notifications/me", get(list_my_notifications))
        .route(
            "/marketplace/notifications/:user_id",
            get(list_user_notifications),
        )
        .route("/marketplace/notifications/:id/read", post(mark_read))
        .route(
            "/marketplace/notifications/user/:user_id/read-all",
            post(mark_all_read),
        )
}

// --------------------------------------------------------------- organizations ---

async fn list_organizations(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"organizations": controller::marketplace::list_organizations(&s.pool)?}),
    ))
}
async fn get_organization(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"organization": controller::marketplace::get_organization(&s.pool, &id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateOrg {
    owner_user_id: String,
    name: String,
    slug: String,
    business_name: Option<String>,
    logo_url: Option<String>,
    description: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    website: Option<String>,
    address: Option<String>,
    city: Option<String>,
    country: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    timezone: Option<String>,
    base_currency: Option<String>,
}
async fn create_organization(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<CreateOrg>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "marketplace", "write")?;
    let r = controller::marketplace::create_organization(
        &s.pool,
        &v.owner_user_id,
        &v.name,
        &v.slug,
        v.business_name.as_deref(),
        v.logo_url.as_deref(),
        v.description.as_deref(),
        v.phone.as_deref(),
        v.email.as_deref(),
        v.website.as_deref(),
        v.address.as_deref(),
        v.city.as_deref(),
        v.country.as_deref(),
        v.latitude.map(|l| l as f32),
        v.longitude.map(|l| l as f32),
        v.timezone.as_deref(),
        v.base_currency.as_deref().unwrap_or("KES"),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"organization": r}))))
}
async fn update_organization(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<UpdateOrganization>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "write")?;
    Ok(Json(
        json!({"organization": controller::marketplace::update_organization(&s.pool, &id, &v)?}),
    ))
}
async fn delete_organization(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "marketplace", "admin")?;
    controller::marketplace::delete_organization(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// --------------------------------------------------------------- org members ---

async fn list_org_members(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(org_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"members": controller::marketplace::list_org_members(&s.pool, &org_id)?}),
    ))
}
#[derive(Deserialize)]
struct AddMember {
    user_id: String,
    role: String,
}
async fn add_org_member(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(org_id): Path<String>,
    Json(v): Json<AddMember>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "marketplace", "write")?;
    let r = controller::marketplace::add_org_member(&s.pool, &org_id, &v.user_id, &v.role)?;
    Ok((StatusCode::CREATED, Json(json!({"member": r}))))
}
async fn update_org_member(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<UpdateOrgMember>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "write")?;
    Ok(Json(
        json!({"member": controller::marketplace::update_org_member(&s.pool, &id, &v)?}),
    ))
}
async fn remove_org_member(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "marketplace", "write")?;
    controller::marketplace::remove_org_member(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// --------------------------------------------------------------- categories ---

async fn list_categories(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"categories": controller::marketplace::list_categories(&s.pool)?}),
    ))
}
async fn get_category(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"category": controller::marketplace::get_category(&s.pool, &id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateCat {
    name: String,
    slug: String,
    description: Option<String>,
    icon_url: Option<String>,
    parent_id: Option<String>,
    sort_order: Option<i32>,
}
async fn create_category(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<CreateCat>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "marketplace", "write")?;
    let r = controller::marketplace::create_category(
        &s.pool,
        &v.name,
        &v.slug,
        v.description.as_deref(),
        v.icon_url.as_deref(),
        v.parent_id.as_deref(),
        v.sort_order.unwrap_or(0),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"category": r}))))
}
async fn update_category(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<UpdateCategory>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "write")?;
    Ok(Json(
        json!({"category": controller::marketplace::update_category(&s.pool, &id, &v)?}),
    ))
}
async fn delete_category(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "marketplace", "admin")?;
    controller::marketplace::delete_category(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// --------------------------------------------------------------- services ---

async fn list_services(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"services": controller::marketplace::list_services(&s.pool)?}),
    ))
}
async fn list_services_by_org(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(org_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"services": controller::marketplace::list_services_by_org(&s.pool, &org_id)?}),
    ))
}
async fn get_service(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"service": controller::marketplace::get_service(&s.pool, &id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateSvc {
    org_id: String,
    category_id: Option<String>,
    name: String,
    slug: String,
    description: Option<String>,
    short_description: Option<String>,
    cover_image_url: Option<String>,
    price_type: Option<String>,
    base_price: f64,
    currency: Option<String>,
    duration_minutes: Option<i32>,
}
async fn create_service(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<CreateSvc>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "marketplace", "write")?;
    let r = controller::marketplace::create_service(
        &s.pool,
        &v.org_id,
        v.category_id.as_deref(),
        &v.name,
        &v.slug,
        v.description.as_deref(),
        v.short_description.as_deref(),
        v.cover_image_url.as_deref(),
        v.price_type.as_deref().unwrap_or("fixed"),
        v.base_price as f32,
        v.currency.as_deref().unwrap_or("KES"),
        v.duration_minutes,
    )?;
    Ok((StatusCode::CREATED, Json(json!({"service": r}))))
}
async fn update_service(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<UpdateService>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "write")?;
    Ok(Json(
        json!({"service": controller::marketplace::update_service(&s.pool, &id, &v)?}),
    ))
}
async fn delete_service(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "marketplace", "write")?;
    controller::marketplace::delete_service(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// --------------------------------------------------------------- zones ---

async fn list_zones(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(org_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"zones": controller::marketplace::list_zones(&s.pool, &org_id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateZone {
    name: String,
    latitude: Option<f64>,
    longitude: Option<f64>,
    radius_km: Option<f64>,
    city: Option<String>,
    country: Option<String>,
}
async fn create_zone(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(org_id): Path<String>,
    Json(v): Json<CreateZone>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "marketplace", "write")?;
    let r = controller::marketplace::create_zone(
        &s.pool,
        &org_id,
        &v.name,
        v.latitude,
        v.longitude,
        v.radius_km,
        v.city.as_deref(),
        v.country.as_deref(),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"zone": r}))))
}
async fn delete_zone(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "marketplace", "write")?;
    controller::marketplace::delete_zone(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// --------------------------------------------------------------- bookings ---

async fn list_bookings(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"bookings": controller::marketplace::list_bookings(&s.pool)?}),
    ))
}
async fn list_bookings_by_org(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(org_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"bookings": controller::marketplace::list_bookings_by_org(&s.pool, &org_id)?}),
    ))
}
async fn list_bookings_by_customer(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(user_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"bookings": controller::marketplace::list_bookings_by_customer(&s.pool, &user_id)?}),
    ))
}
async fn get_booking(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"booking": controller::marketplace::get_booking(&s.pool, &id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateBk {
    org_id: String,
    service_id: String,
    customer_user_id: String,
    booking_type: Option<String>,
    scheduled_at: Option<String>,
    address: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    notes: Option<String>,
    total_price: f64,
    currency: Option<String>,
    commission_amount: Option<f64>,
    provider_payout: Option<f64>,
    payment_method: Option<String>,
    promo_id: Option<String>,
}
async fn create_booking(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<CreateBk>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "marketplace", "write")?;
    let sched = v
        .scheduled_at
        .as_deref()
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok());
    let r = controller::marketplace::create_booking(
        &s.pool,
        &v.org_id,
        &v.service_id,
        &v.customer_user_id,
        v.booking_type.as_deref().unwrap_or("direct"),
        sched,
        v.address.as_deref(),
        v.latitude,
        v.longitude,
        v.notes.as_deref(),
        v.total_price,
        v.currency.as_deref().unwrap_or("KES"),
        v.commission_amount.unwrap_or(0.0),
        v.provider_payout.unwrap_or(v.total_price),
        v.payment_method.as_deref(),
        v.promo_id.as_deref(),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"booking": r}))))
}
async fn update_booking(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<UpdateBooking>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "write")?;
    Ok(Json(
        json!({"booking": controller::marketplace::update_booking(&s.pool, &id, &v)?}),
    ))
}
async fn delete_booking(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "marketplace", "write")?;
    controller::marketplace::delete_booking(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// --------------------------------------------------------------- requests ---

async fn list_requests(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"requests": controller::marketplace::list_requests(&s.pool)?}),
    ))
}
async fn get_request(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"request": controller::marketplace::get_request(&s.pool, &id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateReq {
    customer_user_id: String,
    category_id: Option<String>,
    title: String,
    description: Option<String>,
    address: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    budget_min: Option<f64>,
    budget_max: Option<f64>,
    currency: Option<String>,
    preferred_date: Option<String>,
    expires_at: Option<String>,
}
async fn create_request(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<CreateReq>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "marketplace", "write")?;
    let exp = v
        .expires_at
        .as_deref()
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok());
    let r = controller::marketplace::create_request(
        &s.pool,
        &v.customer_user_id,
        v.category_id.as_deref(),
        &v.title,
        v.description.as_deref(),
        v.address.as_deref(),
        v.latitude,
        v.longitude,
        v.budget_min,
        v.budget_max,
        v.currency.as_deref().unwrap_or("KES"),
        v.preferred_date.as_deref(),
        exp,
    )?;
    Ok((StatusCode::CREATED, Json(json!({"request": r}))))
}
async fn delete_request(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "marketplace", "write")?;
    controller::marketplace::delete_request(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// --------------------------------------------------------------- bids ---

async fn list_bids(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(request_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"bids": controller::marketplace::list_bids(&s.pool, &request_id)?}),
    ))
}
async fn get_bid(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"bid": controller::marketplace::get_bid(&s.pool, &id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateBid {
    org_id: String,
    price: f64,
    currency: Option<String>,
    message: Option<String>,
    estimated_days: Option<i32>,
}
async fn create_bid(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(request_id): Path<String>,
    Json(v): Json<CreateBid>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "marketplace", "write")?;
    let r = controller::marketplace::create_bid(
        &s.pool,
        &request_id,
        &v.org_id,
        v.price,
        v.currency.as_deref().unwrap_or("KES"),
        v.message.as_deref(),
        v.estimated_days,
    )?;
    Ok((StatusCode::CREATED, Json(json!({"bid": r}))))
}
async fn accept_bid(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "write")?;
    Ok(Json(
        json!({"bid": controller::marketplace::accept_bid(&s.pool, &id)?}),
    ))
}
async fn delete_bid(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "marketplace", "write")?;
    controller::marketplace::delete_bid(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// --------------------------------------------------------------- cart ---

async fn list_cart(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(user_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"cart": controller::marketplace::list_cart(&s.pool, &user_id)?}),
    ))
}
#[derive(Deserialize)]
struct AddCart {
    user_id: String,
    service_id: String,
    quantity: Option<i32>,
    notes: Option<String>,
}
async fn add_to_cart_item(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<AddCart>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "marketplace", "write")?;
    let r = controller::marketplace::add_to_cart(
        &s.pool,
        &v.user_id,
        &v.service_id,
        v.quantity.unwrap_or(1),
        v.notes.as_deref(),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"cart_item": r}))))
}
async fn update_cart_item(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<UpdateCartItem>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "write")?;
    Ok(Json(
        json!({"cart_item": controller::marketplace::update_cart_item(&s.pool, &id, &v)?}),
    ))
}
async fn remove_cart_item(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "marketplace", "write")?;
    controller::marketplace::remove_cart_item(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn clear_cart_items(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(user_id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "marketplace", "write")?;
    controller::marketplace::clear_cart(&s.pool, &user_id)?;
    Ok(StatusCode::NO_CONTENT)
}

// --------------------------------------------------------------- reviews ---

async fn list_org_reviews(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(org_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"reviews": controller::marketplace::list_reviews_by_org(&s.pool, &org_id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateRev {
    booking_id: String,
    service_id: Option<String>,
    customer_user_id: String,
    rating: i32,
    comment: Option<String>,
}
async fn create_review(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(org_id): Path<String>,
    Json(v): Json<CreateRev>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "marketplace", "write")?;
    let r = controller::marketplace::create_review(
        &s.pool,
        &v.booking_id,
        &org_id,
        v.service_id.as_deref(),
        &v.customer_user_id,
        v.rating,
        v.comment.as_deref(),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"review": r}))))
}

// --------------------------------------------------------------- commissions ---

async fn list_org_commissions(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(org_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"commissions": controller::marketplace::list_commissions(&s.pool, &org_id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateComm {
    booking_id: String,
    org_id: String,
    amount: f64,
    rate: f64,
    currency: Option<String>,
}
async fn create_commission_entry(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<CreateComm>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "marketplace", "admin")?;
    let r = controller::marketplace::create_commission(
        &s.pool,
        &v.booking_id,
        &v.org_id,
        v.amount,
        v.rate,
        v.currency.as_deref().unwrap_or("KES"),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"commission": r}))))
}

// --------------------------------------------------------------- payouts ---

async fn list_all_payouts(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "admin")?;
    Ok(Json(
        json!({"payouts": controller::marketplace::list_all_payouts(&s.pool)?}),
    ))
}
async fn list_org_payouts(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(org_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"payouts": controller::marketplace::list_payouts(&s.pool, &org_id)?}),
    ))
}
#[derive(Deserialize)]
struct CreatePay {
    org_id: String,
    amount: f64,
    currency: Option<String>,
    method: String,
    reference: Option<String>,
}
async fn create_payout_entry(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<CreatePay>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "marketplace", "admin")?;
    let r = controller::marketplace::create_payout(
        &s.pool,
        &v.org_id,
        v.amount,
        v.currency.as_deref().unwrap_or("KES"),
        &v.method,
        v.reference.as_deref(),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"payout": r}))))
}
async fn complete_payout_entry(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "admin")?;
    Ok(Json(
        json!({"payout": controller::marketplace::complete_payout(&s.pool, &id)?}),
    ))
}

// --------------------------------------------------------------- promotions ---

async fn list_promotions(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"promotions": controller::marketplace::list_promotions(&s.pool)?}),
    ))
}
async fn get_promotion_entry(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"promotion": controller::marketplace::get_promotion(&s.pool, &id)?}),
    ))
}
#[derive(Deserialize)]
struct CreatePromo {
    org_id: Option<String>,
    code: String,
    description: Option<String>,
    discount_type: String,
    discount_value: f64,
    min_booking_value: Option<f64>,
    max_uses: Option<i32>,
    starts_at: String,
    expires_at: String,
}
async fn create_promotion_entry(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<CreatePromo>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "marketplace", "write")?;
    let start = chrono::NaiveDateTime::parse_from_str(&v.starts_at, "%Y-%m-%dT%H:%M:%S")
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;
    let exp = chrono::NaiveDateTime::parse_from_str(&v.expires_at, "%Y-%m-%dT%H:%M:%S")
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;
    let r = controller::marketplace::create_promotion(
        &s.pool,
        v.org_id.as_deref(),
        &v.code,
        v.description.as_deref(),
        &v.discount_type,
        v.discount_value as f32,
        v.min_booking_value.map(|m| m as f32),
        v.max_uses,
        start,
        exp,
    )?;
    Ok((StatusCode::CREATED, Json(json!({"promotion": r}))))
}
async fn update_promotion_entry(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<UpdatePromotion>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "write")?;
    Ok(Json(
        json!({"promotion": controller::marketplace::update_promotion(&s.pool, &id, &v)?}),
    ))
}
async fn delete_promotion_entry(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "marketplace", "admin")?;
    controller::marketplace::delete_promotion(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}
#[derive(Deserialize)]
struct ValidatePromo {
    code: String,
}
async fn validate_promo(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<ValidatePromo>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"promotion": controller::marketplace::validate_promo_code(&s.pool, &v.code)?}),
    ))
}

// --------------------------------------------------------------- notifications ---

async fn list_my_notifications(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"notifications": controller::marketplace::list_notifications(&s.pool, &c.sub)?}),
    ))
}
async fn list_user_notifications(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(user_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "marketplace", "read")?;
    Ok(Json(
        json!({"notifications": controller::marketplace::list_notifications(&s.pool, &user_id)?}),
    ))
}
async fn mark_read(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "marketplace", "write")?;
    controller::marketplace::mark_notification_read(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn mark_all_read(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(user_id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "marketplace", "write")?;
    controller::marketplace::mark_all_notifications_read(&s.pool, &user_id)?;
    Ok(StatusCode::NO_CONTENT)
}
