//! DMC (Destination Management Company / tour operator) models.
//! Mirrors `models::erp` in shape: a `Queryable` + `Identifiable` struct per
//! table, a `New*` `Insertable` for create, an `Update*` `AsChangeset` for
//! patch. All have `id: String` (TEXT primary key) except where the table
//! uses a composite key (just `dmc_supplier_categories`).

use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::schema::{
    dmc_accommodation, dmc_activities, dmc_bookings, dmc_destinations, dmc_documents, dmc_feedback,
    dmc_guides, dmc_incidents, dmc_itineraries, dmc_itinerary_stops, dmc_operations, dmc_packages,
    dmc_payments, dmc_quotes, dmc_reservations, dmc_supplier_categories, dmc_supplier_requests,
    dmc_transport, dmc_trip_events, dmc_trip_services, dmc_trips,
};

// --------------------------------- destinations ---------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_destinations)]
pub struct Destination {
    pub id: String,
    pub country: String,
    pub region: Option<String>,
    pub city: Option<String>,
    pub name: String,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub timezone: Option<String>,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = dmc_destinations)]
pub struct NewDestination<'a> {
    pub id: &'a str,
    pub country: &'a str,
    pub region: Option<&'a str>,
    pub city: Option<&'a str>,
    pub name: &'a str,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub timezone: Option<&'a str>,
    pub description: Option<&'a str>,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = dmc_destinations)]
pub struct UpdateDestination {
    pub country: Option<String>,
    pub region: Option<Option<String>>,
    pub city: Option<Option<String>>,
    pub name: Option<String>,
    pub lat: Option<Option<f64>>,
    pub lon: Option<Option<f64>>,
    pub timezone: Option<Option<String>>,
    pub description: Option<Option<String>>,
}

// ----------------------------------- packages -----------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_packages)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub duration_days: i32,
    pub base_price: f64,
    pub currency: String,
    pub description: Option<String>,
    pub cover_image_file_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = dmc_packages)]
pub struct NewPackage<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub duration_days: i32,
    pub base_price: f64,
    pub currency: &'a str,
    pub description: Option<&'a str>,
    pub cover_image_file_id: Option<i32>,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = dmc_packages)]
pub struct UpdatePackage {
    pub name: Option<String>,
    pub duration_days: Option<i32>,
    pub base_price: Option<f64>,
    pub currency: Option<String>,
    pub description: Option<Option<String>>,
    pub cover_image_file_id: Option<Option<i32>>,
}

// --------------------------------- itineraries ---------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_itineraries)]
pub struct Itinerary {
    pub id: String,
    pub package_id: String,
    pub day_number: i32,
    pub title: String,
    pub notes: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = dmc_itineraries)]
pub struct NewItinerary<'a> {
    pub id: &'a str,
    pub package_id: &'a str,
    pub day_number: i32,
    pub title: &'a str,
    pub notes: Option<&'a str>,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = dmc_itineraries)]
pub struct UpdateItinerary {
    pub day_number: Option<i32>,
    pub title: Option<String>,
    pub notes: Option<Option<String>>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_itinerary_stops)]
pub struct ItineraryStop {
    pub id: String,
    pub itinerary_id: String,
    pub day_number: i32,
    pub hour: i32,
    pub location_id: Option<String>,
    pub activity: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = dmc_itinerary_stops)]
pub struct NewItineraryStop<'a> {
    pub id: &'a str,
    pub itinerary_id: &'a str,
    pub day_number: i32,
    pub hour: i32,
    pub location_id: Option<&'a str>,
    pub activity: Option<&'a str>,
    pub notes: Option<&'a str>,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = dmc_itinerary_stops)]
pub struct UpdateItineraryStop {
    pub day_number: Option<i32>,
    pub hour: Option<i32>,
    pub location_id: Option<Option<String>>,
    pub activity: Option<Option<String>>,
    pub notes: Option<Option<String>>,
}

// ---------------------------------- activities ---------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_activities)]
pub struct Activity {
    pub id: String,
    pub name: String,
    pub category: Option<String>,
    pub duration_minutes: i32,
    pub price: f64,
    pub currency: String,
    pub supplier_id: Option<String>,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = dmc_activities)]
pub struct NewActivity<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub category: Option<&'a str>,
    pub duration_minutes: i32,
    pub price: f64,
    pub currency: &'a str,
    pub supplier_id: Option<&'a str>,
    pub description: Option<&'a str>,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = dmc_activities)]
pub struct UpdateActivity {
    pub name: Option<String>,
    pub category: Option<Option<String>>,
    pub duration_minutes: Option<i32>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub supplier_id: Option<Option<String>>,
    pub description: Option<Option<String>>,
}

// --------------------------------- accommodation --------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_accommodation)]
pub struct Accommodation {
    pub id: String,
    pub name: String,
    pub destination_id: Option<String>,
    pub supplier_id: Option<String>,
    pub room_type: Option<String>,
    pub capacity: i32,
    pub nightly_rate: f64,
    pub currency: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = dmc_accommodation)]
pub struct NewAccommodation<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub destination_id: Option<&'a str>,
    pub supplier_id: Option<&'a str>,
    pub room_type: Option<&'a str>,
    pub capacity: i32,
    pub nightly_rate: f64,
    pub currency: &'a str,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = dmc_accommodation)]
pub struct UpdateAccommodation {
    pub name: Option<String>,
    pub destination_id: Option<Option<String>>,
    pub supplier_id: Option<Option<String>>,
    pub room_type: Option<Option<String>>,
    pub capacity: Option<i32>,
    pub nightly_rate: Option<f64>,
    pub currency: Option<String>,
}

// ----------------------------------- transport ----------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_transport)]
pub struct Transport {
    pub id: String,
    pub mode: String,
    pub supplier_id: Option<String>,
    pub capacity: i32,
    pub hourly_rate: f64,
    pub currency: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = dmc_transport)]
pub struct NewTransport<'a> {
    pub id: &'a str,
    pub mode: &'a str,
    pub supplier_id: Option<&'a str>,
    pub capacity: i32,
    pub hourly_rate: f64,
    pub currency: &'a str,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = dmc_transport)]
pub struct UpdateTransport {
    pub mode: Option<String>,
    pub supplier_id: Option<Option<String>>,
    pub capacity: Option<i32>,
    pub hourly_rate: Option<f64>,
    pub currency: Option<String>,
}

// ---------------------------- supplier_categories ----------------------------

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = dmc_supplier_categories)]
pub struct SupplierCategory {
    pub supplier_id: String,
    pub category: String,
    pub destination_id: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = dmc_supplier_categories)]
pub struct NewSupplierCategory<'a> {
    pub supplier_id: &'a str,
    pub category: &'a str,
    pub destination_id: Option<&'a str>,
}

// ----------------------------------- guides -----------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_guides)]
pub struct Guide {
    pub id: String,
    pub full_name: String,
    pub languages: Option<String>,
    pub daily_rate: f64,
    pub currency: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub supplier_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = dmc_guides)]
pub struct NewGuide<'a> {
    pub id: &'a str,
    pub full_name: &'a str,
    pub languages: Option<&'a str>,
    pub daily_rate: f64,
    pub currency: &'a str,
    pub phone: Option<&'a str>,
    pub email: Option<&'a str>,
    pub supplier_id: Option<&'a str>,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = dmc_guides)]
pub struct UpdateGuide {
    pub full_name: Option<String>,
    pub languages: Option<Option<String>>,
    pub daily_rate: Option<f64>,
    pub currency: Option<String>,
    pub phone: Option<Option<String>>,
    pub email: Option<Option<String>>,
    pub supplier_id: Option<Option<String>>,
}

// ----------------------------------- trips ------------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_trips)]
pub struct Trip {
    pub id: String,
    pub name: String,
    pub package_id: Option<String>,
    pub lead_id: Option<String>,
    pub customer_id: Option<String>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub pax_count: i32,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = dmc_trips)]
pub struct NewTrip {
    pub id: String,
    pub name: String,
    pub package_id: Option<String>,
    pub lead_id: Option<String>,
    pub customer_id: Option<String>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub pax_count: i32,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = dmc_trips)]
pub struct UpdateTrip {
    pub name: Option<String>,
    pub package_id: Option<Option<String>>,
    pub lead_id: Option<Option<String>>,
    pub customer_id: Option<Option<String>>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub pax_count: Option<i32>,
    pub status: Option<String>,
    pub notes: Option<Option<String>>,
}

// -------------------------------- trip_services -------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_trip_services)]
pub struct TripService {
    pub id: String,
    pub trip_id: String,
    pub kind: String,
    pub ref_id: Option<String>,
    pub day_number: i32,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub unit_price: f64,
    pub currency: String,
    pub qty: i32,
    pub total: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = dmc_trip_services)]
pub struct NewTripService<'a> {
    pub id: &'a str,
    pub trip_id: &'a str,
    pub kind: &'a str,
    pub ref_id: Option<&'a str>,
    pub day_number: i32,
    pub start_time: Option<&'a str>,
    pub end_time: Option<&'a str>,
    pub unit_price: f64,
    pub currency: &'a str,
    pub qty: i32,
    pub total: f64,
    pub notes: Option<&'a str>,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = dmc_trip_services)]
pub struct UpdateTripService {
    pub kind: Option<String>,
    pub ref_id: Option<Option<String>>,
    pub day_number: Option<i32>,
    pub start_time: Option<Option<String>>,
    pub end_time: Option<Option<String>>,
    pub unit_price: Option<f64>,
    pub currency: Option<String>,
    pub qty: Option<i32>,
    pub total: Option<f64>,
    pub notes: Option<Option<String>>,
}

// ----------------------------------- quotes -----------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_quotes)]
pub struct Quote {
    pub id: String,
    pub trip_id: String,
    pub version: i32,
    pub currency: String,
    pub subtotal: f64,
    pub tax: f64,
    pub total: f64,
    pub status: String,
    pub sent_at: Option<NaiveDateTime>,
    pub accepted_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = dmc_quotes)]
pub struct NewQuote<'a> {
    pub id: &'a str,
    pub trip_id: &'a str,
    pub version: i32,
    pub currency: &'a str,
    pub subtotal: f64,
    pub tax: f64,
    pub total: f64,
    pub status: &'a str,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = dmc_quotes)]
pub struct UpdateQuote {
    pub version: Option<i32>,
    pub currency: Option<String>,
    pub subtotal: Option<f64>,
    pub tax: Option<f64>,
    pub total: Option<f64>,
    pub status: Option<String>,
    pub sent_at: Option<Option<NaiveDateTime>>,
    pub accepted_at: Option<Option<NaiveDateTime>>,
}

// ---------------------------------- bookings ----------------------------------

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_bookings)]
pub struct Booking {
    pub id: String,
    pub quote_id: String,
    pub status: String,
    pub deposit_paid: f64,
    pub balance_due: f64,
    pub currency: String,
    pub total: f64,
    pub confirmed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = dmc_bookings)]
pub struct NewBooking<'a> {
    pub id: &'a str,
    pub quote_id: &'a str,
    pub status: &'a str,
    pub deposit_paid: f64,
    pub balance_due: f64,
    pub currency: &'a str,
    pub total: f64,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = dmc_bookings)]
pub struct UpdateBooking {
    pub status: Option<String>,
    pub deposit_paid: Option<f64>,
    pub balance_due: Option<f64>,
    pub currency: Option<String>,
    pub total: Option<f64>,
    pub confirmed_at: Option<Option<NaiveDateTime>>,
}

// ==================== TOOGO lifecycle extensions ====================

// Dedicated incidents (trip FK + cost field, separate from support_tickets)
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_incidents)]
pub struct Incident {
    pub id: String,
    pub trip_id: String,
    #[diesel(column_name = type_)]
    pub type_: String,
    pub severity: String,
    pub title: String,
    pub description: Option<String>,
    pub assigned_to: Option<String>,
    pub replacement_vehicle: Option<String>,
    pub extra_cost: f32,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub resolved_at: Option<NaiveDateTime>,
}
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = dmc_incidents)]
pub struct NewIncident {
    pub id: String,
    pub trip_id: String,
    #[serde(rename = "type")]
    #[diesel(column_name = type_)]
    pub type_: String,
    pub severity: String,
    pub title: String,
    pub description: Option<String>,
    pub assigned_to: Option<String>,
    pub replacement_vehicle: Option<String>,
    pub extra_cost: f32,
    pub status: String,
}
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = dmc_incidents)]
pub struct UpdateIncident {
    #[diesel(column_name = type_)]
    pub type_: Option<String>,
    pub severity: Option<String>,
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub assigned_to: Option<Option<String>>,
    pub replacement_vehicle: Option<Option<String>>,
    pub extra_cost: Option<f32>,
    pub status: Option<String>,
    pub resolved_at: Option<Option<NaiveDateTime>>,
}

// Documents linked to storage (reuse user_files)
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_documents)]
pub struct Document {
    pub id: String,
    pub trip_id: String,
    pub kind: String,
    pub file_id: Option<i32>,
    pub collection: String,
    pub filename: Option<String>,
    pub is_primary: i32,
    pub created_at: NaiveDateTime,
}
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = dmc_documents)]
pub struct NewDocument {
    pub id: String,
    pub trip_id: String,
    pub kind: String,
    pub file_id: Option<i32>,
    pub collection: String,
    pub filename: Option<String>,
    pub is_primary: i32,
}

// Reservations per trip_service
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_reservations)]
pub struct Reservation {
    pub id: String,
    pub trip_service_id: String,
    pub supplier_id: Option<String>,
    pub status: String,
    pub external_ref: Option<String>,
    pub confirmed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = dmc_reservations)]
pub struct NewReservation {
    pub id: String,
    pub trip_service_id: String,
    pub supplier_id: Option<String>,
    pub status: String,
    pub external_ref: Option<String>,
}

// Supplier requests
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_supplier_requests)]
pub struct SupplierRequest {
    pub id: String,
    pub trip_id: String,
    pub supplier_id: String,
    pub kind: String,
    pub qty: f32,
    pub cost: f32,
    pub selling: f32,
    pub margin: f32,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = dmc_supplier_requests)]
pub struct NewSupplierRequest {
    pub id: String,
    pub trip_id: String,
    pub supplier_id: String,
    pub kind: String,
    pub qty: f32,
    pub cost: f32,
    pub selling: f32,
    pub margin: f32,
    pub status: String,
}

// Payments (post to accounting_invoices)
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_payments)]
pub struct Payment {
    pub id: String,
    pub booking_id: String,
    pub kind: String,
    pub amount: f32,
    pub currency: String,
    pub method: Option<String>,
    pub status: String,
    pub paid_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = dmc_payments)]
pub struct NewPayment {
    pub id: String,
    pub booking_id: String,
    pub kind: String,
    pub amount: f32,
    pub currency: String,
    pub method: Option<String>,
    pub status: String,
}

// Operations + events
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_operations)]
pub struct Operation {
    pub id: String,
    pub trip_id: String,
    pub day_number: i32,
    pub service_id: Option<String>,
    pub assignee_type: String,
    pub assignee_id: Option<String>,
    pub status: String,
    pub start_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = dmc_operations)]
pub struct NewOperation {
    pub id: String,
    pub trip_id: String,
    pub day_number: i32,
    pub service_id: Option<String>,
    pub assignee_type: String,
    pub assignee_id: Option<String>,
    pub status: String,
}
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_trip_events)]
pub struct TripEvent {
    pub id: String,
    pub trip_id: String,
    pub event_type: String,
    pub scheduled_at: Option<NaiveDateTime>,
    pub actual_at: Option<NaiveDateTime>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
}
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = dmc_trip_events)]
pub struct NewTripEvent {
    pub id: String,
    pub trip_id: String,
    pub event_type: String,
    pub scheduled_at: Option<NaiveDateTime>,
    pub actual_at: Option<NaiveDateTime>,
    pub status: String,
    pub notes: Option<String>,
}

// Feedback
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_feedback)]
pub struct Feedback {
    pub id: String,
    pub trip_id: String,
    pub customer_id: Option<String>,
    pub rating: i32,
    pub category: Option<String>,
    pub comment: Option<String>,
    pub resolved_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = dmc_feedback)]
pub struct NewFeedback {
    pub id: String,
    pub trip_id: String,
    pub customer_id: Option<String>,
    pub rating: i32,
    pub category: Option<String>,
    pub comment: Option<String>,
}
