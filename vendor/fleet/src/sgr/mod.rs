//! Tanzania SGR — Standard Gauge Railway timetables and seating.
//!
//! Covers the TRC TICIDIS booking chain end to end: stations, trip search,
//! coach and seat layout, pricing, then the GEPG payment handoff.

pub mod client;
pub mod models;
pub mod seat_layout;
pub mod public;

pub use client::SgrClient;
pub use models::{BookingState, RailwayCar, Seat, Station, TicketType, TrainSet, Trip};
pub use seat_layout::{LayoutOptions, name_seats, render_svg};

/// Render a coach seat map from a `TrainSetBy` API response and a car id.
///
/// Numbers the seats first. `None` when the car is not in the response.
pub fn seat_map_svg_from_api(
    train_set: &serde_json::Value,
    car_id: i64,
    opts: &LayoutOptions,
) -> Option<String> {
    let mut ts = TrainSet::from_api_value(train_set)?;
    let car = ts.cars.iter_mut().find(|c| c.id == car_id)?;
    name_seats(&mut car.seats);
    Some(render_svg(car, opts))
}
