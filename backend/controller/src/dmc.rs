//! DMC (Destination Management Company / tour operator) controller.
//! Mirrors the shape of `controller::erp`: one free function per CRUD verb
//! per submodule, plus a handful of cross-module helpers for the trip flow
//! (itinerary expansion, quote calculation, trip detail).

use std::sync::Arc;

use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use applewallet::samples::SamplePassInfo;
use applewallet::webservice::{DeviceRegistration, PassRecord, PassStore};
use applewallet::{PKPass, PassKit};

use db::{conn, DbConn, DbPool};
use models::dmc::*;
use models::schema::{
    dmc_accommodation, dmc_activities, dmc_bookings, dmc_destinations, dmc_guides, dmc_itineraries,
    dmc_itinerary_stops, dmc_packages, dmc_quotes, dmc_supplier_categories, dmc_transport,
    dmc_trip_services, dmc_trips, dmc_wallet_passes, dmc_wallet_registrations,
};
use models::{NewWalletPass, NewWalletRegistration, WalletPass};

use crate::validate;
use crate::{Error, Result};

fn nid() -> String {
    Uuid::new_v4().to_string()
}
#[allow(dead_code)]
fn now() -> NaiveDateTime {
    Utc::now().naive_utc()
}

// ============================================================ destinations ===

pub fn list_destinations(pool: &DbPool) -> Result<Vec<Destination>> {
    let mut c = conn(pool)?;
    Ok(dmc_destinations::table
        .order(dmc_destinations::name.asc())
        .load::<Destination>(&mut c)?)
}

pub fn get_destination(pool: &DbPool, id: &str) -> Result<Destination> {
    let mut c = conn(pool)?;
    dmc_destinations::table
        .find(id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("destination"))
}

pub fn create_destination(
    pool: &DbPool,
    name: &str,
    country: &str,
    region: Option<&str>,
    city: Option<&str>,
    lat: Option<f64>,
    lon: Option<f64>,
    timezone: Option<&str>,
    description: Option<&str>,
) -> Result<Destination> {
    let id = nid();
    let new = NewDestination {
        id: &id,
        country,
        region,
        city,
        name,
        lat,
        lon,
        timezone,
        description,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(dmc_destinations::table)
        .values(&new)
        .execute(&mut c)?;
    dmc_destinations::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn update_destination(
    pool: &DbPool,
    id: &str,
    patch: &UpdateDestination,
) -> Result<Destination> {
    let mut c = conn(pool)?;
    diesel::update(dmc_destinations::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    dmc_destinations::table
        .find(id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn delete_destination(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::delete(dmc_destinations::table.find(id)).execute(&mut c)?;
    Ok(())
}

// ================================================================ packages ===

pub fn list_packages(pool: &DbPool) -> Result<Vec<Package>> {
    let mut c = conn(pool)?;
    Ok(dmc_packages::table
        .order(dmc_packages::name.asc())
        .load::<Package>(&mut c)?)
}

pub fn get_package(pool: &DbPool, id: &str) -> Result<Package> {
    let mut c = conn(pool)?;
    dmc_packages::table
        .find(id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("package"))
}

pub fn create_package(
    pool: &DbPool,
    name: &str,
    duration_days: i32,
    base_price: f64,
    currency: &str,
    description: Option<&str>,
    cover_image_file_id: Option<i32>,
) -> Result<Package> {
    let id = nid();
    let currency = validate::currency(currency)?;
    let new = NewPackage {
        id: &id,
        name,
        duration_days,
        base_price,
        currency: &currency,
        description,
        cover_image_file_id,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(dmc_packages::table)
        .values(&new)
        .execute(&mut c)?;
    dmc_packages::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn update_package(pool: &DbPool, id: &str, patch: &UpdatePackage) -> Result<Package> {
    let mut c = conn(pool)?;
    diesel::update(dmc_packages::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    dmc_packages::table
        .find(id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn delete_package(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::delete(dmc_packages::table.find(id)).execute(&mut c)?;
    Ok(())
}

// ============================================ itineraries (parent + stops) ===

pub fn list_itineraries(pool: &DbPool, package_id: &str) -> Result<Vec<Itinerary>> {
    let mut c = conn(pool)?;
    Ok(dmc_itineraries::table
        .filter(dmc_itineraries::package_id.eq(package_id))
        .order(dmc_itineraries::day_number.asc())
        .load::<Itinerary>(&mut c)?)
}

pub fn get_itinerary(pool: &DbPool, id: &str) -> Result<Itinerary> {
    let mut c = conn(pool)?;
    dmc_itineraries::table
        .find(id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("itinerary"))
}

pub fn create_itinerary(
    pool: &DbPool,
    package_id: &str,
    day_number: i32,
    title: &str,
    notes: Option<&str>,
) -> Result<Itinerary> {
    let id = nid();
    let new = NewItinerary {
        id: &id,
        package_id,
        day_number,
        title,
        notes,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(dmc_itineraries::table)
        .values(&new)
        .execute(&mut c)?;
    dmc_itineraries::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn update_itinerary(pool: &DbPool, id: &str, patch: &UpdateItinerary) -> Result<Itinerary> {
    let mut c = conn(pool)?;
    diesel::update(dmc_itineraries::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    dmc_itineraries::table
        .find(id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn delete_itinerary(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::delete(dmc_itineraries::table.find(id)).execute(&mut c)?;
    Ok(())
}

pub fn list_itinerary_stops(pool: &DbPool, itinerary_id: &str) -> Result<Vec<ItineraryStop>> {
    let mut c = conn(pool)?;
    Ok(dmc_itinerary_stops::table
        .filter(dmc_itinerary_stops::itinerary_id.eq(itinerary_id))
        .order((
            dmc_itinerary_stops::day_number.asc(),
            dmc_itinerary_stops::hour.asc(),
        ))
        .load::<ItineraryStop>(&mut c)?)
}

pub fn create_itinerary_stop(
    pool: &DbPool,
    itinerary_id: &str,
    day_number: i32,
    hour: i32,
    location_id: Option<&str>,
    activity: Option<&str>,
    notes: Option<&str>,
) -> Result<ItineraryStop> {
    let id = nid();
    let new = NewItineraryStop {
        id: &id,
        itinerary_id,
        day_number,
        hour,
        location_id,
        activity,
        notes,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(dmc_itinerary_stops::table)
        .values(&new)
        .execute(&mut c)?;
    dmc_itinerary_stops::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn update_itinerary_stop(
    pool: &DbPool,
    id: &str,
    patch: &UpdateItineraryStop,
) -> Result<ItineraryStop> {
    let mut c = conn(pool)?;
    diesel::update(dmc_itinerary_stops::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    dmc_itinerary_stops::table
        .find(id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn delete_itinerary_stop(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::delete(dmc_itinerary_stops::table.find(id)).execute(&mut c)?;
    Ok(())
}

// ================================================================ activities ===

pub fn list_activities(pool: &DbPool) -> Result<Vec<Activity>> {
    let mut c = conn(pool)?;
    Ok(dmc_activities::table
        .order(dmc_activities::name.asc())
        .load::<Activity>(&mut c)?)
}

pub fn get_activity(pool: &DbPool, id: &str) -> Result<Activity> {
    let mut c = conn(pool)?;
    dmc_activities::table
        .find(id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("activity"))
}

pub fn create_activity(
    pool: &DbPool,
    name: &str,
    category: Option<&str>,
    duration_minutes: i32,
    price: f64,
    currency: &str,
    supplier_id: Option<&str>,
    description: Option<&str>,
) -> Result<Activity> {
    let id = nid();
    let currency = validate::currency(currency)?;
    let new = NewActivity {
        id: &id,
        name,
        category,
        duration_minutes,
        price,
        currency: &currency,
        supplier_id,
        description,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(dmc_activities::table)
        .values(&new)
        .execute(&mut c)?;
    dmc_activities::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn update_activity(pool: &DbPool, id: &str, patch: &UpdateActivity) -> Result<Activity> {
    let mut c = conn(pool)?;
    diesel::update(dmc_activities::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    dmc_activities::table
        .find(id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn delete_activity(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::delete(dmc_activities::table.find(id)).execute(&mut c)?;
    Ok(())
}

// ============================================================ accommodation ===

pub fn list_accommodation(pool: &DbPool) -> Result<Vec<Accommodation>> {
    let mut c = conn(pool)?;
    Ok(dmc_accommodation::table
        .order(dmc_accommodation::name.asc())
        .load::<Accommodation>(&mut c)?)
}

pub fn get_accommodation(pool: &DbPool, id: &str) -> Result<Accommodation> {
    let mut c = conn(pool)?;
    dmc_accommodation::table
        .find(id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("accommodation"))
}

pub fn create_accommodation(
    pool: &DbPool,
    name: &str,
    destination_id: Option<&str>,
    supplier_id: Option<&str>,
    room_type: Option<&str>,
    capacity: i32,
    nightly_rate: f64,
    currency: &str,
) -> Result<Accommodation> {
    let id = nid();
    let currency = validate::currency(currency)?;
    let new = NewAccommodation {
        id: &id,
        name,
        destination_id,
        supplier_id,
        room_type,
        capacity,
        nightly_rate,
        currency: &currency,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(dmc_accommodation::table)
        .values(&new)
        .execute(&mut c)?;
    dmc_accommodation::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn update_accommodation(
    pool: &DbPool,
    id: &str,
    patch: &UpdateAccommodation,
) -> Result<Accommodation> {
    let mut c = conn(pool)?;
    diesel::update(dmc_accommodation::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    dmc_accommodation::table
        .find(id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn delete_accommodation(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::delete(dmc_accommodation::table.find(id)).execute(&mut c)?;
    Ok(())
}

// ============================================================== transport ===

pub fn list_transport(pool: &DbPool) -> Result<Vec<Transport>> {
    let mut c = conn(pool)?;
    Ok(dmc_transport::table
        .order(dmc_transport::mode.asc())
        .load::<Transport>(&mut c)?)
}

pub fn get_transport(pool: &DbPool, id: &str) -> Result<Transport> {
    let mut c = conn(pool)?;
    dmc_transport::table
        .find(id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("transport"))
}

pub fn create_transport(
    pool: &DbPool,
    mode: &str,
    supplier_id: Option<&str>,
    capacity: i32,
    hourly_rate: f64,
    currency: &str,
) -> Result<Transport> {
    let id = nid();
    let currency = validate::currency(currency)?;
    let new = NewTransport {
        id: &id,
        mode,
        supplier_id,
        capacity,
        hourly_rate,
        currency: &currency,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(dmc_transport::table)
        .values(&new)
        .execute(&mut c)?;
    dmc_transport::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn update_transport(pool: &DbPool, id: &str, patch: &UpdateTransport) -> Result<Transport> {
    let mut c = conn(pool)?;
    diesel::update(dmc_transport::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    dmc_transport::table
        .find(id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn delete_transport(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::delete(dmc_transport::table.find(id)).execute(&mut c)?;
    Ok(())
}

// ========================================================== supplier_categories ===

pub fn list_supplier_categories(pool: &DbPool) -> Result<Vec<SupplierCategory>> {
    let mut c = conn(pool)?;
    Ok(dmc_supplier_categories::table.load::<SupplierCategory>(&mut c)?)
}

pub fn upsert_supplier_category(
    pool: &DbPool,
    supplier_id: &str,
    category: &str,
    destination_id: Option<&str>,
) -> Result<SupplierCategory> {
    let new = NewSupplierCategory {
        supplier_id,
        category,
        destination_id,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(dmc_supplier_categories::table)
        .values(&new)
        .on_conflict_do_nothing()
        .execute(&mut c)?;
    dmc_supplier_categories::table
        .find((supplier_id, category))
        .first(&mut c)
        .map_err(Into::into)
}

pub fn delete_supplier_category(pool: &DbPool, supplier_id: &str, category: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::delete(dmc_supplier_categories::table.find((supplier_id, category))).execute(&mut c)?;
    Ok(())
}

// ================================================================ guides ===

pub fn list_guides(pool: &DbPool) -> Result<Vec<Guide>> {
    let mut c = conn(pool)?;
    Ok(dmc_guides::table
        .order(dmc_guides::full_name.asc())
        .load::<Guide>(&mut c)?)
}

pub fn get_guide(pool: &DbPool, id: &str) -> Result<Guide> {
    let mut c = conn(pool)?;
    dmc_guides::table
        .find(id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("guide"))
}

pub fn create_guide(
    pool: &DbPool,
    full_name: &str,
    languages: Option<&str>,
    daily_rate: f64,
    currency: &str,
    phone: Option<&str>,
    email: Option<&str>,
    supplier_id: Option<&str>,
) -> Result<Guide> {
    let id = nid();
    let currency = validate::currency(currency)?;
    let new = NewGuide {
        id: &id,
        full_name,
        languages,
        daily_rate,
        currency: &currency,
        phone,
        email,
        supplier_id,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(dmc_guides::table)
        .values(&new)
        .execute(&mut c)?;
    dmc_guides::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn update_guide(pool: &DbPool, id: &str, patch: &UpdateGuide) -> Result<Guide> {
    let mut c = conn(pool)?;
    diesel::update(dmc_guides::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    dmc_guides::table.find(id).first(&mut c).map_err(Into::into)
}

pub fn delete_guide(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::delete(dmc_guides::table.find(id)).execute(&mut c)?;
    Ok(())
}

// ================================================================ trips ===

pub fn list_trips(pool: &DbPool) -> Result<Vec<Trip>> {
    let mut c = conn(pool)?;
    Ok(dmc_trips::table
        .order(dmc_trips::start_date.desc())
        .load::<Trip>(&mut c)?)
}

pub fn get_trip(pool: &DbPool, id: &str) -> Result<Trip> {
    let mut c = conn(pool)?;
    dmc_trips::table
        .find(id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("trip"))
}

pub fn create_trip(
    pool: &DbPool,
    name: &str,
    package_id: Option<&str>,
    lead_id: Option<&str>,
    customer_id: Option<&str>,
    start_date: &str,
    end_date: &str,
    pax_count: i32,
    status: &str,
    notes: Option<&str>,
) -> Result<Trip> {
    let id = nid();
    let start_dt = NaiveDateTime::parse_from_str(start_date, "%Y-%m-%d")
        .or_else(|_| NaiveDateTime::parse_from_str(start_date, "%Y-%m-%dT%H:%M:%S"))
        .or_else(|_| NaiveDateTime::parse_from_str(start_date, "%Y-%m-%d %H:%M:%S"))
        .map_err(|_| Error::Invalid("invalid start_date format".into()))?;
    let end_dt = NaiveDateTime::parse_from_str(end_date, "%Y-%m-%d")
        .or_else(|_| NaiveDateTime::parse_from_str(end_date, "%Y-%m-%dT%H:%M:%S"))
        .or_else(|_| NaiveDateTime::parse_from_str(end_date, "%Y-%m-%d %H:%M:%S"))
        .map_err(|_| Error::Invalid("invalid end_date format".into()))?;
    let new = NewTrip {
        id: id.clone(),
        name: name.to_string(),
        package_id: package_id.map(String::from),
        lead_id: lead_id.map(String::from),
        customer_id: customer_id.map(String::from),
        start_date: start_dt.and_utc(),
        end_date: end_dt.and_utc(),
        pax_count,
        status: status.to_string(),
        notes: notes.map(String::from),
    };
    let mut c = conn(pool)?;
    diesel::insert_into(dmc_trips::table)
        .values(&new)
        .execute(&mut c)?;
    dmc_trips::table.find(&id).first(&mut c).map_err(Into::into)
}

pub fn update_trip(pool: &DbPool, id: &str, patch: &UpdateTrip) -> Result<Trip> {
    let mut c = conn(pool)?;
    diesel::update(dmc_trips::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    dmc_trips::table.find(id).first(&mut c).map_err(Into::into)
}

pub fn delete_trip(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::delete(dmc_trips::table.find(id)).execute(&mut c)?;
    Ok(())
}

// ============================================================ trip_services ===

pub fn list_trip_services(pool: &DbPool, trip_id: &str) -> Result<Vec<TripService>> {
    let mut c = conn(pool)?;
    Ok(dmc_trip_services::table
        .filter(dmc_trip_services::trip_id.eq(trip_id))
        .order(dmc_trip_services::day_number.asc())
        .load::<TripService>(&mut c)?)
}

pub fn create_trip_service(
    pool: &DbPool,
    trip_id: &str,
    kind: &str,
    ref_id: Option<&str>,
    day_number: i32,
    start_time: Option<&str>,
    end_time: Option<&str>,
    unit_price: f64,
    currency: &str,
    qty: i32,
    total: f64,
    notes: Option<&str>,
) -> Result<TripService> {
    let id = nid();
    let currency = validate::currency(currency)?;
    let new = NewTripService {
        id: &id,
        trip_id,
        kind,
        ref_id,
        day_number,
        start_time,
        end_time,
        unit_price,
        currency: &currency,
        qty,
        total,
        notes,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(dmc_trip_services::table)
        .values(&new)
        .execute(&mut c)?;
    dmc_trip_services::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn update_trip_service(
    pool: &DbPool,
    id: &str,
    patch: &UpdateTripService,
) -> Result<TripService> {
    let mut c = conn(pool)?;
    diesel::update(dmc_trip_services::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    dmc_trip_services::table
        .find(id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn delete_trip_service(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::delete(dmc_trip_services::table.find(id)).execute(&mut c)?;
    Ok(())
}

// ================================================================ quotes ===

pub fn list_quotes(pool: &DbPool, trip_id: &str) -> Result<Vec<Quote>> {
    let mut c = conn(pool)?;
    Ok(dmc_quotes::table
        .filter(dmc_quotes::trip_id.eq(trip_id))
        .order(dmc_quotes::version.desc())
        .load::<Quote>(&mut c)?)
}

pub fn get_quote(pool: &DbPool, id: &str) -> Result<Quote> {
    let mut c = conn(pool)?;
    dmc_quotes::table
        .find(id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("quote"))
}

pub fn create_quote(
    pool: &DbPool,
    trip_id: &str,
    version: i32,
    currency: &str,
    subtotal: f64,
    tax: f64,
    total: f64,
    status: &str,
) -> Result<Quote> {
    let id = nid();
    let currency = validate::currency(currency)?;
    let new = NewQuote {
        id: &id,
        trip_id,
        version,
        currency: &currency,
        subtotal,
        tax,
        total,
        status,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(dmc_quotes::table)
        .values(&new)
        .execute(&mut c)?;
    dmc_quotes::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn update_quote(pool: &DbPool, id: &str, patch: &UpdateQuote) -> Result<Quote> {
    let mut c = conn(pool)?;
    diesel::update(dmc_quotes::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    dmc_quotes::table.find(id).first(&mut c).map_err(Into::into)
}

pub fn delete_quote(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::delete(dmc_quotes::table.find(id)).execute(&mut c)?;
    Ok(())
}

/// Compute a fresh quote from a trip's line items (services). Tax is left at
/// zero — pricing rules / VAT are an admin-configured concern that's a future
/// ticket. The currency is taken from the trip's first service (assumed
/// uniform) or TZS as a safe default.
pub fn calculate_quote(pool: &DbPool, trip_id: &str) -> Result<(String, f64, f64, f64)> {
    let services = list_trip_services(pool, trip_id)?;
    let subtotal: f64 = services.iter().map(|s| s.total).sum();
    let tax = 0.0_f64;
    let total = subtotal + tax;
    let currency = services
        .first()
        .map(|s| s.currency.clone())
        .unwrap_or_else(|| "TZS".to_string());
    Ok((currency, subtotal, tax, total))
}

pub fn accept_quote(pool: &DbPool, id: &str) -> Result<Quote> {
    let mut c = conn(pool)?;
    let n = diesel::update(dmc_quotes::table.find(id))
        .set(dmc_quotes::accepted_at.eq(Some(Utc::now().naive_utc())))
        .execute(&mut c)?;
    if n == 0 {
        return Err(Error::NotFound("quote"));
    }
    dmc_quotes::table.find(id).first(&mut c).map_err(Into::into)
}

// ================================================================ bookings ===

pub fn list_bookings(pool: &DbPool, quote_id: &str) -> Result<Vec<Booking>> {
    let mut c = conn(pool)?;
    Ok(dmc_bookings::table
        .filter(dmc_bookings::quote_id.eq(quote_id))
        .load::<Booking>(&mut c)?)
}

pub fn get_booking(pool: &DbPool, id: &str) -> Result<Booking> {
    let mut c = conn(pool)?;
    dmc_bookings::table
        .find(id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("booking"))
}

pub fn create_booking(
    pool: &DbPool,
    quote_id: &str,
    status: &str,
    deposit_paid: f64,
    balance_due: f64,
    currency: &str,
    total: f64,
) -> Result<Booking> {
    let id = nid();
    let currency = validate::currency(currency)?;
    let new = NewBooking {
        id: &id,
        quote_id,
        status,
        deposit_paid,
        balance_due,
        currency: &currency,
        total,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(dmc_bookings::table)
        .values(&new)
        .execute(&mut c)?;
    dmc_bookings::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn update_booking(pool: &DbPool, id: &str, patch: &UpdateBooking) -> Result<Booking> {
    let mut c = conn(pool)?;
    diesel::update(dmc_bookings::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    dmc_bookings::table
        .find(id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn confirm_booking(pool: &DbPool, id: &str) -> Result<Booking> {
    let mut c = conn(pool)?;
    let n = diesel::update(dmc_bookings::table.find(id))
        .set((
            dmc_bookings::status.eq("confirmed"),
            dmc_bookings::confirmed_at.eq(Some(Utc::now().naive_utc())),
        ))
        .execute(&mut c)?;
    if n == 0 {
        return Err(Error::NotFound("booking"));
    }
    dmc_bookings::table
        .find(id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn delete_booking(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::delete(dmc_bookings::table.find(id)).execute(&mut c)?;
    Ok(())
}

// ============================================== cross-module trip detail ===

/// Return a trip joined with its services and the full itinerary, suitable
/// for the trip-detail screen in the SPA. The `stops` list is flattened across
/// all the trip's package's itinerary days, with each stop's calendar date
/// resolved from `dmc_trips.start_date + (day_number - 1) days` so the SPA
/// doesn't have to know the day→date mapping.
pub struct TripDetail {
    pub trip: Trip,
    pub services: Vec<TripService>,
    pub itinerary: Vec<ItineraryStopWithDate>,
    pub destinations: Vec<Destination>,
}

pub struct ItineraryStopWithDate {
    pub stop: ItineraryStop,
    pub calendar_date: NaiveDate,
    pub location: Option<Destination>,
}

pub fn trip_detail(pool: &DbPool, trip_id: &str) -> Result<TripDetail> {
    let trip = get_trip(pool, trip_id)?;
    let services = list_trip_services(pool, trip_id)?;
    let mut c = conn(pool)?;

    // Pull the trip's package's itineraries + their stops. `trip.package_id`
    // is optional; if None there are no itineraries to pull.
    let itineraries: Vec<Itinerary> = if let Some(pkg_id) = trip.package_id.as_deref() {
        dmc_itineraries::table
            .filter(dmc_itineraries::package_id.eq(pkg_id))
            .order(dmc_itineraries::day_number.asc())
            .load::<Itinerary>(&mut c)?
    } else {
        Vec::new()
    };
    let it_ids: Vec<String> = itineraries.iter().map(|i| i.id.clone()).collect();

    let stops: Vec<ItineraryStop> = dmc_itinerary_stops::table
        .filter(dmc_itinerary_stops::itinerary_id.eq_any(&it_ids))
        .order((
            dmc_itinerary_stops::day_number.asc(),
            dmc_itinerary_stops::hour.asc(),
        ))
        .load::<ItineraryStop>(&mut c)?;

    // Resolve destination coords for each stop in one query.
    let dest_ids: Vec<String> = stops.iter().filter_map(|s| s.location_id.clone()).collect();
    let dests: Vec<Destination> = if dest_ids.is_empty() {
        Vec::new()
    } else {
        dmc_destinations::table
            .filter(dmc_destinations::id.eq_any(&dest_ids))
            .load::<Destination>(&mut c)?
    };

    let start_date = trip.start_date.naive_utc().date();

    let itinerary: Vec<ItineraryStopWithDate> = stops
        .into_iter()
        .map(|s| {
            let calendar_date =
                start_date + Duration::days((s.day_number as i64).saturating_sub(1));
            let location = s
                .location_id
                .as_ref()
                .and_then(|lid| dests.iter().find(|d| d.id == *lid).cloned());
            ItineraryStopWithDate {
                stop: s,
                calendar_date,
                location,
            }
        })
        .collect();

    Ok(TripDetail {
        trip,
        services,
        itinerary,
        destinations: dests,
    })
}

/// Trips for a given destination — used by the map preview to highlight
/// which trips touch a given city/region.
pub fn trips_for_destination(pool: &DbPool, destination_id: &str) -> Result<Vec<Trip>> {
    let mut c = conn(pool)?;
    let it_ids: Vec<String> = dmc_itinerary_stops::table
        .filter(dmc_itinerary_stops::location_id.eq(destination_id))
        .select(dmc_itinerary_stops::itinerary_id)
        .load::<String>(&mut c)?;
    if it_ids.is_empty() {
        return Ok(Vec::new());
    }
    let pkg_ids: Vec<String> = dmc_itineraries::table
        .filter(dmc_itineraries::id.eq_any(&it_ids))
        .filter(dmc_itineraries::package_id.is_not_null())
        .select(dmc_itineraries::package_id.assume_not_null())
        .load::<String>(&mut c)?;
    if pkg_ids.is_empty() {
        return Ok(Vec::new());
    }
    Ok(dmc_trips::table
        .filter(dmc_trips::package_id.eq_any(&pkg_ids))
        .load::<Trip>(&mut c)?)
}

// ==================== TOOGO lifecycle: incidents (dedicated, trip FK + cost) ====================

pub fn list_incidents(pool: &DbPool, trip_id: &str) -> Result<Vec<Incident>> {
    let mut c = conn(pool)?;
    Ok(models::schema::dmc_incidents::table
        .filter(models::schema::dmc_incidents::trip_id.eq(trip_id))
        .order(models::schema::dmc_incidents::created_at.desc())
        .load(&mut c)?)
}
pub fn create_incident(
    pool: &DbPool,
    trip_id: &str,
    type_: &str,
    severity: &str,
    title: &str,
    description: Option<&str>,
    assigned_to: Option<&str>,
    replacement_vehicle: Option<&str>,
    extra_cost: f32,
) -> Result<Incident> {
    let id = nid();
    let new = NewIncident {
        id: id.clone(),
        trip_id: trip_id.to_string(),
        type_: type_.to_string(),
        severity: severity.to_string(),
        title: title.to_string(),
        description: description.map(|s| s.to_string()),
        assigned_to: assigned_to.map(|s| s.to_string()),
        replacement_vehicle: replacement_vehicle.map(|s| s.to_string()),
        extra_cost,
        status: "OPEN".to_string(),
    };
    let mut c = conn(pool)?;
    diesel::insert_into(models::schema::dmc_incidents::table)
        .values(&new)
        .execute(&mut c)?;
    models::schema::dmc_incidents::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}
pub fn update_incident(pool: &DbPool, id: &str, patch: &UpdateIncident) -> Result<Incident> {
    let mut c = conn(pool)?;
    diesel::update(models::schema::dmc_incidents::table.find(id))
        .set(patch)
        .execute(&mut c)?;
    models::schema::dmc_incidents::table
        .find(id)
        .first(&mut c)
        .map_err(Into::into)
}
pub fn resolve_incident(pool: &DbPool, id: &str) -> Result<Incident> {
    let mut c = conn(pool)?;
    diesel::update(models::schema::dmc_incidents::table.find(id))
        .set((
            models::schema::dmc_incidents::status.eq("RESOLVED"),
            models::schema::dmc_incidents::resolved_at.eq(Some(Utc::now().naive_utc())),
        ))
        .execute(&mut c)?;
    models::schema::dmc_incidents::table
        .find(id)
        .first(&mut c)
        .map_err(Into::into)
}
pub fn delete_incident(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::delete(models::schema::dmc_incidents::table.find(id)).execute(&mut c)?;
    Ok(())
}

// documents (storage picker, reuse user_files)
pub fn list_documents(pool: &DbPool, trip_id: &str) -> Result<Vec<Document>> {
    let mut c = conn(pool)?;
    Ok(models::schema::dmc_documents::table
        .filter(models::schema::dmc_documents::trip_id.eq(trip_id))
        .load(&mut c)?)
}
pub fn attach_document(
    pool: &DbPool,
    trip_id: &str,
    kind: &str,
    file_id: Option<i32>,
    collection: &str,
    filename: Option<&str>,
    is_primary: bool,
) -> Result<Document> {
    let id = nid();
    let new = NewDocument {
        id: id.clone(),
        trip_id: trip_id.to_string(),
        kind: kind.to_string(),
        file_id,
        collection: collection.to_string(),
        filename: filename.map(|s| s.to_string()),
        is_primary: if is_primary { 1i32 } else { 0i32 },
    };
    let mut c = conn(pool)?;
    diesel::insert_into(models::schema::dmc_documents::table)
        .values(&new)
        .execute(&mut c)?;
    models::schema::dmc_documents::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}
pub fn delete_document(pool: &DbPool, id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::delete(models::schema::dmc_documents::table.find(id)).execute(&mut c)?;
    Ok(())
}

// reservations
pub fn list_reservations(pool: &DbPool, trip_id: &str) -> Result<Vec<Reservation>> {
    let mut c = conn(pool)?;
    let svc_ids: Vec<String> = models::schema::dmc_trip_services::table
        .filter(models::schema::dmc_trip_services::trip_id.eq(trip_id))
        .select(models::schema::dmc_trip_services::id)
        .load(&mut c)?;
    if svc_ids.is_empty() {
        return Ok(vec![]);
    }
    Ok(models::schema::dmc_reservations::table
        .filter(models::schema::dmc_reservations::trip_service_id.eq_any(&svc_ids))
        .load(&mut c)?)
}
pub fn create_reservation(
    pool: &DbPool,
    trip_service_id: &str,
    supplier_id: Option<&str>,
    external_ref: Option<&str>,
) -> Result<Reservation> {
    let id = nid();
    let new = NewReservation {
        id: id.clone(),
        trip_service_id: trip_service_id.to_string(),
        supplier_id: supplier_id.map(|s| s.to_string()),
        status: "PENDING".to_string(),
        external_ref: external_ref.map(|s| s.to_string()),
    };
    let mut c = conn(pool)?;
    diesel::insert_into(models::schema::dmc_reservations::table)
        .values(&new)
        .execute(&mut c)?;
    models::schema::dmc_reservations::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}
pub fn confirm_reservation(pool: &DbPool, id: &str) -> Result<Reservation> {
    let mut c = conn(pool)?;
    diesel::update(models::schema::dmc_reservations::table.find(id))
        .set((
            models::schema::dmc_reservations::status.eq("CONFIRMED"),
            models::schema::dmc_reservations::confirmed_at.eq(Some(Utc::now().naive_utc())),
        ))
        .execute(&mut c)?;
    models::schema::dmc_reservations::table
        .find(id)
        .first(&mut c)
        .map_err(Into::into)
}

// supplier requests
pub fn list_supplier_requests(pool: &DbPool, trip_id: &str) -> Result<Vec<SupplierRequest>> {
    let mut c = conn(pool)?;
    Ok(models::schema::dmc_supplier_requests::table
        .filter(models::schema::dmc_supplier_requests::trip_id.eq(trip_id))
        .load(&mut c)?)
}
pub fn create_supplier_request(
    pool: &DbPool,
    trip_id: &str,
    supplier_id: &str,
    kind: &str,
    qty: f32,
    cost: f32,
    selling: f32,
) -> Result<SupplierRequest> {
    let id = nid();
    let margin = selling - cost;
    let new = NewSupplierRequest {
        id: id.clone(),
        trip_id: trip_id.to_string(),
        supplier_id: supplier_id.to_string(),
        kind: kind.to_string(),
        qty,
        cost,
        selling,
        margin,
        status: "REQUESTED".to_string(),
    };
    let mut c = conn(pool)?;
    diesel::insert_into(models::schema::dmc_supplier_requests::table)
        .values(&new)
        .execute(&mut c)?;
    models::schema::dmc_supplier_requests::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}
pub fn confirm_supplier_request(pool: &DbPool, id: &str) -> Result<SupplierRequest> {
    let mut c = conn(pool)?;
    diesel::update(models::schema::dmc_supplier_requests::table.find(id))
        .set(models::schema::dmc_supplier_requests::status.eq("CONFIRMED"))
        .execute(&mut c)?;
    // side-effect: create erp_purchase_orders entry reuse existing ERP
    let req: SupplierRequest = models::schema::dmc_supplier_requests::table
        .find(id)
        .first(&mut c)?;
    let po_id = nid();
    let po_no = format!("PO-DMC-{}", &po_id[0..8].to_uppercase());
    diesel::insert_into(models::schema::erp_purchase_orders::table)
        .values((
            models::schema::erp_purchase_orders::id.eq(po_id.clone()),
            models::schema::erp_purchase_orders::po_number.eq(po_no),
            models::schema::erp_purchase_orders::supplier_id.eq(req.supplier_id.clone()),
            models::schema::erp_purchase_orders::status.eq("CONFIRMED"),
        ))
        .execute(&mut c)
        .ok();
    models::schema::dmc_supplier_requests::table
        .find(id)
        .first(&mut c)
        .map_err(Into::into)
}

// payments + finance posting to accounting_invoices
pub fn list_payments(pool: &DbPool, booking_id: &str) -> Result<Vec<Payment>> {
    let mut c = conn(pool)?;
    Ok(models::schema::dmc_payments::table
        .filter(models::schema::dmc_payments::booking_id.eq(booking_id))
        .load(&mut c)?)
}

pub fn create_payment(
    pool: &DbPool,
    booking_id: &str,
    kind: &str,
    amount: f32,
    currency: &str,
    method: Option<&str>,
) -> Result<Payment> {
    let id = nid();

    let new = NewPayment {
        id: id.clone(),
        booking_id: booking_id.to_string(),
        kind: kind.to_string(),
        amount,
        currency: currency.to_string(),
        method: method.map(|s| s.to_string()),
        status: "PENDING".to_string(),
    };
    let mut c = conn(pool)?;
    diesel::insert_into(models::schema::dmc_payments::table)
        .values(&new)
        .execute(&mut c)?;

    models::schema::dmc_payments::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}

pub fn check_payment_status(
    pool: &DbPool,
    id: &str,
) -> Result<Payment> {
    let mut c = conn(pool)?;
    models::schema::dmc_payments::table.find(id).first(&mut c).map_err(Into::into)
}

pub fn confirm_payment(pool: &DbPool, id: &str) -> Result<Payment> {
    let mut c = conn(pool)?;
    diesel::update(models::schema::dmc_payments::table.find(id))
        .set((
            models::schema::dmc_payments::status.eq("PAID"),
            models::schema::dmc_payments::paid_at.eq(Some(Utc::now().naive_utc())),
        ))
        .execute(&mut c)?;
    let pay: Payment = models::schema::dmc_payments::table.find(id).first(&mut c)?;
    // post to accounting_invoices (reuse existing finance)
    let inv_id = nid();
    let inv_no = format!("INV-DMC-{}", &inv_id[0..8].to_uppercase());
    let booking: Booking = models::schema::dmc_bookings::table
        .find(&pay.booking_id)
        .first(&mut c)?;
    diesel::insert_into(models::schema::accounting_invoices::table)
        .values((
            models::schema::accounting_invoices::id.eq(inv_id),
            models::schema::accounting_invoices::invoice_number.eq(inv_no),
            models::schema::accounting_invoices::sales_order_id.eq(Option::<String>::None),
            models::schema::accounting_invoices::quote_id.eq(Some(booking.quote_id.clone())),
            models::schema::accounting_invoices::total.eq(pay.amount),
            models::schema::accounting_invoices::status.eq("PAID"),
        ))
        .execute(&mut c)
        .ok();
    models::schema::dmc_payments::table
        .find(id)
        .first(&mut c)
        .map_err(Into::into)
}

// operations / events / feedback / catalog
pub fn list_operations(pool: &DbPool, trip_id: &str) -> Result<Vec<Operation>> {
    let mut c = conn(pool)?;
    Ok(models::schema::dmc_operations::table
        .filter(models::schema::dmc_operations::trip_id.eq(trip_id))
        .load(&mut c)?)
}
pub fn create_operation(
    pool: &DbPool,
    trip_id: &str,
    day_number: i32,
    service_id: Option<&str>,
    assignee_type: &str,
    assignee_id: Option<&str>,
) -> Result<Operation> {
    let id = nid();
    let new = NewOperation {
        id: id.clone(),
        trip_id: trip_id.to_string(),
        day_number,
        service_id: service_id.map(|s| s.to_string()),
        assignee_type: assignee_type.to_string(),
        assignee_id: assignee_id.map(|s| s.to_string()),
        status: "PLANNED".to_string(),
    };
    let mut c = conn(pool)?;
    diesel::insert_into(models::schema::dmc_operations::table)
        .values(&new)
        .execute(&mut c)?;
    models::schema::dmc_operations::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}
pub fn list_events(pool: &DbPool, trip_id: &str) -> Result<Vec<TripEvent>> {
    let mut c = conn(pool)?;
    Ok(models::schema::dmc_trip_events::table
        .filter(models::schema::dmc_trip_events::trip_id.eq(trip_id))
        .load(&mut c)?)
}
pub fn create_event(
    pool: &DbPool,
    trip_id: &str,
    event_type: &str,
    scheduled_at: Option<chrono::NaiveDateTime>,
    status: &str,
) -> Result<TripEvent> {
    let id = nid();
    let new = NewTripEvent {
        id: id.clone(),
        trip_id: trip_id.to_string(),
        event_type: event_type.to_string(),
        scheduled_at,
        actual_at: None,
        status: status.to_string(),
        notes: None,
    };
    let mut c = conn(pool)?;
    diesel::insert_into(models::schema::dmc_trip_events::table)
        .values(&new)
        .execute(&mut c)?;
    models::schema::dmc_trip_events::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}
pub fn list_feedback(pool: &DbPool, trip_id: &str) -> Result<Vec<Feedback>> {
    let mut c = conn(pool)?;
    Ok(models::schema::dmc_feedback::table
        .filter(models::schema::dmc_feedback::trip_id.eq(trip_id))
        .load(&mut c)?)
}
pub fn create_feedback(
    pool: &DbPool,
    trip_id: &str,
    customer_id: Option<&str>,
    rating: i32,
    comment: Option<&str>,
) -> Result<Feedback> {
    let id = nid();
    let new = NewFeedback {
        id: id.clone(),
        trip_id: trip_id.to_string(),
        customer_id: customer_id.map(|s| s.to_string()),
        rating,
        category: None,
        comment: comment.map(|s| s.to_string()),
    };
    let mut c = conn(pool)?;
    diesel::insert_into(models::schema::dmc_feedback::table)
        .values(&new)
        .execute(&mut c)?;
    models::schema::dmc_feedback::table
        .find(&id)
        .first(&mut c)
        .map_err(Into::into)
}
#[derive(Debug, serde::Serialize, diesel::QueryableByName)]
pub struct CatalogItem {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub sku: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub name: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub description: Option<String>,
}
pub fn list_catalog_items(pool: &DbPool) -> Result<Vec<CatalogItem>> {
    let mut c = conn(pool)?;
    Ok(
        diesel::sql_query("SELECT id, sku, name, description FROM dmc_catalog_items ORDER BY name")
            .load::<CatalogItem>(&mut c)?,
    )
}

// =================================================== Apple Wallet (dmc_wallet_*) ==

pub struct WalletDbStore {
    pool: DbPool,
}
impl WalletDbStore {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}
fn wallet_rfc3339(naive: NaiveDateTime) -> String {
    DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc).to_rfc3339()
}
fn parse_since(since: Option<&str>) -> Option<DateTime<Utc>> {
    since.and_then(|s| {
        DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|d| d.with_timezone(&Utc))
    })
}
impl PassStore for WalletDbStore {
    fn upsert_pass(&self, record: PassRecord) {
        let row = NewWalletPass {
            id: Uuid::new_v4().to_string(),
            pass_type: record.pass_type.clone(),
            serial_number: record.serial.clone(),
            auth_token: record.auth_token,
            user_id: None,
            data: String::from_utf8_lossy(&record.pass_json).into_owned(),
            is_active: true,
            last_updated: record.updated_at.naive_utc(),
            created_at: Utc::now().naive_utc(),
        };
        let Ok(mut c) = conn(&self.pool) else {
            return;
        };
        let res = diesel::insert_into(dmc_wallet_passes::table)
            .values(&row)
            .on_conflict((
                dmc_wallet_passes::pass_type,
                dmc_wallet_passes::serial_number,
            ))
            .do_update()
            .set((
                dmc_wallet_passes::auth_token.eq(&row.auth_token),
                dmc_wallet_passes::data.eq(&row.data),
                dmc_wallet_passes::last_updated.eq(row.last_updated),
            ))
            .execute(&mut c);
        if let Err(e) = res {
            tracing::error!("wallet: storing pass {}: {e}", row.serial_number);
        }
    }
    fn get_pass(&self, pass_type: &str, serial: &str) -> Option<PassRecord> {
        let mut c = conn(&self.pool).ok()?;
        let row = wallet_get_row(&mut c, pass_type, serial).ok()?;
        Some(PassRecord {
            pass_type: row.pass_type,
            serial: row.serial_number,
            auth_token: row.auth_token,
            pass_json: row.data.into_bytes(),
            updated_at: row.last_updated.and_utc(),
        })
    }
    fn register_device(&self, reg: DeviceRegistration) {
        let row = NewWalletRegistration {
            id: Uuid::new_v4().to_string(),
            device_id: reg.device_id,
            pass_type: reg.pass_type,
            serial_number: reg.serial,
            push_token: Some(reg.push_token),
            created_at: Utc::now().naive_utc(),
        };
        let Ok(mut c) = conn(&self.pool) else {
            return;
        };
        let res = diesel::insert_into(dmc_wallet_registrations::table)
            .values(&row)
            .on_conflict((
                dmc_wallet_registrations::device_id,
                dmc_wallet_registrations::pass_type,
                dmc_wallet_registrations::serial_number,
            ))
            .do_update()
            .set(dmc_wallet_registrations::push_token.eq(&row.push_token))
            .execute(&mut c);
        if let Err(e) = res {
            tracing::error!("wallet: registering device {}: {e}", row.device_id);
        }
    }
    fn unregister_device(&self, device_id: &str, pass_type: &str, serial: &str) {
        let Ok(mut c) = conn(&self.pool) else {
            return;
        };
        let res = diesel::delete(
            dmc_wallet_registrations::table
                .filter(dmc_wallet_registrations::device_id.eq(device_id))
                .filter(dmc_wallet_registrations::pass_type.eq(pass_type))
                .filter(dmc_wallet_registrations::serial_number.eq(serial)),
        )
        .execute(&mut c);
        if let Err(e) = res {
            tracing::error!("wallet: unregistering device {device_id}: {e}");
        }
    }
    fn serials_updated_since(
        &self,
        device_id: &str,
        pass_type: &str,
        since: Option<&str>,
    ) -> (Vec<String>, String) {
        let Ok(mut c) = conn(&self.pool) else {
            return (Vec::new(), Utc::now().to_rfc3339());
        };
        let rows: Vec<(String, NaiveDateTime)> = dmc_wallet_registrations::table
            .inner_join(
                dmc_wallet_passes::table.on(dmc_wallet_passes::pass_type
                    .eq(dmc_wallet_registrations::pass_type)
                    .and(
                        dmc_wallet_passes::serial_number
                            .eq(dmc_wallet_registrations::serial_number),
                    )),
            )
            .filter(dmc_wallet_registrations::device_id.eq(device_id))
            .filter(dmc_wallet_registrations::pass_type.eq(pass_type))
            .select((
                dmc_wallet_passes::serial_number,
                dmc_wallet_passes::last_updated,
            ))
            .load(&mut c)
            .unwrap_or_default();
        let after = parse_since(since);
        let mut newest = after.unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
        let mut serials = Vec::new();
        for (serial, updated_at) in rows {
            let updated = updated_at.and_utc();
            if after.is_none_or(|s| updated > s) && !serials.contains(&serial) {
                serials.push(serial.clone());
            }
            if updated > newest {
                newest = updated;
            }
        }
        (serials, wallet_rfc3339(newest.naive_utc()))
    }
    fn push_tokens_for_serial(&self, pass_type: &str, serial: &str) -> Vec<String> {
        let Ok(mut c) = conn(&self.pool) else {
            return Vec::new();
        };
        dmc_wallet_registrations::table
            .filter(dmc_wallet_registrations::pass_type.eq(pass_type))
            .filter(dmc_wallet_registrations::serial_number.eq(serial))
            .select(dmc_wallet_registrations::push_token)
            .load::<Option<String>>(&mut c)
            .unwrap_or_default()
            .into_iter()
            .flatten()
            .collect()
    }
}
fn wallet_get_row(c: &mut DbConn, pass_type: &str, serial: &str) -> QueryResult<WalletPass> {
    dmc_wallet_passes::table
        .filter(dmc_wallet_passes::pass_type.eq(pass_type))
        .filter(dmc_wallet_passes::serial_number.eq(serial))
        .select(WalletPass::as_select())
        .first(c)
}
#[derive(Debug, serde::Serialize)]
pub struct WalletIssuedView {
    pub pass_type_id: String,
    pub serial_number: String,
    pub auth_token: String,
    pub download_url: String,
    pub qr_svg: String,
}
pub fn wallet_catalog() -> Vec<SamplePassInfo> {
    applewallet::samples::sample_catalog()
}
pub fn wallet_list(pool: &DbPool) -> Result<Vec<WalletPass>> {
    let mut c = conn(pool)?;
    Ok(dmc_wallet_passes::table
        .select(WalletPass::as_select())
        .order(dmc_wallet_passes::last_updated.desc())
        .load(&mut c)?)
}
pub fn wallet_get(pool: &DbPool, pass_type: &str, serial: &str) -> Result<WalletPass> {
    let mut c = conn(pool)?;
    wallet_get_row(&mut c, pass_type, serial).map_err(|_| Error::NotFound("pass"))
}
pub fn wallet_issue(
    kit: &Arc<PassKit>,
    sample_id: Option<&str>,
    custom: Option<PKPass>,
) -> Result<WalletIssuedView> {
    let mut pass = match (sample_id, custom) {
        (Some(id), _) => applewallet::samples::sample_pass_by_id(id)
            .ok_or_else(|| Error::Invalid(format!("unknown sample '{id}'")))?,
        (None, Some(custom)) => custom,
        (None, None) => return Err(Error::Invalid("provide sample_id or pass".into())),
    };
    if pass.serial.trim().is_empty() || pass.serial.starts_with("SAMPLE-") {
        pass.serial = format!("PASS-{}", Uuid::new_v4().simple());
    }
    let issued = kit.issue_updatable(&pass).map_err(Error::from)?;
    let download_url = wallet_download_url(kit, &issued.pass_type, &issued.serial);
    Ok(WalletIssuedView {
        qr_svg: kit.qr_svg(&download_url).map_err(Error::from)?,
        download_url,
        pass_type_id: issued.pass_type,
        serial_number: issued.serial,
        auth_token: issued.auth_token,
    })
}
pub fn wallet_update_json(
    pool: &DbPool,
    pass_type: &str,
    serial: &str,
    new_json: serde_json::Value,
) -> Result<()> {
    let parsed: PKPass = serde_json::from_value(new_json)
        .map_err(|e| Error::Invalid(format!("invalid pass: {e}")))?;
    if parsed.serial != serial {
        return Err(Error::Invalid(
            "pass serialNumber does not match URL".into(),
        ));
    }
    let body = serde_json::to_string(&parsed).map_err(|e| Error::Invalid(e.to_string()))?;
    let _existing = wallet_get(pool, pass_type, serial)?;
    let mut c = conn(pool)?;
    diesel::update(
        dmc_wallet_passes::table
            .filter(dmc_wallet_passes::pass_type.eq(pass_type))
            .filter(dmc_wallet_passes::serial_number.eq(serial))
    )
        .set((
            dmc_wallet_passes::data.eq(body),
            dmc_wallet_passes::last_updated.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut c)?;
    Ok(())
}
pub fn wallet_delete(pool: &DbPool, pass_type: &str, serial: &str) -> Result<()> {
    let mut c = conn(pool)?;
    let n = diesel::delete(
        dmc_wallet_passes::table
            .filter(dmc_wallet_passes::pass_type.eq(pass_type))
            .filter(dmc_wallet_passes::serial_number.eq(serial))
    ).execute(&mut c)?;
    if n == 0 {
        return Err(Error::NotFound("pass"));
    }
    diesel::delete(
        dmc_wallet_registrations::table
            .filter(dmc_wallet_registrations::pass_type.eq(pass_type))
            .filter(dmc_wallet_registrations::serial_number.eq(serial)),
    )
    .execute(&mut c)?;
    Ok(())
}
pub fn wallet_download(kit: &Arc<PassKit>, pass_type: &str, serial: &str) -> Result<Vec<u8>> {
    kit.sign_stored(pass_type, serial).map_err(Error::from)
}
pub fn wallet_download_url(kit: &Arc<PassKit>, pass_type: &str, serial: &str) -> String {
    format!(
        "{}/api/v1/wallet/pass/{pass_type}/{serial}.pkpass",
        kit.config.base_url.trim_end_matches('/')
    )
}
pub fn wallet_verify_auth(kit: &Arc<PassKit>, pass_type: &str, serial: &str, token: &str) -> bool {
    kit.verify_auth(pass_type, serial, token)
}
pub fn wallet_has_registration(
    pool: &DbPool,
    device_id: &str,
    pass_type: &str,
    serial: &str,
) -> bool {
    let Ok(mut c) = conn(pool) else {
        return false;
    };
    diesel::select(diesel::dsl::exists(
        dmc_wallet_registrations::table
            .filter(dmc_wallet_registrations::device_id.eq(device_id))
            .filter(dmc_wallet_registrations::pass_type.eq(pass_type))
            .filter(dmc_wallet_registrations::serial_number.eq(serial)),
    ))
    .get_result::<bool>(&mut c)
    .unwrap_or(false)
}
