use crate::{
    middleware::{ApiError, ApiResult},
    state::AppState,
};
use auth::Claims;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use serde_json::{json, Value};

fn need(c: &Claims, m: &str, a: &str) -> Result<(), ApiError> {
    c.require(m, a)
        .map_err(|e| ApiError::new(StatusCode::FORBIDDEN, e.to_string()))
}

pub fn crm_routes() -> Router<AppState> {
    Router::new()
        .route("/crm/leads", get(list_leads).post(create_lead))
        .route(
            "/crm/leads/:id",
            get(get_lead).patch(update_lead).delete(delete_lead),
        )
        .route("/crm/opportunities", get(list_opps).post(create_opp))
        .route("/crm/opportunities/:id", get(get_opp).delete(delete_opp))
        .route("/crm/customers", get(list_customers).post(create_customer))
        .route(
            "/crm/customers/:id",
            get(get_customer).delete(delete_customer),
        )
        .route("/crm/contacts", get(list_contacts).post(create_contact))
        .route("/crm/contacts/:id", get(get_contact).delete(delete_contact))
        .route(
            "/crm/activities",
            get(list_activities).post(create_activity),
        )
        .route(
            "/crm/activities/:id",
            get(get_activity).delete(delete_activity),
        )
        .route("/crm/quotes", get(list_quotes).post(create_quote))
        .route("/crm/quotes/:id", get(get_quote).delete(delete_quote))
        .route(
            "/crm/quotes/:id/lines",
            get(list_quote_lines).post(create_quote_line),
        )
        .route("/crm/quotes/:id/accept", post(accept_quote))
        .route(
            "/accounting/invoices",
            get(list_invoices).post(create_invoice),
        )
        .route("/accounting/invoices/:id", get(get_invoice))
}

async fn list_leads(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "crm", "read")?;
    Ok(Json(json!({"leads": controller::crm::leads(&s.pool)?})))
}
async fn get_lead(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "crm", "read")?;
    Ok(Json(
        json!({"lead": controller::crm::lead_get(&s.pool,&id)?}),
    ))
}
async fn create_lead(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<models::crm::NewLead>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "crm", "write")?;
    Ok((
        StatusCode::CREATED,
        Json(json!({"lead": controller::crm::lead_create(&s.pool,v)?})),
    ))
}
async fn update_lead(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<models::crm::UpdateLead>,
) -> ApiResult<Json<Value>> {
    need(&c, "crm", "write")?;
    Ok(Json(
        json!({"lead": controller::crm::lead_update_full(&s.pool,&id,v)?}),
    ))
}
async fn delete_lead(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "crm", "write")?;
    controller::crm::lead_delete(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_opps(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "crm", "read")?;
    Ok(Json(
        json!({"opportunities": controller::crm::opportunities(&s.pool)?}),
    ))
}
async fn get_opp(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "crm", "read")?;
    Ok(Json(
        json!({"opportunity": controller::crm::opp_get(&s.pool,&id)?}),
    ))
}
async fn create_opp(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<models::crm::NewOpportunity>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "crm", "write")?;
    Ok((
        StatusCode::CREATED,
        Json(json!({"opportunity": controller::crm::opp_create(&s.pool,v)?})),
    ))
}
async fn delete_opp(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "crm", "write")?;
    controller::crm::opp_delete(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_customers(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "crm", "read")?;
    Ok(Json(
        json!({"customers": controller::crm::customers(&s.pool)?}),
    ))
}
async fn get_customer(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "crm", "read")?;
    Ok(Json(
        json!({"customer": controller::crm::customer_get(&s.pool,&id)?}),
    ))
}
async fn create_customer(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<models::crm::NewCustomer>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "crm", "write")?;
    Ok((
        StatusCode::CREATED,
        Json(json!({"customer": controller::crm::customer_create(&s.pool,v)?})),
    ))
}
async fn delete_customer(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "crm", "write")?;
    controller::crm::customer_delete(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_contacts(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "crm", "read")?;
    Ok(Json(
        json!({"contacts": controller::crm::contacts(&s.pool)?}),
    ))
}
async fn get_contact(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "crm", "read")?;
    Ok(Json(
        json!({"contact": controller::crm::contact_get(&s.pool,&id)?}),
    ))
}
async fn create_contact(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<models::crm::NewContact>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "crm", "write")?;
    Ok((
        StatusCode::CREATED,
        Json(json!({"contact": controller::crm::contact_create(&s.pool,v)?})),
    ))
}
async fn delete_contact(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "crm", "write")?;
    controller::crm::contact_delete(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_activities(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "crm", "read")?;
    Ok(Json(
        json!({"activities": controller::crm::activities(&s.pool)?}),
    ))
}
async fn get_activity(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "crm", "read")?;
    Ok(Json(
        json!({"activity": controller::crm::activity_get(&s.pool,&id)?}),
    ))
}
async fn create_activity(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<models::crm::NewActivity>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "crm", "write")?;
    Ok((
        StatusCode::CREATED,
        Json(json!({"activity": controller::crm::activity_create(&s.pool,v)?})),
    ))
}
async fn delete_activity(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "crm", "write")?;
    controller::crm::activity_delete(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_quotes(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "crm", "read")?;
    Ok(Json(json!({"quotes": controller::crm::quotes(&s.pool)?})))
}
async fn get_quote(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "crm", "read")?;
    Ok(Json(
        json!({"quote": controller::crm::quote_get(&s.pool,&id)?}),
    ))
}
async fn create_quote(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<models::crm::NewQuote>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "crm", "write")?;
    Ok((
        StatusCode::CREATED,
        Json(json!({"quote": controller::crm::quote_create(&s.pool,v)?})),
    ))
}
async fn delete_quote(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "crm", "write")?;
    controller::crm::quote_delete(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn list_quote_lines(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "crm", "read")?;
    Ok(Json(
        json!({"lines": controller::crm::quote_lines(&s.pool,&id)?}),
    ))
}
async fn create_quote_line(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(mut v): Json<models::crm::NewQuoteLine>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "crm", "write")?;
    v.quote_id = id.clone();
    Ok((
        StatusCode::CREATED,
        Json(json!({"line": controller::crm::quote_line_create(&s.pool,v)?})),
    ))
}
async fn accept_quote(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "crm", "write")?;
    let so = controller::crm::quote_accept(&s.pool, &id)?;
    Ok(Json(
        json!({"sales_order": so, "message":"Quote accepted -> Sales Order created"}),
    ))
}

async fn list_invoices(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "crm", "read")?;
    Ok(Json(
        json!({"invoices": controller::crm::invoices(&s.pool)?}),
    ))
}
async fn get_invoice(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "crm", "read")?;
    Ok(Json(
        json!({"invoice": controller::crm::invoice_get(&s.pool,&id)?}),
    ))
}
async fn create_invoice(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<models::crm::NewInvoice>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "crm", "write")?;
    Ok((
        StatusCode::CREATED,
        Json(json!({"invoice": controller::crm::invoice_create(&s.pool,v)?})),
    ))
}
