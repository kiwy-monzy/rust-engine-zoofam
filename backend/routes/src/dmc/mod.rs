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
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::{json, Value};

fn need(c: &Claims, m: &str, a: &str) -> Result<(), ApiError> {
    c.require(m, a)
        .map_err(|e| ApiError::new(StatusCode::FORBIDDEN, e.to_string()))
}

pub fn dmc_routes() -> Router<AppState> {
    Router::new()
        // Destinations
        .route(
            "/dmc/destinations",
            get(list_destinations).post(create_destination),
        )
        .route(
            "/dmc/destinations/:id",
            get(get_destination)
                .patch(update_destination)
                .delete(delete_destination),
        )
        // Packages
        .route("/dmc/packages", get(list_packages).post(create_package))
        .route(
            "/dmc/packages/:id",
            get(get_package)
                .patch(update_package)
                .delete(delete_package),
        )
        // Itineraries (scoped to package)
        .route(
            "/dmc/packages/:pkg_id/itineraries",
            get(list_itineraries).post(create_itinerary),
        )
        .route(
            "/dmc/itineraries/:id",
            get(get_itinerary)
                .patch(update_itinerary)
                .delete(delete_itinerary),
        )
        // Itinerary stops (scoped to itinerary)
        .route(
            "/dmc/itineraries/:it_id/stops",
            get(list_itinerary_stops).post(create_itinerary_stop),
        )
        .route(
            "/dmc/itinerary-stops/:id",
            patch(update_itinerary_stop).delete(delete_itinerary_stop),
        )
        // Activities
        .route(
            "/dmc/activities",
            get(list_activities).post(create_activity),
        )
        .route(
            "/dmc/activities/:id",
            get(get_activity)
                .patch(update_activity)
                .delete(delete_activity),
        )
        // Accommodation
        .route(
            "/dmc/accommodation",
            get(list_accommodation).post(create_accommodation),
        )
        .route(
            "/dmc/accommodation/:id",
            get(get_accommodation)
                .patch(update_accommodation)
                .delete(delete_accommodation),
        )
        // Transport
        .route("/dmc/transport", get(list_transport).post(create_transport))
        .route(
            "/dmc/transport/:id",
            get(get_transport)
                .patch(update_transport)
                .delete(delete_transport),
        )
        // Supplier categories
        .route(
            "/dmc/supplier-categories",
            get(list_supplier_categories).post(upsert_supplier_category),
        )
        .route(
            "/dmc/supplier-categories/:sid/:cat",
            delete(delete_supplier_category),
        )
        // Guides
        .route("/dmc/guides", get(list_guides).post(create_guide))
        .route(
            "/dmc/guides/:id",
            get(get_guide).patch(update_guide).delete(delete_guide),
        )
        // Trips
        .route("/dmc/trips", get(list_trips).post(create_trip))
        .route(
            "/dmc/trips/:id",
            get(get_trip).patch(update_trip).delete(delete_trip),
        )
        .route("/dmc/trips/:id/detail", get(trip_detail))
        .route(
            "/dmc/trips/destination/:dest_id",
            get(trips_for_destination),
        )
        // Trip services
        .route(
            "/dmc/trips/:trip_id/services",
            get(list_trip_services).post(create_trip_service),
        )
        .route(
            "/dmc/trip-services/:id",
            patch(update_trip_service).delete(delete_trip_service),
        )
        // Quotes
        .route(
            "/dmc/trips/:trip_id/quotes",
            get(list_quotes).post(create_quote),
        )
        .route(
            "/dmc/quotes/:id",
            get(get_quote).patch(update_quote).delete(delete_quote),
        )
        .route("/dmc/quotes/:id/accept", post(accept_quote))
        .route("/dmc/trips/:trip_id/calculate-quote", post(calculate_quote))
        // Bookings
        .route(
            "/dmc/quotes/:quote_id/bookings",
            get(list_bookings).post(create_booking),
        )
        .route(
            "/dmc/bookings/:id",
            get(get_booking)
                .patch(update_booking)
                .delete(delete_booking),
        )
        .route("/dmc/bookings/:id/confirm", post(confirm_booking))
        // TOOGO lifecycle: catalog view (unified over erp_products)
        .route("/dmc/catalog", get(list_catalog))
        // dedicated incidents (trip FK + cost field)
        .route(
            "/dmc/trips/:trip_id/incidents",
            get(list_incidents).post(create_incident),
        )
        .route(
            "/dmc/incidents/:id",
            get(get_incident)
                .patch(update_incident)
                .delete(delete_incident),
        )
        .route("/dmc/incidents/:id/resolve", post(resolve_incident))
        // documents (storage picker, reuse user_files)
        .route(
            "/dmc/trips/:trip_id/documents",
            get(list_documents).post(attach_document),
        )
        .route("/dmc/documents/:id", delete(delete_document))
        // reservations readiness
        .route("/dmc/trips/:trip_id/reservations", get(list_reservations))
        .route(
            "/dmc/trip-services/:svc_id/reservations",
            post(create_reservation),
        )
        .route("/dmc/reservations/:id/confirm", post(confirm_reservation))
        // supplier requests (posts to erp_purchase_orders on confirm)
        .route(
            "/dmc/trips/:trip_id/supplier-requests",
            get(list_supplier_requests).post(create_supplier_request),
        )
        .route(
            "/dmc/supplier-requests/:id/confirm",
            post(confirm_supplier_request),
        )
        // payments + finance (posts to accounting_invoices)
        .route(
            "/dmc/bookings/:bid/payments",
            get(list_payments).post(create_payment),
        )
        .route("/dmc/payments/:id/confirm", post(confirm_payment))
        .route("/dmc/payments/:id/status", get(check_payment_status))
        // operations / events / feedback
        .route(
            "/dmc/trips/:trip_id/operations",
            get(list_operations).post(create_operation),
        )
        .route(
            "/dmc/trips/:trip_id/events",
            get(list_events).post(create_event),
        )
        .route(
            "/dmc/trips/:trip_id/feedback",
            get(list_feedback).post(create_feedback),
        )
        // Apple Wallet (dmc_wallet_passes)
        .route("/dmc/wallet/samples", get(wallet_list_samples))
        .route("/dmc/wallet/passes", get(wallet_list_passes))
        .route("/dmc/wallet/issue", post(wallet_issue_pass))
        .route(
            "/dmc/wallet/passes/:pass_type/:serial",
            get(wallet_get_pass)
                .patch(wallet_update_pass)
                .delete(wallet_delete_pass),
        )
        .route(
            "/dmc/wallet/passes/:pass_type/:serial/qr.svg",
            get(wallet_qr),
        )
        // Clickpesa webhook
        .route("/webhooks/clickpesa", post(clickpesa_webhook))
}

// --------------------------------------------------------------- destinations ---

async fn list_destinations(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"destinations": controller::dmc::list_destinations(&s.pool)?}),
    ))
}
async fn get_destination(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"destination": controller::dmc::get_destination(&s.pool, &id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateDestination {
    name: String,
    country: String,
    region: Option<String>,
    city: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    timezone: Option<String>,
    description: Option<String>,
}
async fn create_destination(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<CreateDestination>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let r = controller::dmc::create_destination(
        &s.pool,
        &v.name,
        &v.country,
        v.region.as_deref(),
        v.city.as_deref(),
        v.lat,
        v.lon,
        v.timezone.as_deref(),
        v.description.as_deref(),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"destination": r}))))
}
async fn update_destination(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<models::dmc::UpdateDestination>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "write")?;
    Ok(Json(
        json!({"destination": controller::dmc::update_destination(&s.pool, &id, &v)?}),
    ))
}
async fn delete_destination(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "dmc", "write")?;
    controller::dmc::delete_destination(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// ----------------------------------------------------------------- packages ---

async fn list_packages(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"packages": controller::dmc::list_packages(&s.pool)?}),
    ))
}
async fn get_package(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"package": controller::dmc::get_package(&s.pool, &id)?}),
    ))
}
#[derive(Deserialize)]
struct CreatePackage {
    name: String,
    duration_days: i32,
    base_price: f64,
    currency: String,
    description: Option<String>,
    cover_image_file_id: Option<i32>,
}
async fn create_package(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<CreatePackage>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let r = controller::dmc::create_package(
        &s.pool,
        &v.name,
        v.duration_days,
        v.base_price,
        &v.currency,
        v.description.as_deref(),
        v.cover_image_file_id,
    )?;
    Ok((StatusCode::CREATED, Json(json!({"package": r}))))
}
async fn update_package(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<models::dmc::UpdatePackage>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "write")?;
    Ok(Json(
        json!({"package": controller::dmc::update_package(&s.pool, &id, &v)?}),
    ))
}
async fn delete_package(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "dmc", "write")?;
    controller::dmc::delete_package(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// ----------------------------------------------------------- itineraries ---

async fn list_itineraries(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(pkg_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"itineraries": controller::dmc::list_itineraries(&s.pool, &pkg_id)?}),
    ))
}
async fn get_itinerary(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"itinerary": controller::dmc::get_itinerary(&s.pool, &id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateItinerary {
    day_number: i32,
    title: String,
    notes: Option<String>,
}
async fn create_itinerary(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(pkg_id): Path<String>,
    Json(v): Json<CreateItinerary>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let r = controller::dmc::create_itinerary(
        &s.pool,
        &pkg_id,
        v.day_number,
        &v.title,
        v.notes.as_deref(),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"itinerary": r}))))
}
async fn update_itinerary(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<models::dmc::UpdateItinerary>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "write")?;
    Ok(Json(
        json!({"itinerary": controller::dmc::update_itinerary(&s.pool, &id, &v)?}),
    ))
}
async fn delete_itinerary(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "dmc", "write")?;
    controller::dmc::delete_itinerary(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// ------------------------------------------------------- itinerary stops ---

async fn list_itinerary_stops(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(it_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"stops": controller::dmc::list_itinerary_stops(&s.pool, &it_id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateItineraryStop {
    day_number: i32,
    hour: i32,
    location_id: Option<String>,
    activity: Option<String>,
    notes: Option<String>,
}
async fn create_itinerary_stop(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(it_id): Path<String>,
    Json(v): Json<CreateItineraryStop>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let r = controller::dmc::create_itinerary_stop(
        &s.pool,
        &it_id,
        v.day_number,
        v.hour,
        v.location_id.as_deref(),
        v.activity.as_deref(),
        v.notes.as_deref(),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"stop": r}))))
}
async fn update_itinerary_stop(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<models::dmc::UpdateItineraryStop>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "write")?;
    Ok(Json(
        json!({"stop": controller::dmc::update_itinerary_stop(&s.pool, &id, &v)?}),
    ))
}
async fn delete_itinerary_stop(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "dmc", "write")?;
    controller::dmc::delete_itinerary_stop(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// ----------------------------------------------------------------- activities ---

async fn list_activities(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"activities": controller::dmc::list_activities(&s.pool)?}),
    ))
}
async fn get_activity(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"activity": controller::dmc::get_activity(&s.pool, &id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateActivity {
    name: String,
    category: Option<String>,
    duration_minutes: i32,
    price: f64,
    currency: String,
    supplier_id: Option<String>,
    description: Option<String>,
}
async fn create_activity(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<CreateActivity>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let r = controller::dmc::create_activity(
        &s.pool,
        &v.name,
        v.category.as_deref(),
        v.duration_minutes,
        v.price,
        &v.currency,
        v.supplier_id.as_deref(),
        v.description.as_deref(),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"activity": r}))))
}
async fn update_activity(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<models::dmc::UpdateActivity>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "write")?;
    Ok(Json(
        json!({"activity": controller::dmc::update_activity(&s.pool, &id, &v)?}),
    ))
}
async fn delete_activity(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "dmc", "write")?;
    controller::dmc::delete_activity(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// ----------------------------------------------------------- accommodation ---

async fn list_accommodation(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"accommodation": controller::dmc::list_accommodation(&s.pool)?}),
    ))
}
async fn get_accommodation(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"accommodation": controller::dmc::get_accommodation(&s.pool, &id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateAccommodation {
    name: String,
    destination_id: Option<String>,
    supplier_id: Option<String>,
    room_type: Option<String>,
    capacity: i32,
    nightly_rate: f64,
    currency: String,
}
async fn create_accommodation(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<CreateAccommodation>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let r = controller::dmc::create_accommodation(
        &s.pool,
        &v.name,
        v.destination_id.as_deref(),
        v.supplier_id.as_deref(),
        v.room_type.as_deref(),
        v.capacity,
        v.nightly_rate,
        &v.currency,
    )?;
    Ok((StatusCode::CREATED, Json(json!({"accommodation": r}))))
}
async fn update_accommodation(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<models::dmc::UpdateAccommodation>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "write")?;
    Ok(Json(
        json!({"accommodation": controller::dmc::update_accommodation(&s.pool, &id, &v)?}),
    ))
}
async fn delete_accommodation(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "dmc", "write")?;
    controller::dmc::delete_accommodation(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// ----------------------------------------------------------------- transport ---

async fn list_transport(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"transport": controller::dmc::list_transport(&s.pool)?}),
    ))
}
async fn get_transport(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"transport": controller::dmc::get_transport(&s.pool, &id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateTransport {
    mode: String,
    supplier_id: Option<String>,
    capacity: i32,
    hourly_rate: f64,
    currency: String,
}
async fn create_transport(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<CreateTransport>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let r = controller::dmc::create_transport(
        &s.pool,
        &v.mode,
        v.supplier_id.as_deref(),
        v.capacity,
        v.hourly_rate,
        &v.currency,
    )?;
    Ok((StatusCode::CREATED, Json(json!({"transport": r}))))
}
async fn update_transport(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<models::dmc::UpdateTransport>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "write")?;
    Ok(Json(
        json!({"transport": controller::dmc::update_transport(&s.pool, &id, &v)?}),
    ))
}
async fn delete_transport(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "dmc", "write")?;
    controller::dmc::delete_transport(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// ----------------------------------------------------- supplier categories ---

async fn list_supplier_categories(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"supplier_categories": controller::dmc::list_supplier_categories(&s.pool)?}),
    ))
}
#[derive(Deserialize)]
struct UpsertSupplierCategory {
    supplier_id: String,
    category: String,
    destination_id: Option<String>,
}
async fn upsert_supplier_category(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<UpsertSupplierCategory>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let r = controller::dmc::upsert_supplier_category(
        &s.pool,
        &v.supplier_id,
        &v.category,
        v.destination_id.as_deref(),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"supplier_category": r}))))
}
async fn delete_supplier_category(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path((sid, cat)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    need(&c, "dmc", "write")?;
    controller::dmc::delete_supplier_category(&s.pool, &sid, &cat)?;
    Ok(StatusCode::NO_CONTENT)
}

// ------------------------------------------------------------------- guides ---

async fn list_guides(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"guides": controller::dmc::list_guides(&s.pool)?}),
    ))
}
async fn get_guide(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"guide": controller::dmc::get_guide(&s.pool, &id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateGuide {
    full_name: String,
    languages: Option<String>,
    daily_rate: f64,
    currency: String,
    phone: Option<String>,
    email: Option<String>,
    supplier_id: Option<String>,
}
async fn create_guide(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<CreateGuide>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let r = controller::dmc::create_guide(
        &s.pool,
        &v.full_name,
        v.languages.as_deref(),
        v.daily_rate,
        &v.currency,
        v.phone.as_deref(),
        v.email.as_deref(),
        v.supplier_id.as_deref(),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"guide": r}))))
}
async fn update_guide(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<models::dmc::UpdateGuide>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "write")?;
    Ok(Json(
        json!({"guide": controller::dmc::update_guide(&s.pool, &id, &v)?}),
    ))
}
async fn delete_guide(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "dmc", "write")?;
    controller::dmc::delete_guide(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// -------------------------------------------------------------------- trips ---

async fn list_trips(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"trips": controller::dmc::list_trips(&s.pool)?}),
    ))
}
async fn get_trip(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"trip": controller::dmc::get_trip(&s.pool, &id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateTrip {
    name: String,
    package_id: Option<String>,
    lead_id: Option<String>,
    customer_id: Option<String>,
    start_date: String,
    end_date: String,
    pax_count: i32,
    status: Option<String>,
    notes: Option<String>,
}
async fn create_trip(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<CreateTrip>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let r = controller::dmc::create_trip(
        &s.pool,
        &v.name,
        v.package_id.as_deref(),
        v.lead_id.as_deref(),
        v.customer_id.as_deref(),
        &v.start_date,
        &v.end_date,
        v.pax_count,
        v.status.as_deref().unwrap_or("draft"),
        v.notes.as_deref(),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"trip": r}))))
}
async fn update_trip(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<models::dmc::UpdateTrip>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "write")?;
    Ok(Json(
        json!({"trip": controller::dmc::update_trip(&s.pool, &id, &v)?}),
    ))
}
async fn delete_trip(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "dmc", "write")?;
    controller::dmc::delete_trip(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn trip_detail(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    let d = controller::dmc::trip_detail(&s.pool, &id)?;
    Ok(Json(
        json!({"trip": d.trip, "services": d.services, "itinerary": d.itinerary.into_iter().map(|s| json!({"stop": s.stop, "calendar_date": s.calendar_date.to_string(), "location": s.location})).collect::<Vec<_>>(), "destinations": d.destinations}),
    ))
}
async fn trips_for_destination(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(dest_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"trips": controller::dmc::trips_for_destination(&s.pool, &dest_id)?}),
    ))
}

// ----------------------------------------------------------- trip services ---

async fn list_trip_services(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(trip_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"services": controller::dmc::list_trip_services(&s.pool, &trip_id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateTripService {
    kind: String,
    ref_id: Option<String>,
    day_number: i32,
    start_time: Option<String>,
    end_time: Option<String>,
    unit_price: f64,
    currency: String,
    qty: i32,
    total: f64,
    notes: Option<String>,
}
async fn create_trip_service(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(trip_id): Path<String>,
    Json(v): Json<CreateTripService>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let r = controller::dmc::create_trip_service(
        &s.pool,
        &trip_id,
        &v.kind,
        v.ref_id.as_deref(),
        v.day_number,
        v.start_time.as_deref(),
        v.end_time.as_deref(),
        v.unit_price,
        &v.currency,
        v.qty,
        v.total,
        v.notes.as_deref(),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"service": r}))))
}
async fn update_trip_service(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<models::dmc::UpdateTripService>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "write")?;
    Ok(Json(
        json!({"service": controller::dmc::update_trip_service(&s.pool, &id, &v)?}),
    ))
}
async fn delete_trip_service(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "dmc", "write")?;
    controller::dmc::delete_trip_service(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// ------------------------------------------------------------------- quotes ---

async fn list_quotes(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(trip_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"quotes": controller::dmc::list_quotes(&s.pool, &trip_id)?}),
    ))
}
async fn get_quote(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"quote": controller::dmc::get_quote(&s.pool, &id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateQuote {
    #[serde(default)]
    trip_id: String,
    version: i32,
    currency: String,
    subtotal: f64,
    tax: f64,
    total: f64,
    status: Option<String>,
}
async fn create_quote(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(trip_id): Path<String>,
    Json(v): Json<CreateQuote>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let _ = &v.trip_id; // path param is canonical; body trip_id kept for backward compat
    let r = controller::dmc::create_quote(
        &s.pool,
        &trip_id,
        v.version,
        &v.currency,
        v.subtotal,
        v.tax,
        v.total,
        v.status.as_deref().unwrap_or("draft"),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"quote": r}))))
}
async fn update_quote(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<models::dmc::UpdateQuote>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "write")?;
    Ok(Json(
        json!({"quote": controller::dmc::update_quote(&s.pool, &id, &v)?}),
    ))
}
async fn delete_quote(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "dmc", "write")?;
    controller::dmc::delete_quote(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn accept_quote(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "write")?;
    let q = controller::dmc::accept_quote(&s.pool, &id)?;
    Ok(Json(json!({"quote": q})))
}
async fn calculate_quote(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(trip_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    let (currency, subtotal, tax, total) = controller::dmc::calculate_quote(&s.pool, &trip_id)?;
    Ok(Json(
        json!({"currency": currency, "subtotal": subtotal, "tax": tax, "total": total}),
    ))
}

// ----------------------------------------------------------------- bookings ---

async fn list_bookings(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(quote_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"bookings": controller::dmc::list_bookings(&s.pool, &quote_id)?}),
    ))
}
async fn get_booking(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"booking": controller::dmc::get_booking(&s.pool, &id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateBooking {
    #[serde(default)]
    quote_id: String,
    status: Option<String>,
    deposit_paid: Option<f64>,
    balance_due: Option<f64>,
    currency: Option<String>,
    total: Option<f64>,
}
async fn create_booking(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(quote_id): Path<String>,
    Json(v): Json<CreateBooking>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let _ = &v.quote_id; // path param is canonical
    let r = controller::dmc::create_booking(
        &s.pool,
        &quote_id,
        v.status.as_deref().unwrap_or("pending"),
        v.deposit_paid.unwrap_or(0.0),
        v.balance_due.unwrap_or(0.0),
        v.currency.as_deref().unwrap_or("TZS"),
        v.total.unwrap_or(0.0),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"booking": r}))))
}
async fn update_booking(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<models::dmc::UpdateBooking>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "write")?;
    Ok(Json(
        json!({"booking": controller::dmc::update_booking(&s.pool, &id, &v)?}),
    ))
}
async fn confirm_booking(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "write")?;
    let b = controller::dmc::confirm_booking(&s.pool, &id)?;
    Ok(Json(json!({"booking": b})))
}
async fn delete_booking(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "dmc", "write")?;
    controller::dmc::delete_booking(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}

// TOOGO lifecycle: catalog view over erp_products + dedicated incidents/documents/reservations/payments/operations
async fn list_catalog(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"catalog": controller::dmc::list_catalog_items(&s.pool)?}),
    ))
}
async fn list_incidents(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(trip_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"incidents": controller::dmc::list_incidents(&s.pool, &trip_id)?}),
    ))
}
async fn get_incident(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    let mut conn = db::conn(&s.pool).map_err(|e| {
        crate::middleware::ApiError::new(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            e.to_string(),
        )
    })?;
    let r: models::dmc::Incident = models::schema::dmc_incidents::table
        .find(id.as_str())
        .first(&mut conn)
        .map_err(|_| {
            crate::middleware::ApiError::new(axum::http::StatusCode::NOT_FOUND, "incident")
        })?;
    Ok(Json(json!({"incident": r})))
}
#[derive(Deserialize)]
struct CreateIncident {
    #[serde(rename = "type")]
    type_: String,
    severity: Option<String>,
    title: String,
    description: Option<String>,
    assigned_to: Option<String>,
    replacement_vehicle: Option<String>,
    extra_cost: Option<f64>,
}
async fn create_incident(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(trip_id): Path<String>,
    Json(v): Json<CreateIncident>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let r = controller::dmc::create_incident(
        &s.pool,
        &trip_id,
        &v.type_,
        v.severity.as_deref().unwrap_or("medium"),
        &v.title,
        v.description.as_deref(),
        v.assigned_to.as_deref(),
        v.replacement_vehicle.as_deref(),
        v.extra_cost.unwrap_or(0.0) as f32,
    )?;
    Ok((StatusCode::CREATED, Json(json!({"incident": r}))))
}
async fn update_incident(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
    Json(v): Json<models::dmc::UpdateIncident>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "write")?;
    Ok(Json(
        json!({"incident": controller::dmc::update_incident(&s.pool, &id, &v)?}),
    ))
}
async fn resolve_incident(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "write")?;
    Ok(Json(
        json!({"incident": controller::dmc::resolve_incident(&s.pool, &id)?}),
    ))
}
async fn delete_incident(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "dmc", "write")?;
    controller::dmc::delete_incident(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn list_documents(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(trip_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"documents": controller::dmc::list_documents(&s.pool, &trip_id)?}),
    ))
}
#[derive(Deserialize)]
struct AttachDoc {
    kind: Option<String>,
    file_id: Option<i32>,
    collection: Option<String>,
    filename: Option<String>,
    is_primary: Option<bool>,
}
async fn attach_document(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(trip_id): Path<String>,
    Json(v): Json<AttachDoc>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let r = controller::dmc::attach_document(
        &s.pool,
        &trip_id,
        v.kind.as_deref().unwrap_or("voucher"),
        v.file_id,
        v.collection.as_deref().unwrap_or("dmc-documents"),
        v.filename.as_deref(),
        v.is_primary.unwrap_or(false),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"document": r}))))
}
async fn delete_document(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    need(&c, "dmc", "write")?;
    controller::dmc::delete_document(&s.pool, &id)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn list_reservations(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(trip_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"reservations": controller::dmc::list_reservations(&s.pool, &trip_id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateRes {
    supplier_id: Option<String>,
    external_ref: Option<String>,
}
async fn create_reservation(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(svc_id): Path<String>,
    Json(v): Json<CreateRes>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let r = controller::dmc::create_reservation(
        &s.pool,
        &svc_id,
        v.supplier_id.as_deref(),
        v.external_ref.as_deref(),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"reservation": r}))))
}
async fn confirm_reservation(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "write")?;
    Ok(Json(
        json!({"reservation": controller::dmc::confirm_reservation(&s.pool, &id)?}),
    ))
}
async fn list_supplier_requests(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(trip_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"requests": controller::dmc::list_supplier_requests(&s.pool, &trip_id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateSupReq {
    supplier_id: String,
    kind: Option<String>,
    qty: Option<f64>,
    cost: Option<f64>,
    selling: Option<f64>,
}
async fn create_supplier_request(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(trip_id): Path<String>,
    Json(v): Json<CreateSupReq>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let r = controller::dmc::create_supplier_request(
        &s.pool,
        &trip_id,
        &v.supplier_id,
        v.kind.as_deref().unwrap_or("service"),
        v.qty.unwrap_or(1.0) as f32,
        v.cost.unwrap_or(0.0) as f32,
        v.selling.unwrap_or(0.0) as f32,
    )?;
    Ok((StatusCode::CREATED, Json(json!({"request": r}))))
}
async fn confirm_supplier_request(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "write")?;
    Ok(Json(
        json!({"request": controller::dmc::confirm_supplier_request(&s.pool, &id)?}),
    ))
}
async fn list_payments(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(bid): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"payments": controller::dmc::list_payments(&s.pool, &bid)?}),
    ))
}
#[derive(Deserialize)]
struct CreatePay {
    kind: Option<String>,
    amount: f64,
    currency: Option<String>,
    method: Option<String>,
}
async fn create_payment(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(bid): Path<String>,
    Json(v): Json<CreatePay>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let r = controller::dmc::create_payment(
        &s.pool,
        &bid,
        v.kind.as_deref().unwrap_or("DEPOSIT"),
        v.amount as f32,
        v.currency.as_deref().unwrap_or("TZS"),
        v.method.as_deref(),
    )
    .map_err(|e| crate::middleware::ApiError::new(axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(json!({"payment": r}))))
}
async fn confirm_payment(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "write")?;
    let payment = controller::dmc::check_payment_status(&s.pool, &id)
        .map_err(|e| crate::middleware::ApiError::new(axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if payment.status == "SUCCESS" || payment.status == "SETTLED" || payment.status == "PAID" {
        let confirmed = controller::dmc::confirm_payment(&s.pool, &id)?;
        Ok(Json(json!({"payment": confirmed})))
    } else {
        Ok(Json(json!({"payment": payment})))
    }
}

async fn check_payment_status(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    let payment = controller::dmc::check_payment_status(&s.pool, &id)
        .map_err(|e| crate::middleware::ApiError::new(axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json!({"payment": payment})))
}

#[derive(Debug, Deserialize)]
struct ClickpesaWebhook {
    #[serde(rename = "orderReference")]
    order_reference: String,
    status: String,
    channel: Option<String>,
    #[serde(rename = "collectedAmount")]
    collected_amount: Option<String>,
    #[serde(rename = "collectedCurrency")]
    collected_currency: Option<String>,
    id: Option<String>,
}

async fn clickpesa_webhook(
    State(s): State<AppState>,
    Json(payload): Json<ClickpesaWebhook>,
) -> ApiResult<Json<Value>> {
    tracing::info!("Clickpesa webhook received: {:?}", payload);
    if payload.status == "SUCCESS" || payload.status == "SETTLED" {
        if let Some(order_ref) = payload.order_reference.strip_prefix("DMC-") {
            let parts: Vec<&str> = order_ref.splitn(2, '-').collect();
            if parts.len() == 2 {
                let booking_id = parts[0];
                let payment_id = parts[1];
                let mut c = db::conn(&s.pool).map_err(|e| {
                    crate::middleware::ApiError::new(axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                })?;
                diesel::update(models::schema::dmc_payments::table.find(payment_id))
                    .set((
                        models::schema::dmc_payments::status.eq("PAID"),
                        models::schema::dmc_payments::paid_at.eq(Some(chrono::Utc::now().naive_utc())),
                    ))
                    .execute(&mut c)
                    .ok();
                tracing::info!("Payment {} confirmed via webhook", payment_id);
            }
        }
    }
    Ok(Json(json!({"received": true})))
}
async fn list_operations(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(trip_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"operations": controller::dmc::list_operations(&s.pool, &trip_id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateOp {
    day_number: Option<i32>,
    service_id: Option<String>,
    assignee_type: Option<String>,
    assignee_id: Option<String>,
}
async fn create_operation(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(trip_id): Path<String>,
    Json(v): Json<CreateOp>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let r = controller::dmc::create_operation(
        &s.pool,
        &trip_id,
        v.day_number.unwrap_or(1),
        v.service_id.as_deref(),
        v.assignee_type.as_deref().unwrap_or("GUIDE"),
        v.assignee_id.as_deref(),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"operation": r}))))
}
async fn list_events(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(trip_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"events": controller::dmc::list_events(&s.pool, &trip_id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateEvent {
    event_type: String,
    scheduled_at: Option<String>,
    status: Option<String>,
}
async fn create_event(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(trip_id): Path<String>,
    Json(v): Json<CreateEvent>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let sched = v.scheduled_at.as_deref().and_then(|s| {
        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
            .ok()
            .or_else(|| {
                chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .ok()
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
            })
    });
    let r = controller::dmc::create_event(
        &s.pool,
        &trip_id,
        &v.event_type,
        sched,
        v.status.as_deref().unwrap_or("PLANNED"),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"event": r}))))
}
async fn list_feedback(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(trip_id): Path<String>,
) -> ApiResult<Json<Value>> {
    need(&c, "dmc", "read")?;
    Ok(Json(
        json!({"feedback": controller::dmc::list_feedback(&s.pool, &trip_id)?}),
    ))
}
#[derive(Deserialize)]
struct CreateFeedback {
    customer_id: Option<String>,
    rating: Option<i32>,
    comment: Option<String>,
}
async fn create_feedback(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path(trip_id): Path<String>,
    Json(v): Json<CreateFeedback>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "dmc", "write")?;
    let r = controller::dmc::create_feedback(
        &s.pool,
        &trip_id,
        v.customer_id.as_deref(),
        v.rating.unwrap_or(5),
        v.comment.as_deref(),
    )?;
    Ok((StatusCode::CREATED, Json(json!({"feedback": r}))))
}

// ==================================================== Apple Wallet (dmc_wallet) ==

async fn wallet_list_samples(Extension(c): Extension<Claims>) -> ApiResult<Json<Value>> {
    need(&c, "wallet", "read")?;
    Ok(Json(json!({"samples": controller::dmc::wallet_catalog()})))
}
async fn wallet_list_passes(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
) -> ApiResult<Json<Value>> {
    need(&c, "wallet", "read")?;
    Ok(Json(
        json!({"passes": controller::dmc::wallet_list(&s.pool)?}),
    ))
}
#[derive(Deserialize)]
struct WalletIssue {
    sample_id: Option<String>,
}
async fn wallet_issue_pass(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Json(v): Json<WalletIssue>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    need(&c, "wallet", "write")?;
    let wallet = s.wallet.as_ref().ok_or_else(|| {
        ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "wallet not configured".to_string(),
        )
    })?;
    let pass = controller::dmc::wallet_issue(wallet, v.sample_id.as_deref(), None)?;
    Ok((StatusCode::CREATED, Json(json!({"pass": pass}))))
}
async fn wallet_get_pass(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path((pass_type, serial)): Path<(String, String)>,
) -> ApiResult<Json<Value>> {
    need(&c, "wallet", "read")?;
    let p = controller::dmc::wallet_get(&s.pool, &pass_type, &serial)?;
    let json: serde_json::Value =
        serde_json::from_str(&p.data).unwrap_or(serde_json::Value::Null);
    Ok(Json(json!({"pass": p, "passJson": json})))
}
#[derive(Deserialize)]
struct WalletPatch {
    pass_json: serde_json::Value,
}
async fn wallet_update_pass(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path((pass_type, serial)): Path<(String, String)>,
    Json(v): Json<WalletPatch>,
) -> ApiResult<StatusCode> {
    need(&c, "wallet", "write")?;
    controller::dmc::wallet_update_json(&s.pool, &pass_type, &serial, v.pass_json)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn wallet_delete_pass(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path((pass_type, serial)): Path<(String, String)>,
) -> ApiResult<StatusCode> {
    need(&c, "wallet", "write")?;
    controller::dmc::wallet_delete(&s.pool, &pass_type, &serial)?;
    Ok(StatusCode::NO_CONTENT)
}
async fn wallet_qr(
    State(s): State<AppState>,
    Extension(c): Extension<Claims>,
    Path((pass_type, serial)): Path<(String, String)>,
) -> ApiResult<String> {
    need(&c, "wallet", "read")?;
    let wallet = s.wallet.as_ref().ok_or_else(|| {
        ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "wallet not configured".to_string(),
        )
    })?;
    let url = controller::dmc::wallet_download_url(wallet, &pass_type, &serial);
    let svg = wallet
        .qr_svg(&url)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(svg)
}
