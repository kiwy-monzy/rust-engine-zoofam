use crate::{
    middleware::{ApiError, ApiResult},
    state::AppState,
};
use auth::Claims;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Extension, Json, Router,
};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use models::schema::*;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

fn need(c: &Claims, m: &str, a: &str) -> Result<(), ApiError> {
    c.require(m, a)
        .map_err(|e| ApiError::new(StatusCode::FORBIDDEN, e.to_string()))
}

pub fn website_routes() -> Router<AppState> {
    Router::new()
        // templates
        .route(
            "/website/templates",
            get(list_templates).post(create_template),
        )
        .route(
            "/website/templates/:id",
            get(get_template)
                .patch(update_template)
                .delete(delete_template),
        )
        // template preview (assembled)
        .route("/website/templates/:id/preview", get(preview_template))
        // sections (kind unique per template)
        .route(
            "/website/templates/:id/sections",
            get(list_sections).post(create_section),
        )
        .route(
            "/website/templates/:id/sections/:kind",
            get(get_section_by_kind)
                .patch(patch_section_by_kind)
                .delete(delete_section_by_kind),
        )
        // links
        .route(
            "/website/templates/:id/links",
            get(list_links).post(create_link),
        )
        .route(
            "/website/templates/:id/links/:link_id",
            get(get_link).patch(update_link).delete(delete_link),
        )
        // cart (per-user per-template)
        .route(
            "/website/templates/:id/cart",
            get(list_cart).post(add_cart_item),
        )
        .route(
            "/website/templates/:id/cart/:item_id",
            delete(delete_cart_item),
        )
        .route("/website/templates/:id/cart/checkout", post(checkout_cart))
        // bookings
        .route(
            "/website/templates/:id/bookings",
            get(list_bookings).post(create_booking),
        )
        .route(
            "/website/templates/:id/bookings/:booking_id",
            get(get_booking)
                .patch(update_booking)
                .delete(delete_booking),
        )
        .route(
            "/website/templates/:id/bookings/:booking_id/confirm",
            post(confirm_booking),
        )
}

// ---------------------------------------------------------------- templates --

async fn list_templates(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "website", "read")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let rows: Vec<models::website::WebsiteTemplate> = website_templates::table
        .load(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json!({"templates": rows})))
}
async fn get_template(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "website", "read")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let r: models::website::WebsiteTemplate = website_templates::table
        .find(id.as_str())
        .first(&mut conn)
        .map_err(|_| ApiError::new(StatusCode::NOT_FOUND, "template not found"))?;
    Ok(Json(json!({"template": r})))
}
#[derive(Deserialize)]
struct CreateTemplate {
    slug: String,
    name: String,
    version: Option<String>,
    is_active: Option<bool>,
    theme_json: Option<String>,
}
async fn create_template(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<CreateTemplate>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "website", "write")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let id = Uuid::new_v4().to_string();
    let row = models::website::NewWebsiteTemplate {
        id: id.clone(),
        slug: v.slug,
        name: v.name,
        version: v.version.unwrap_or_else(|| "1.0.0".into()),
        is_active: v.is_active.map(|b| if b { 1 } else { 0 }).unwrap_or(1),
        theme_json: v.theme_json,
    };
    diesel::insert_into(website_templates::table)
        .values(&row)
        .execute(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;
    let r: models::website::WebsiteTemplate = website_templates::table
        .find(id.as_str())
        .first(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(json!({"template": r}))))
}
async fn update_template(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<models::website::UpdateWebsiteTemplate>,
) -> ApiResult<Json<Value>> {
    need(&c, "website", "write")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    diesel::update(website_templates::table.find(id.as_str()))
        .set(&v)
        .execute(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;
    let r: models::website::WebsiteTemplate = website_templates::table
        .find(id.as_str())
        .first(&mut conn)
        .map_err(|_| ApiError::new(StatusCode::NOT_FOUND, "template not found"))?;
    Ok(Json(json!({"template": r})))
}
async fn delete_template(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "website", "write")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    diesel::delete(website_templates::table.find(id.as_str()))
        .execute(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}
async fn preview_template(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "website", "read")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let tpl: models::website::WebsiteTemplate = website_templates::table
        .find(id.as_str())
        .first(&mut conn)
        .map_err(|_| ApiError::new(StatusCode::NOT_FOUND, "template not found"))?;
    let sections: Vec<models::website::WebsiteSection> = website_sections::table
        .filter(website_sections::template_id.eq(id.as_str()))
        .load(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let links: Vec<models::website::WebsiteLink> = website_links::table
        .filter(website_links::template_id.eq(id.as_str()))
        .order(website_links::position.asc())
        .load(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(
        json!({"template": tpl, "sections": sections, "links": links}),
    ))
}

// ---------------------------------------------------------------- sections --

async fn list_sections(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "website", "read")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let rows: Vec<models::website::WebsiteSection> = website_sections::table
        .filter(website_sections::template_id.eq(id.as_str()))
        .order(website_sections::position.asc())
        .load(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json!({"sections": rows})))
}
async fn get_section_by_kind(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path((id, kind)): Path<(String, String)>,
) -> ApiResult<Json<Value>> {
    need(&c, "website", "read")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let r: models::website::WebsiteSection = website_sections::table
        .filter(website_sections::template_id.eq(id.as_str()))
        .filter(website_sections::kind.eq(kind.as_str()))
        .first(&mut conn)
        .map_err(|_| ApiError::new(StatusCode::NOT_FOUND, "section not found"))?;
    Ok(Json(json!({"section": r})))
}
#[derive(Deserialize)]
struct CreateSection {
    kind: String,
    position: Option<i32>,
    config_json: Option<String>,
    is_visible: Option<bool>,
}
async fn create_section(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<CreateSection>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "website", "write")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    // ensure template exists
    let _: models::website::WebsiteTemplate = website_templates::table
        .find(id.as_str())
        .first(&mut conn)
        .map_err(|_| ApiError::new(StatusCode::NOT_FOUND, "template not found"))?;
    let sid = Uuid::new_v4().to_string();
    let row = models::website::NewWebsiteSection {
        id: sid.clone(),
        template_id: id.clone(),
        kind: v.kind,
        position: v.position.unwrap_or(0),
        config_json: v.config_json.unwrap_or_else(|| "{}".into()),
        is_visible: v.is_visible.map(|b| if b { 1 } else { 0 }).unwrap_or(1),
    };
    diesel::insert_into(website_sections::table)
        .values(&row)
        .execute(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;
    let r: models::website::WebsiteSection = website_sections::table
        .find(sid.as_str())
        .first(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(json!({"section": r}))))
}
async fn patch_section_by_kind(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path((id, kind)): Path<(String, String)>,
    Json(v): Json<models::website::UpdateWebsiteSection>,
) -> ApiResult<Json<Value>> {
    need(&c, "website", "write")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    diesel::update(
        website_sections::table
            .filter(website_sections::template_id.eq(id.as_str()))
            .filter(website_sections::kind.eq(kind.as_str())),
    )
    .set(&v)
    .execute(&mut conn)
    .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;
    let r: models::website::WebsiteSection = website_sections::table
        .filter(website_sections::template_id.eq(id.as_str()))
        .filter(website_sections::kind.eq(kind.as_str()))
        .first(&mut conn)
        .map_err(|_| ApiError::new(StatusCode::NOT_FOUND, "section not found"))?;
    Ok(Json(json!({"section": r})))
}
async fn delete_section_by_kind(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path((id, kind)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    need(&c, "website", "write")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    diesel::delete(
        website_sections::table
            .filter(website_sections::template_id.eq(id.as_str()))
            .filter(website_sections::kind.eq(kind.as_str())),
    )
    .execute(&mut conn)
    .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------- links --

async fn list_links(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "website", "read")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let rows: Vec<models::website::WebsiteLink> = website_links::table
        .filter(website_links::template_id.eq(id.as_str()))
        .order(website_links::position.asc())
        .load(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json!({"links": rows})))
}
async fn get_link(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path((id, link_id)): Path<(String, String)>,
) -> ApiResult<Json<Value>> {
    need(&c, "website", "read")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let r: models::website::WebsiteLink = website_links::table
        .find(link_id.as_str())
        .first(&mut conn)
        .map_err(|_| ApiError::new(StatusCode::NOT_FOUND, "link not found"))?;
    if r.template_id != id {
        return Err(ApiError::new(StatusCode::NOT_FOUND, "link not found"));
    }
    Ok(Json(json!({"link": r})))
}
#[derive(Deserialize)]
struct CreateLink {
    label: String,
    href: String,
    icon: Option<String>,
    position: Option<i32>,
    is_visible: Option<bool>,
}
async fn create_link(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<CreateLink>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "website", "write")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let lid = Uuid::new_v4().to_string();
    let row = models::website::NewWebsiteLink {
        id: lid.clone(),
        template_id: id.clone(),
        label: v.label,
        href: v.href,
        icon: v.icon,
        position: v.position.unwrap_or(0),
        is_visible: v.is_visible.map(|b| if b { 1 } else { 0 }).unwrap_or(1),
    };
    diesel::insert_into(website_links::table)
        .values(&row)
        .execute(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;
    let r: models::website::WebsiteLink = website_links::table
        .find(lid.as_str())
        .first(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(json!({"link": r}))))
}
async fn update_link(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path((id, link_id)): Path<(String, String)>,
    Json(v): Json<models::website::UpdateWebsiteLink>,
) -> ApiResult<Json<Value>> {
    need(&c, "website", "write")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    diesel::update(website_links::table.find(link_id.as_str()))
        .set(&v)
        .execute(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;
    let r: models::website::WebsiteLink = website_links::table
        .find(link_id.as_str())
        .first(&mut conn)
        .map_err(|_| ApiError::new(StatusCode::NOT_FOUND, "link not found"))?;
    if r.template_id != id {
        return Err(ApiError::new(StatusCode::NOT_FOUND, "link not found"));
    }
    Ok(Json(json!({"link": r})))
}
async fn delete_link(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path((id, link_id)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    need(&c, "website", "write")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    diesel::delete(website_links::table.find(link_id.as_str()))
        .execute(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let _ = id;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------- cart --

async fn list_cart(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "website", "read")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let user = c.sub.clone();
    let items: Vec<models::website::WebsiteCartItem> = website_cart_items::table
        .filter(website_cart_items::template_id.eq(id.as_str()))
        .filter(website_cart_items::user_id.eq(user.as_str()))
        .load(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    // enrich with product names
    let mut out: Vec<Value> = Vec::new();
    for it in items {
        let prod: Option<models::erp::Product> = erp_products::table
            .find(it.product_id.as_str())
            .select(models::erp::Product::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        out.push(json!({
            "id": it.id,
            "template_id": it.template_id,
            "user_id": it.user_id,
            "product_id": it.product_id,
            "quantity": it.quantity,
            "unit_price": it.unit_price,
            "product_name": prod.as_ref().map(|p| p.name.clone()),
            "sku": prod.as_ref().map(|p| p.sku.clone()),
            "added_at": it.added_at
        }));
    }
    Ok(Json(json!({"items": out})))
}
#[derive(Deserialize)]
struct AddCart {
    product_id: String,
    quantity: Option<f64>,
}
async fn add_cart_item(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<AddCart>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "website", "write")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let user = c.sub.clone();
    // fetch product for price (use 0 if not priced) — products have no price column, default to 0
    let _prod: models::erp::Product = erp_products::table
        .find(v.product_id.as_str())
        .select(models::erp::Product::as_select())
        .first(&mut conn)
        .map_err(|_| ApiError::new(StatusCode::NOT_FOUND, "product not found"))?;
    let iid = Uuid::new_v4().to_string();
    let row = models::website::NewWebsiteCartItem {
        id: iid.clone(),
        template_id: id.clone(),
        user_id: user,
        product_id: v.product_id,
        quantity: v.quantity.map(|q| q as f32).unwrap_or(1.0f32),
        unit_price: 0.0f32,
    };
    diesel::insert_into(website_cart_items::table)
        .values(&row)
        .execute(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;
    let r: models::website::WebsiteCartItem = website_cart_items::table
        .find(iid.as_str())
        .first(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(json!({"item": r}))))
}
async fn delete_cart_item(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path((id, item_id)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    need(&c, "website", "write")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let user = c.sub.clone();
    let target: models::website::WebsiteCartItem = website_cart_items::table
        .find(item_id.as_str())
        .first(&mut conn)
        .map_err(|_| ApiError::new(StatusCode::NOT_FOUND, "item not found"))?;
    if target.template_id != id || target.user_id != user {
        return Err(ApiError::new(StatusCode::FORBIDDEN, "not your cart item"));
    }
    diesel::delete(website_cart_items::table.find(item_id.as_str()))
        .execute(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}
async fn checkout_cart(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "website", "write")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let user = c.sub.clone();
    let items: Vec<models::website::WebsiteCartItem> = website_cart_items::table
        .filter(website_cart_items::template_id.eq(id.as_str()))
        .filter(website_cart_items::user_id.eq(user.as_str()))
        .load(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if items.is_empty() {
        return Err(ApiError::new(StatusCode::BAD_REQUEST, "cart is empty"));
    }
    // create a sales order that groups the checkout
    let so_id = Uuid::new_v4().to_string();
    let so_number = format!("WSO-{}", &so_id[0..8].to_uppercase());
    diesel::insert_into(erp_sales_orders::table)
        .values((
            erp_sales_orders::id.eq(so_id.clone()),
            erp_sales_orders::so_number.eq(so_number.clone()),
            erp_sales_orders::customer_name.eq(user.clone()),
            erp_sales_orders::status.eq("PENDING"),
            erp_sales_orders::notes.eq(format!("Website checkout template={}", id)),
        ))
        .execute(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    for it in &items {
        let lid = Uuid::new_v4().to_string();
        diesel::insert_into(erp_sales_lines::table)
            .values((
                erp_sales_lines::id.eq(lid),
                erp_sales_lines::sales_order_id.eq(so_id.clone()),
                erp_sales_lines::product_id.eq(it.product_id.clone()),
                erp_sales_lines::quantity.eq(it.quantity),
                erp_sales_lines::unit_price.eq(it.unit_price),
            ))
            .execute(&mut conn)
            .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    diesel::delete(
        website_cart_items::table
            .filter(website_cart_items::template_id.eq(id.as_str()))
            .filter(website_cart_items::user_id.eq(user.as_str())),
    )
    .execute(&mut conn)
    .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json!({"order_id": so_id, "so_number": so_number})))
}

// ---------------------------------------------------------------- bookings --

async fn list_bookings(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "website", "read")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let rows: Vec<models::website::WebsiteBooking> = website_bookings::table
        .filter(website_bookings::template_id.eq(id.as_str()))
        .order(website_bookings::date.desc())
        .load(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json!({"bookings": rows})))
}
async fn get_booking(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path((id, booking_id)): Path<(String, String)>,
) -> ApiResult<Json<Value>> {
    need(&c, "website", "read")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let r: models::website::WebsiteBooking = website_bookings::table
        .find(booking_id.as_str())
        .first(&mut conn)
        .map_err(|_| ApiError::new(StatusCode::NOT_FOUND, "booking not found"))?;
    if r.template_id != id {
        return Err(ApiError::new(StatusCode::NOT_FOUND, "booking not found"));
    }
    Ok(Json(json!({"booking": r})))
}
#[derive(Deserialize)]
struct CreateBooking {
    customer_name: String,
    customer_email: Option<String>,
    service: Option<String>,
    product_id: Option<String>,
    date: Option<String>,
    status: Option<String>,
    notes: Option<String>,
}
fn parse_date(s: &str) -> Result<NaiveDateTime, ApiError> {
    // accept ISO datetime or date-only
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(dt);
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%SZ") {
        return Ok(dt);
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(d.and_hms_opt(0, 0, 0).unwrap());
    }
    if let Ok(dt) = s.parse::<chrono::DateTime<chrono::Utc>>() {
        return Ok(dt.naive_utc());
    }
    Err(ApiError::new(
        StatusCode::BAD_REQUEST,
        "invalid date, use YYYY-MM-DD or ISO datetime",
    ))
}
async fn create_booking(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<CreateBooking>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "website", "write")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let bid = Uuid::new_v4().to_string();
    let dt = if let Some(ref d) = v.date {
        parse_date(d)?
    } else {
        chrono::Utc::now().naive_utc()
    };
    let row = models::website::NewWebsiteBooking {
        id: bid.clone(),
        template_id: id.clone(),
        user_id: Some(c.sub.clone()),
        customer_name: v.customer_name,
        customer_email: v.customer_email,
        service: v.service,
        product_id: v.product_id,
        date: dt,
        status: v.status.unwrap_or_else(|| "PENDING".into()),
        notes: v.notes,
    };
    diesel::insert_into(website_bookings::table)
        .values(&row)
        .execute(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;
    let r: models::website::WebsiteBooking = website_bookings::table
        .find(bid.as_str())
        .first(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(json!({"booking": r}))))
}
async fn update_booking(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path((id, booking_id)): Path<(String, String)>,
    Json(v): Json<models::website::UpdateWebsiteBooking>,
) -> ApiResult<Json<Value>> {
    need(&c, "website", "write")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    diesel::update(website_bookings::table.find(booking_id.as_str()))
        .set(&v)
        .execute(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;
    let r: models::website::WebsiteBooking = website_bookings::table
        .find(booking_id.as_str())
        .first(&mut conn)
        .map_err(|_| ApiError::new(StatusCode::NOT_FOUND, "booking not found"))?;
    if r.template_id != id {
        return Err(ApiError::new(StatusCode::NOT_FOUND, "booking not found"));
    }
    Ok(Json(json!({"booking": r})))
}
async fn delete_booking(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path((id, booking_id)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    need(&c, "website", "write")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let r: models::website::WebsiteBooking = website_bookings::table
        .find(booking_id.as_str())
        .first(&mut conn)
        .map_err(|_| ApiError::new(StatusCode::NOT_FOUND, "booking not found"))?;
    if r.template_id != id {
        return Err(ApiError::new(StatusCode::NOT_FOUND, "booking not found"));
    }
    diesel::delete(website_bookings::table.find(booking_id.as_str()))
        .execute(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}
async fn confirm_booking(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path((id, booking_id)): Path<(String, String)>,
) -> ApiResult<Json<Value>> {
    need(&c, "website", "write")?;
    let mut conn = db::conn(&s.pool)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    diesel::update(website_bookings::table.find(booking_id.as_str()))
        .set(website_bookings::status.eq("CONFIRMED"))
        .execute(&mut conn)
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, e.to_string()))?;
    let r: models::website::WebsiteBooking = website_bookings::table
        .find(booking_id.as_str())
        .first(&mut conn)
        .map_err(|_| ApiError::new(StatusCode::NOT_FOUND, "booking not found"))?;
    if r.template_id != id {
        return Err(ApiError::new(StatusCode::NOT_FOUND, "booking not found"));
    }
    Ok(Json(json!({"booking": r})))
}
