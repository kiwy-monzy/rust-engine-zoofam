#![allow(non_snake_case)]
// FlightRadar24's legacy /airports JSON ships everything as camelCase; we mirror
// the wire format on the struct fields so serde maps directly without renames.
// The trade-off is `non_snake_case` on each field — silenced at file scope here
// rather than 30+ individual `#[serde(rename = ...)]` attributes.

use serde::{Deserialize, Serialize};

// Country models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryListResponse {
    pub metadata: CountryMetadata,
    pub data: Vec<Country>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryMetadata {
    pub timestamp: u64,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Country {
    pub id: u32,
    pub name: CountryName,
    pub code: CountryCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryName {
    #[serde(rename = "default")]
    pub default_name: String,
    pub full: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryCode {
    #[serde(rename = "alpha2")]
    pub alpha2: String,
    #[serde(rename = "alpha3")]
    pub alpha3: String,
}

// Airport models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirportResponse {
    pub data: AirportData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirportData {
    pub info: AirportInfo,
    #[serde(default)]
    pub images: Vec<AirportImage>,
    #[serde(default)]
    pub weather: Option<AirportWeather>,
    #[serde(default)]
    pub stats: Option<AirportStats>,
    #[serde(default)]
    pub runways: Vec<AirportRunway>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirportInfo {
    pub id: u32,
    pub name: String,
    pub iata: String,
    pub icao: String,
    pub elevation: u32,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub faaLidCode: Option<String>,
    pub countryId: u32,
    pub city: String,
    pub timezone: String,
    pub sunrise: u64,
    pub sunset: u64,
    #[serde(default)]
    pub satelliteImage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirportImage {
    #[serde(default)]
    pub thumbnail: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub link: Option<String>,
    #[serde(default)]
    pub copyright: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirportWeather {
    pub timestamp: u64,
    pub condition: String,
    pub temperature: i32,
    pub wind: WindInfo,
    pub pressure: u32,
    pub dewPoint: i32,
    pub humidity: u32,
    pub metar: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindInfo {
    pub direction: u32,
    pub speed: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirportStats {
    pub summary: StatsSummary,
    #[serde(default)]
    pub flights: Vec<FlightStat>,
    #[serde(default)]
    pub busiestRoutes: Vec<BusiestRoute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsSummary {
    pub airportsServed: u32,
    pub countriesServed: u32,
    pub takeoffCount: u32,
    pub landingCount: u32,
    pub totalFlights: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightStat {
    // Fields may vary
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusiestRoute {
    pub from: String,
    pub to: String,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirportRunway {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub length: Option<u32>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub surface: Option<String>,
}

// Disruption/Status models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisruptionsResponse {
    pub data: DisruptionData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisruptionData {
    pub airport: DisruptionAirport,
    #[serde(default)]
    pub arrivals: Option<FlightStatsPeriod>,
    #[serde(default)]
    pub departures: Option<FlightStatsPeriod>,
    #[serde(default)]
    pub combined: Option<FlightStatsPeriod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisruptionAirport {
    pub id: u32,
    pub name: String,
    pub code: AirportCodeBrief,
    pub country: CountryBrief,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub latitude: Option<f64>,
    #[serde(default)]
    pub longitude: Option<f64>,
    #[serde(default)]
    pub weather: Option<DisruptionWeather>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirportCodeBrief {
    pub iata: String,
    pub icao: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryBrief {
    pub name: String,
    pub alpha2: String,
    pub alpha3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisruptionWeather {
    pub temp: TempInfo,
    pub wind: WindInfoBrief,
    pub sky: SkyInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempInfo {
    pub celsius: i32,
    pub fahrenheit: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindInfoBrief {
    pub direction: WindDirection,
    pub speed: WindSpeed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindDirection {
    pub degree: u32,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindSpeed {
    pub kmh: f64,
    pub kts: f64,
    pub mph: f64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkyInfo {
    pub condition: SkyCondition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkyCondition {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightStatsPeriod {
    #[serde(default)]
    pub yesterday: Option<DailyStats>,
    #[serde(default)]
    pub today: Option<DailyStats>,
    #[serde(default)]
    pub tomorrow: Option<DailyStats>,
    #[serde(default)]
    pub live: Option<LiveStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub delayed: u32,
    #[serde(default)]
    pub delayedPercentage: f64,
    #[serde(default)]
    pub cancelled: u32,
    #[serde(default)]
    pub cancelledPercentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveStats {
    #[serde(default)]
    pub index: f64,
    #[serde(default)]
    pub averageDelayMin: u32,
    #[serde(default)]
    pub onTime: u32,
    #[serde(default)]
    pub delayed: u32,
    #[serde(default)]
    pub delayedPercentage: f64,
    #[serde(default)]
    pub cancelled: u32,
    #[serde(default)]
    pub cancelledPercentage: f64,
    #[serde(default)]
    pub trend: String,
}

// Airport search/list models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirportListResponse {
    pub data: Vec<AirportListItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirportListItem {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub iata: Option<String>,
    #[serde(default)]
    pub icao: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub countryId: Option<u32>,
}

// Search API models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    #[serde(default)]
    pub stats: Option<SearchStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub detail: Option<SearchDetail>,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub r#match: String,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDetail {
    #[serde(default)]
    pub operator_id: Option<u32>,
    #[serde(default)]
    pub iata: Option<String>,
    #[serde(default)]
    pub icao: Option<String>,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(default)]
    pub lat: Option<f64>,
    #[serde(default)]
    pub lon: Option<f64>,
    #[serde(default)]
    pub size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStats {
    pub total: SearchStatsCount,
    pub count: SearchStatsCount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStatsCount {
    #[serde(default)]
    pub all: u32,
    #[serde(default)]
    pub airport: u32,
    #[serde(default)]
    pub operator: u32,
    #[serde(default)]
    pub live: u32,
    #[serde(default)]
    pub schedule: u32,
    #[serde(default)]
    pub aircraft: u32,
}

impl SearchResult {
    pub fn is_airport(&self) -> bool {
        self.r#type == "airport"
    }

    pub fn is_operator(&self) -> bool {
        self.r#type == "operator"
    }

    pub fn is_live(&self) -> bool {
        self.r#type == "live"
    }
}

impl std::fmt::Display for Country {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name.default_name, self.code.alpha2)
    }
}

impl AirportInfo {
    pub fn format_coordinates(&self) -> String {
        format!("{:.4}°, {:.4}°", self.latitude, self.longitude)
    }

    pub fn format_elevation(&self) -> String {
        format!("{} ft", self.elevation)
    }

    pub fn format_timezone(&self) -> String {
        self.timezone.replace('/', " / ")
    }
}

impl AirportWeather {
    pub fn format_temperature(&self) -> String {
        format!("{}°C", self.temperature)
    }

    pub fn format_wind(&self) -> String {
        format!("{}° {} kts", self.wind.direction, self.wind.speed.round())
    }

    pub fn format_humidity(&self) -> String {
        format!("{}%", self.humidity)
    }

    pub fn format_pressure(&self) -> String {
        format!("{} hPa", self.pressure)
    }
}

// Flight schedule models (departures, arrivals, on-grounds)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightScheduleResponse {
    pub data: Vec<FlightSchedule>,
    pub meta: ScheduleMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleMeta {
    pub currentPage: u32,
    #[serde(default)]
    pub previousPage: Option<i32>,
    #[serde(default)]
    pub nextPage: Option<u32>,
    #[serde(default)]
    pub hasMoreNextData: Option<bool>,
    #[serde(default)]
    pub hasMorePreviousData: Option<bool>,
    pub perPage: u32,
    pub date: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightSchedule {
    pub id: u64,
    pub flight: FlightScheduleInfo,
    #[serde(default)]
    pub status: Option<FlightStatus>,
    #[serde(default)]
    pub scheduledTime: Option<u64>,
    #[serde(default)]
    pub estimatedTime: Option<u64>,
    #[serde(default)]
    pub actualTime: Option<u64>,
    #[serde(default)]
    pub landedTime: Option<u64>,
    #[serde(default)]
    pub airportId: Option<u32>,
    #[serde(default)]
    pub divertedToId: Option<u32>,
    #[serde(default)]
    pub airlineId: Option<u32>,
    #[serde(default)]
    pub airline: Option<String>,
    #[serde(default)]
    pub aircraftType: Option<String>,
    #[serde(default)]
    pub runway: Option<String>,
    #[serde(default)]
    pub aircraft: Option<AircraftInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightScheduleInfo {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub number: Option<String>,
    #[serde(default)]
    pub callsign: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightStatus {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub isAmbiguous: Option<bool>,
    #[serde(default)]
    pub live: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AircraftInfo {
    #[serde(default)]
    pub registration: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub category: Option<u32>,
    #[serde(default)]
    pub registeredOwners: Option<String>,
}

impl FlightSchedule {
    pub fn format_scheduled_time(&self) -> String {
        self.scheduledTime
            .and_then(|t| chrono::DateTime::from_timestamp(t as i64, 0))
            .map(|dt| dt.format("%H:%M").to_string())
            .unwrap_or_else(|| "N/A".to_string())
    }

    pub fn format_estimated_time(&self) -> String {
        self.estimatedTime
            .and_then(|t| chrono::DateTime::from_timestamp(t as i64, 0))
            .map(|dt| dt.format("%H:%M").to_string())
            .unwrap_or_else(|| "N/A".to_string())
    }

    pub fn format_landed_time(&self) -> String {
        self.landedTime
            .and_then(|t| chrono::DateTime::from_timestamp(t as i64, 0))
            .map(|dt| dt.format("%H:%M").to_string())
            .unwrap_or_else(|| "N/A".to_string())
    }

    pub fn get_flight_number(&self) -> String {
        self.flight
            .number
            .clone()
            .unwrap_or_else(|| "N/A".to_string())
    }

    pub fn get_callsign(&self) -> String {
        self.flight
            .callsign
            .clone()
            .unwrap_or_else(|| "N/A".to_string())
    }

    pub fn get_airline(&self) -> String {
        self.airline.clone().unwrap_or_else(|| "N/A".to_string())
    }

    pub fn get_aircraft_type(&self) -> String {
        self.aircraftType.clone().unwrap_or_else(|| {
            self.aircraft
                .as_ref()
                .and_then(|a| a.r#type.clone())
                .unwrap_or_else(|| "N/A".to_string())
        })
    }

    pub fn get_status_name(&self) -> String {
        self.status
            .as_ref()
            .and_then(|s| s.name.clone())
            .unwrap_or_else(|| "N/A".to_string())
    }

    pub fn get_status_color(&self) -> String {
        self.status
            .as_ref()
            .and_then(|s| s.color.clone())
            .unwrap_or_else(|| "gray".to_string())
    }

    pub fn get_aircraft_registration(&self) -> String {
        self.aircraft
            .as_ref()
            .and_then(|a| a.registration.clone())
            .unwrap_or_else(|| "N/A".to_string())
    }

    pub fn get_aircraft_name(&self) -> String {
        self.aircraft
            .as_ref()
            .and_then(|a| a.name.clone())
            .unwrap_or_else(|| "N/A".to_string())
    }
}
