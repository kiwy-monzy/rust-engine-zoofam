use super::Flight;
use serde::{Deserialize, Serialize};

/// Flight information for display purposes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FlightInfo {
    pub id: i32,
    pub callsign: String,
    pub flight_number: String,
    pub registration: String,
    pub aircraft_type: String,
    pub altitude: i32,
    pub speed: i32,
    pub origin: Option<String>,
    pub destination: Option<String>,
    pub squawk: i32,
    pub heading: Option<u32>,
    pub vertical_speed: Option<i32>,
    pub latitude: f32,
    pub longitude: f32,
    pub on_ground: bool,
    pub source: Option<String>,
}

impl From<&Flight> for FlightInfo {
    fn from(flight: &Flight) -> Self {
        let (
            origin,
            destination,
            flight_number,
            registration,
            aircraft_type,
            squawk,
            heading,
            vertical_speed,
        ) = if let Some(ref extra_info) = flight.extra_info {
            let (orig, dest) = if let Some(ref route) = extra_info.route {
                (Some(route.from.clone()), Some(route.to.clone()))
            } else {
                (None, None)
            };
            (
                orig,
                dest,
                extra_info.flight.clone(),
                extra_info.reg.clone(),
                extra_info.r#type.clone(),
                extra_info.squawk,
                Some(extra_info.ems_info.as_ref().map(|_| 0u32).unwrap_or(0)),
                Some(extra_info.vspeed),
            )
        } else {
            (
                None,
                None,
                String::new(),
                String::new(),
                String::new(),
                0,
                None,
                None,
            )
        };

        FlightInfo {
            id: flight.flightid,
            callsign: flight.callsign.clone(),
            flight_number,
            registration,
            aircraft_type,
            altitude: flight.alt,
            speed: flight.speed,
            origin,
            destination,
            squawk,
            heading,
            vertical_speed,
            latitude: flight.lat,
            longitude: flight.lon,
            on_ground: flight.on_ground,
            source: Some(format!("{:?}", flight.source)),
        }
    }
}

/// Trail point for flight path visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrailPoint {
    pub latitude: i32,
    pub longitude: i32,
    pub altitude: i32,
}
