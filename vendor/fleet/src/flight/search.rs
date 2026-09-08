//! Convenience searches over the live feed.
//!
//! Every method here used to name one country: `search_air_tanzania`,
//! `search_dar_es_salaam`, `search_moroni`, and an airport table with nine
//! Tanzanian entries compiled in. That is an application's configuration, not a
//! library's knowledge — the same call is wanted for any airline, any airport,
//! any bounding box — so the codes are parameters now and the table is gone.

use tracing::{info, warn};

use super::client::FlightRadar24Client;
use super::models::FlightInfo;
use super::{AirportFilterType, FlightResult, LocationBoundaries};

/// Pause between calls when sweeping several codes, so a batch does not read as
/// a burst and earn a 429.
const SWEEP_GAP: std::time::Duration = std::time::Duration::from_millis(1500);

pub struct FlightSearcher {
    pub client: FlightRadar24Client,
}

/// Summary statistics over a set of flights.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FlightStats {
    pub total: usize,
    pub on_ground: usize,
    pub airborne: usize,
    pub avg_altitude: f64,
    pub avg_speed: f64,
}

impl FlightStats {
    pub fn from_flights(flights: &[FlightInfo]) -> Self {
        let total = flights.len();
        let airborne_flights: Vec<_> = flights.iter().filter(|f| !f.on_ground).collect();
        let airborne = airborne_flights.len();

        // Averaged over airborne flights only: a parked aircraft reports zero
        // altitude and zero speed, and including those drags the mean towards
        // nothing in a way that says more about the apron than the sky.
        let mean = |pick: fn(&&FlightInfo) -> f64| {
            if airborne_flights.is_empty() {
                0.0
            } else {
                airborne_flights.iter().map(pick).sum::<f64>() / airborne_flights.len() as f64
            }
        };

        FlightStats {
            total,
            on_ground: total - airborne,
            airborne,
            avg_altitude: mean(|f| f.altitude as f64),
            avg_speed: mean(|f| f.speed as f64),
        }
    }
}

/// Flights touching a named set of airports, and how busy each one was.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Activity {
    pub flights: Vec<FlightInfo>,
    /// `(code, label)` to the number of flights with that airport as origin or
    /// destination. A flight between two of them counts once for each.
    pub airport_activity: std::collections::HashMap<(String, String), usize>,
    pub stats: FlightStats,
    /// How many flights the feed held in total, so a caller can see what
    /// fraction the filter kept.
    pub total_flights_in_feed: usize,
}

impl FlightSearcher {
    pub fn new() -> Self {
        Self { client: FlightRadar24Client::new() }
    }

    pub fn with_client(client: FlightRadar24Client) -> Self {
        Self { client }
    }

    /// Every flight of one airline, by ICAO code.
    pub async fn by_airline(&self, icao: &str) -> FlightResult<Vec<FlightInfo>> {
        info!("Searching flights for airline {icao}");
        let flights = self.client.search_by_airline(icao).await?;
        Ok(flights.iter().map(FlightInfo::from).collect())
    }

    /// Every flight inside a bounding box.
    pub async fn in_bounds(&self, bounds: LocationBoundaries) -> FlightResult<Vec<FlightInfo>> {
        let mut request = self.client.default_live_feed_request();
        request.bounds = Some(bounds);
        let response = self.client.get_live_feed(&request).await?;
        Ok(response.flights_list.iter().map(FlightInfo::from).collect())
    }

    /// Flights into or out of one airport, by IATA code.
    pub async fn by_airport(
        &self,
        iata: &str,
        direction: AirportFilterType,
    ) -> FlightResult<Vec<FlightInfo>> {
        let flights = self.client.search_by_airport(iata, direction).await?;
        Ok(flights.iter().map(FlightInfo::from).collect())
    }

    /// Flights bound for one airport, by IATA code.
    pub async fn by_destination(&self, iata: &str) -> FlightResult<Vec<FlightInfo>> {
        let flights = self.client.search_by_destination(iata).await?;
        Ok(flights.iter().map(FlightInfo::from).collect())
    }

    pub async fn by_callsign(&self, callsign: &str) -> FlightResult<Option<FlightInfo>> {
        let flights = self.client.search_by_callsign(callsign).await?;
        let Some(first) = flights.first() else {
            warn!("Flight {callsign} not found");
            return Ok(None);
        };
        Ok(Some(FlightInfo::from(first)))
    }

    /// Sweep several ICAO codes, keeping only those that returned anything.
    ///
    /// An airline may file under more than one code, and which one is live is
    /// not knowable in advance — so this tries each and reports what answered,
    /// rather than stopping at the first that is quiet. A code that errors is
    /// logged and skipped: one bad code must not lose the results of the rest.
    pub async fn by_airlines(&self, icaos: &[&str]) -> FlightResult<Vec<(String, Vec<FlightInfo>)>> {
        let mut results = Vec::new();
        for (i, code) in icaos.iter().enumerate() {
            if i > 0 {
                tokio::time::sleep(SWEEP_GAP).await;
            }
            match self.by_airline(code).await {
                Ok(flights) if !flights.is_empty() => {
                    info!("{} flights for {code}", flights.len());
                    results.push(((*code).to_string(), flights));
                }
                Ok(_) => {}
                Err(e) => warn!("Search for {code} failed: {e}"),
            }
        }
        Ok(results)
    }

    /// Flights touching any of `airports`, with a per-airport tally.
    ///
    /// `airports` is `(IATA code, label)` — the caller's list, because which
    /// airports are interesting is the caller's business.
    pub async fn activity_around(&self, airports: &[(&str, &str)]) -> FlightResult<Activity> {
        let request = self.client.default_live_feed_request();
        let response = self.client.get_live_feed(&request).await?;

        let mut flights = Vec::new();
        let mut airport_activity = std::collections::HashMap::new();

        for flight in &response.flights_list {
            let Some(route) = flight.extra_info.as_ref().and_then(|x| x.route.as_ref()) else {
                continue;
            };
            let find = |code: &str| airports.iter().find(|(c, _)| *c == code);
            let from = find(&route.from);
            let to = find(&route.to);
            if from.is_none() && to.is_none() {
                continue;
            }
            flights.push(FlightInfo::from(flight));
            for (code, label) in from.into_iter().chain(to) {
                *airport_activity
                    .entry(((*code).to_string(), (*label).to_string()))
                    .or_insert(0) += 1;
            }
        }

        Ok(Activity {
            stats: FlightStats::from_flights(&flights),
            flights,
            airport_activity,
            total_flights_in_feed: response.flights_list.len(),
        })
    }
}

impl Default for FlightSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for FlightSearcher {
    fn clone(&self) -> Self {
        Self { client: self.client.clone() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flight(on_ground: bool, altitude: i32, speed: i32) -> FlightInfo {
        FlightInfo { on_ground, altitude, speed, ..FlightInfo::default() }
    }

    /// Parked aircraft report zero altitude and zero speed; averaging them in
    /// would describe the apron rather than the sky.
    #[test]
    fn averages_ignore_aircraft_on_the_ground() {
        let stats = FlightStats::from_flights(&[
            flight(true, 0, 0),
            flight(false, 30_000, 450),
            flight(false, 34_000, 470),
        ]);
        assert_eq!(stats.total, 3);
        assert_eq!(stats.on_ground, 1);
        assert_eq!(stats.airborne, 2);
        assert_eq!(stats.avg_altitude, 32_000.0);
        assert_eq!(stats.avg_speed, 460.0);
    }

    /// No airborne flights must not divide by zero.
    #[test]
    fn an_all_grounded_set_averages_to_zero() {
        let stats = FlightStats::from_flights(&[flight(true, 0, 0)]);
        assert_eq!(stats.avg_altitude, 0.0);
        assert_eq!(stats.avg_speed, 0.0);
    }

    #[test]
    fn an_empty_set_is_not_a_panic() {
        let stats = FlightStats::from_flights(&[]);
        assert_eq!(stats.total, 0);
        assert_eq!(stats.avg_speed, 0.0);
    }
}
