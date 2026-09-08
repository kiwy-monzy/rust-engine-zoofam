//! SGR ticketing data model — ported from the TRC TICIDIS API shapes
//! (train sets, railway cars, seats) and the booking-flow cache.

use serde::{Deserialize, Serialize};

/// Seat type codes used by the SGR API.
pub mod seat_type {
    pub const SEAT: i32 = 39;       // bookable passenger seat
    pub const LUGGAGE: i32 = 40;    // luggage area
    pub const WC: i32 = 41;         // toilet
    pub const CANCELLED: i32 = 42;  // not rendered (gap)
    pub const DISABLED: i32 = 43;   // disabled passenger seat
}

/// One seat (or non-seat fixture) inside a railway car grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Seat {
    pub column: i32,
    #[serde(rename = "columnDescription", default)]
    pub column_description: String,
    #[serde(rename = "isAvailable", default)]
    pub is_available: bool,
    #[serde(rename = "seatTypeId")]
    pub seat_type_id: i32,
    pub row: i32,
    #[serde(rename = "rowDescription", default)]
    pub row_description: String,
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub rotation: i32,
    #[serde(rename = "newSeatNo", default)]
    pub new_seat_no: String,
}

impl Seat {
    pub fn is_bookable(&self) -> bool { self.seat_type_id == seat_type::SEAT }
}

/// A railway car / coach within a train set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RailwayCar {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub capacity: i32,
    #[serde(rename = "emptySeats", default)]
    pub empty_seats: i32,
    #[serde(rename = "rowCount", default)]
    pub row_count: i32,
    #[serde(rename = "columnCount", default)]
    pub column_count: i32,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "railwayCarType", default)]
    pub railway_car_type: String,
    #[serde(rename = "orderNo", default)]
    pub order_no: i32,
    #[serde(rename = "hasSeat", default)]
    pub has_seat: bool,
    #[serde(default)]
    pub seats: Vec<Seat>,
}

/// A train set (the physical train assigned to a trip).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainSet {
    #[serde(default)]
    pub id: i64,
    #[serde(rename = "trainType", default)]
    pub train_type: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub no: i64,
    #[serde(rename = "trainSetRailwayCars", default)]
    pub cars: Vec<RailwayCar>,
}

impl TrainSet {
    /// Parse a `TrainSetBy` API response (`{ data: { trainSetRailwayCars: [...] } }`)
    /// or a bare train-set object.
    pub fn from_api_value(v: &serde_json::Value) -> Option<Self> {
        let obj = v.get("data").unwrap_or(v);
        serde_json::from_value(obj.clone()).ok()
    }
    pub fn car(&self, id: i64) -> Option<&RailwayCar> { self.cars.iter().find(|c| c.id == id) }
}

/// A boarding/landing station.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub id: i64,
    /// The SGR API returns this as `title`.
    #[serde(default, alias = "title")]
    pub name: String,
    #[serde(rename = "code", default)]
    pub code: Option<String>,
}

/// A searched trip / schedule result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trip {
    #[serde(default)]
    pub id: i64,
    #[serde(rename = "tripTimeTableTrainSetId", default)]
    pub trip_time_table_train_set_id: Option<i64>,
    #[serde(rename = "trainName", default)]
    pub train_name: Option<String>,
    #[serde(rename = "departureTime", default)]
    pub departure_time: Option<String>,
    #[serde(rename = "arrivalTime", default)]
    pub arrival_time: Option<String>,
    #[serde(rename = "fromStation", default)]
    pub from_station: Option<String>,
    #[serde(rename = "toStation", default)]
    pub to_station: Option<String>,
    /// Keep the raw object so callers can read any extra fields the API returns.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// A ticket/fare type for a route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketType {
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub price: Option<f64>,
}

/// Booking-flow state — the working set a booking session accumulates
/// (ported from the Python `Caches` model). Persist this per chat/session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BookingState {
    pub access_token: Option<String>,
    pub boarding_station_id: Option<i64>,
    pub landing_station_id: Option<i64>,
    pub boarding_station_name: Option<String>,
    pub landing_station_name: Option<String>,
    pub railway_id: Option<i64>,
    pub railway_car_id: Option<String>,
    pub trip_time_table_train_set_id: Option<i64>,
    pub route_id: Option<i64>,
    pub seat_id: Option<i64>,
    pub ticket_type_id: Option<i64>,
    pub price: Option<i64>,
    pub passenger_name: Option<String>,
    pub identity_no: Option<i64>,
    pub nationality_id_type: Option<String>,
    pub passenger_count: i32,
    pub departure_date: Option<String>,
    pub return_date: Option<String>,
    pub oneway: bool,
    pub is_reservation: bool,
    pub direction_type: i32,
    // Payment / result
    pub payment_phone_number: Option<String>,
    pub payment_provider: Option<String>,
    pub payment_provider_id: Option<String>,
    pub payer_email: Option<String>,
    pub pnr_code: Option<String>,
    pub ticket_no: Option<String>,
    pub bill_id: Option<String>,
    pub control_no: Option<String>,
    pub group_reference: Option<String>,
}

impl BookingState {
    pub fn new() -> Self {
        Self { passenger_count: 1, oneway: true, ..Default::default() }
    }
}
