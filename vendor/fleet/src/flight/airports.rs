use crate::flight::airport_models::*;
use super::{FlightRadarError, FlightResult as Result};
use reqwest::Client;
use tracing::{debug, info};

/// Flight from legacy API
#[derive(Debug, Clone, serde::Deserialize)]
pub struct LegacyFlight {
    #[serde(default)]
    pub identification: Option<FlightIdentification>,
    #[serde(default)]
    pub status: Option<FlightStatusLegacy>,
    #[serde(default)]
    pub aircraft: Option<FlightAircraft>,
    #[serde(default)]
    pub owner: Option<FlightOwner>,
    #[serde(default)]
    pub airline: Option<FlightAirline>,
    #[serde(default)]
    pub airport: Option<FlightAirports>,
    #[serde(default)]
    pub time: Option<FlightTime>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FlightIdentification {
    #[serde(default)]
    pub number: Option<FlightNumber>,
    #[serde(default)]
    pub callsign: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FlightNumber {
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub alternative: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FlightStatusLegacy {
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub live: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FlightAircraft {
    #[serde(default)]
    pub model: Option<AircraftModel>,
    #[serde(default)]
    pub registration: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AircraftModel {
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FlightOwner {
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FlightAirline {
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FlightAirports {
    #[serde(default)]
    pub origin: Option<FlightAirportInfo>,
    #[serde(default)]
    pub destination: Option<FlightAirportInfo>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FlightAirportInfo {
    #[serde(default)]
    pub code: Option<AirportCodeLegacy>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AirportCodeLegacy {
    #[serde(default)]
    pub iata: Option<String>,
    #[serde(default)]
    pub icao: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FlightTime {
    #[serde(default)]
    pub scheduled: Option<FlightScheduledTime>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FlightScheduledTime {
    #[serde(default)]
    pub departure: Option<u64>,
    #[serde(default)]
    pub arrival: Option<u64>,
}

impl LegacyFlight {
    pub fn get_flight_number(&self) -> String {
        self.identification
            .as_ref()
            .and_then(|i| i.number.as_ref())
            .and_then(|n| n.default.as_ref())
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_callsign(&self) -> String {
        self.identification
            .as_ref()
            .and_then(|i| i.callsign.as_ref())
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_airline(&self) -> String {
        self.airline
            .as_ref()
            .and_then(|a| a.name.as_ref())
            .cloned()
            .unwrap_or_else(|| {
                self.owner
                    .as_ref()
                    .and_then(|o| o.name.as_ref())
                    .cloned()
                    .unwrap_or_default()
            })
    }

    pub fn get_aircraft(&self) -> String {
        self.aircraft
            .as_ref()
            .and_then(|a| a.model.as_ref())
            .and_then(|m| m.text.as_ref())
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_aircraft_code(&self) -> String {
        self.aircraft
            .as_ref()
            .and_then(|a| a.model.as_ref())
            .and_then(|m| m.code.as_ref())
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_registration(&self) -> String {
        self.aircraft
            .as_ref()
            .and_then(|a| a.registration.as_ref())
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_origin_iata(&self) -> String {
        self.airport
            .as_ref()
            .and_then(|a| a.origin.as_ref())
            .and_then(|o| o.code.as_ref())
            .and_then(|c| c.iata.as_ref())
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_destination_iata(&self) -> String {
        self.airport
            .as_ref()
            .and_then(|a| a.destination.as_ref())
            .and_then(|d| d.code.as_ref())
            .and_then(|c| c.iata.as_ref())
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_status(&self) -> String {
        self.status
            .as_ref()
            .and_then(|s| s.text.as_ref())
            .cloned()
            .unwrap_or_default()
    }

    pub fn format_scheduled_departure(&self) -> String {
        self.time
            .as_ref()
            .and_then(|t| t.scheduled.as_ref())
            .and_then(|s| s.departure)
            .and_then(|ts| chrono::DateTime::from_timestamp(ts as i64, 0))
            .map(|dt| dt.format("%H:%M").to_string())
            .unwrap_or_else(|| "N/A".to_string())
    }

    pub fn format_scheduled_arrival(&self) -> String {
        self.time
            .as_ref()
            .and_then(|t| t.scheduled.as_ref())
            .and_then(|s| s.arrival)
            .and_then(|ts| chrono::DateTime::from_timestamp(ts as i64, 0))
            .map(|dt| dt.format("%H:%M").to_string())
            .unwrap_or_else(|| "N/A".to_string())
    }
}

pub struct AirportClient {
    client: Client,
    base_url: String,
}

impl AirportClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36")
                .build()
                .expect("Failed to create HTTP client"),
            base_url: "https://www.flightradar24.com".to_string(),
        }
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T> {
        debug!("Fetching URL: {}", url);

        let response = self.client
            .get(url)
            .header("accept", "*/*")
            .header("accept-language", "en-US,en;q=0.9")
            .header("cache-control", "no-cache")
            .header("pragma", "no-cache")
            .header("referer", "https://www.flightradar24.com/")
            .header("sec-ch-ua", "\"Not:A-Brand\";v=\"99\", \"Brave\";v=\"145\", \"Chromium\";v=\"145\"")
            .header("sec-ch-ua-mobile", "?0")
            .header("sec-ch-ua-platform", "\"Windows\"")
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-origin")
            .header("sec-gpc", "1")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(FlightRadarError::InvalidResponse(format!("HTTP {status}: {text}")));
        }

        let data = response.json::<T>().await?;
        Ok(data)
    }

    /// Get list of all countries
    pub async fn get_countries(&self) -> Result<Vec<Country>> {
        let url = format!("{}/mobile/countries", self.base_url);
        let response: CountryListResponse = self.get_json(&url).await?;
        info!("Fetched {} countries", response.data.len());
        Ok(response.data)
    }

    /// Get airport details by ID (numeric) or IATA code
    pub async fn get_airport(&self, airport_id: impl ToString) -> Result<AirportData> {
        let id = airport_id.to_string();
        
        // Try the legacy API first (works with IATA codes)
        let legacy_url = format!("https://api.flightradar24.com/common/v1/airport.json?code={}&plugin[]=details", id);
        
        #[derive(serde::Deserialize)]
        struct LegacyResponse {
            result: LegacyResult,
        }
        
        #[derive(serde::Deserialize)]
        struct LegacyResult {
            response: LegacyResponseData,
        }
        
        #[derive(serde::Deserialize)]
        struct LegacyResponseData {
            airport: LegacyAirport,
        }
        
        #[derive(serde::Deserialize)]
        struct LegacyAirport {
            #[serde(rename = "pluginData")]
            plugin_data: PluginData,
        }
        
        #[derive(serde::Deserialize)]
        struct PluginData {
            details: AirportDetails,
        }
        
        #[derive(serde::Deserialize)]
        struct AirportDetails {
            #[serde(default)]
            name: String,
            #[serde(default)]
            code: AirportCode,
            #[serde(default)]
            position: AirportPosition,
            #[serde(default)]
            timezone: Option<TimezoneInfo>,
        }
        
        #[derive(serde::Deserialize, Default)]
        struct AirportCode {
            #[serde(default)]
            iata: String,
            #[serde(default)]
            icao: String,
        }
        
        #[derive(serde::Deserialize, Default)]
        struct AirportPosition {
            #[serde(default)]
            latitude: f64,
            #[serde(default)]
            longitude: f64,
            #[serde(default)]
            elevation: u32,
            #[serde(default)]
            country: CountryInfo,
            #[serde(default)]
            region: RegionInfo,
        }
        
        // Inline DTO — fields populated by serde even when unread, so silence dead_code.
        #[allow(dead_code)]
        #[derive(serde::Deserialize, Default)]
        struct CountryInfo {
            #[serde(default)]
            name: String,
            #[serde(default)]
            code: String,
            #[serde(default)]
            id: u32,
        }
        
        #[derive(serde::Deserialize, Default)]
        struct RegionInfo {
            #[serde(default)]
            city: String,
        }
        
        #[derive(serde::Deserialize, Default)]
        struct TimezoneInfo {
            #[serde(default)]
            name: String,
        }

        // Use a simple client for legacy API
        let legacy_client = reqwest::Client::new();

        let legacy_response = legacy_client
            .get(&legacy_url)
            .send()
            .await;

        match legacy_response {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<LegacyResponse>().await {
                    Ok(legacy) => {
                        let details = legacy.result.response.airport.plugin_data.details;
                        info!("Fetched airport: {}", details.name);
                        
                        // Convert legacy response to AirportData format
                        let airport_info = AirportInfo {
                            id: 0,
                            name: details.name,
                            iata: details.code.iata,
                            icao: details.code.icao,
                            elevation: details.position.elevation,
                            latitude: details.position.latitude,
                            longitude: details.position.longitude,
                            state: None,
                            faaLidCode: None,
                            countryId: details.position.country.id,
                            city: details.position.region.city,
                            timezone: details.timezone.map(|tz| tz.name).unwrap_or_default(),
                            sunrise: 0,
                            sunset: 0,
                            satelliteImage: None,
                        };
                        
                        Ok(AirportData {
                            info: airport_info,
                            images: vec![],
                            weather: None,
                            stats: None,
                            runways: vec![],
                        })
                    }
                    Err(_) => {
                        // Fall back to the new API (requires numeric ID)
                        let url = format!("{}/api/v1/airports/{}", self.base_url, id);
                        let response: AirportResponse = self.get_json(&url).await?;
                        info!("Fetched airport: {}", response.data.info.name);
                        Ok(response.data)
                    }
                }
            }
            _ => {
                // Fall back to the new API (requires numeric ID)
                let url = format!("{}/api/v1/airports/{}", self.base_url, id);
                let response: AirportResponse = self.get_json(&url).await?;
                info!("Fetched airport: {}", response.data.info.name);
                Ok(response.data)
            }
        }
    }

    /// Get airport images
    pub async fn get_airport_images(&self, airport_code: impl ToString) -> Result<Vec<AirportImage>> {
        let url = format!("{}/api/v1/airports/{}/images", self.base_url, airport_code.to_string());
        let response: AirportResponse = self.get_json(&url).await?;
        Ok(response.data.images)
    }

    /// Get airport weather
    pub async fn get_airport_weather(&self, airport_code: impl ToString) -> Result<Option<AirportWeather>> {
        let url = format!("{}/api/v1/airports/{}/weather", self.base_url, airport_code.to_string());
        
        #[derive(serde::Deserialize)]
        struct WeatherResponse {
            data: Option<AirportWeather>,
        }
        
        let response: WeatherResponse = self.get_json(&url).await?;
        Ok(response.data)
    }

    /// Get airport disruptions/status
    pub async fn get_airport_disruptions(&self, airport_code: impl ToString) -> Result<DisruptionsResponse> {
        let url = format!("{}/api/v1/airports/{}/disruptions", self.base_url, airport_code.to_string());
        let response: DisruptionsResponse = self.get_json(&url).await?;
        info!("Fetched disruptions for {}", response.data.airport.name);
        Ok(response)
    }

    /// Search airports by country name (partial match)
    pub async fn search_airports_by_country(&self, country_name: &str) -> Result<Vec<Country>> {
        let countries = self.get_countries().await?;
        let matches: Vec<Country> = countries
            .into_iter()
            .filter(|c| {
                c.name.default_name.to_lowercase().contains(&country_name.to_lowercase()) ||
                c.code.alpha2.to_lowercase() == country_name.to_lowercase() ||
                c.code.alpha3.to_lowercase() == country_name.to_lowercase()
            })
            .collect();
        Ok(matches)
    }

    /// Find country by code (alpha2 or alpha3)
    pub async fn find_country_by_code(&self, code: &str) -> Result<Option<Country>> {
        let countries = self.get_countries().await?;
        let code_upper = code.to_uppercase();
        Ok(countries
            .into_iter()
            .find(|c| c.code.alpha2 == code_upper || c.code.alpha3 == code_upper))
    }

    /// Search using the web search API (finds airports, operators, etc.)
    pub async fn search_web(&self, query: &str, limit: u32) -> Result<SearchResponse> {
        let url = format!("{}/v1/search/web/find?query={}&limit={}", self.base_url, query, limit);
        let response: SearchResponse = self.get_json(&url).await?;
        info!("Search '{}' returned {} results", query, response.results.len());
        Ok(response)
    }

    /// Search for airports using the web search API
    pub async fn search_airports(&self, query: &str) -> Result<Vec<SearchResult>> {
        let response = self.search_web(query, 50).await?;
        Ok(response.results.into_iter().filter(|r| r.is_airport()).collect())
    }

    /// Search for operators/airlines using the web search API
    pub async fn search_operators(&self, query: &str) -> Result<Vec<SearchResult>> {
        let response = self.search_web(query, 50).await?;
        Ok(response.results.into_iter().filter(|r| r.is_operator()).collect())
    }

    /// Lookup airport by IATA code - returns the airport info if found
    pub async fn lookup_airport_by_iata(&self, iata: &str) -> Result<Option<AirportData>> {
        // Try fetching airport info using the IATA code as ID first
        // If that fails, return None
        match self.get_airport_by_code(iata).await {
            Ok(airport) => Ok(Some(airport)),
            Err(_) => Ok(None),
        }
    }

    /// Get airport by code (IATA or numeric ID)
    async fn get_airport_by_code(&self, code: &str) -> Result<AirportData> {
        let url = format!("{}/api/v1/airports/{}", self.base_url, code);
        let response: AirportResponse = self.get_json(&url).await?;
        Ok(response.data)
    }

    /// Get airport departures using legacy API (fetches all pages up to 1000+)
    pub async fn get_departures(&self, airport_code: impl ToString) -> Result<Vec<LegacyFlight>> {
        let code = airport_code.to_string();
        let base_url = "https://api.flightradar24.com/common/v1/airport.json";
        
        let mut departures = Vec::new();
        let mut current_page = 1;
        let max_pages = 20; // Fetch up to 20 pages (potentially 1000+ flights if perPage is 50+)
        
        loop {
            let url = format!("{}?code={}&plugin[]=schedule&page={}", base_url, code, current_page);
            let legacy_client = reqwest::Client::new();
            let response = legacy_client.get(&url).send().await?;
            
            if !response.status().is_success() {
                return Err(FlightRadarError::InvalidResponse(format!("HTTP {}", response.status())));
            }
            
            // Parse as generic value and extract departures data
            let json: serde_json::Value = response.json().await?;
            
            let mut page_found = false;
            if let Some(data_array) = json
                .pointer("/result/response/airport/pluginData/schedule/departures/data")
                .and_then(|v| v.as_array())
            {
                page_found = true;
                for item in data_array {
                    // The JSON has {"flight": {...}} but our struct expects fields at top level
                    if let Some(flight_obj) = item.get("flight") {
                        if let Ok(flight) = serde_json::from_value(flight_obj.clone()) {
                            departures.push(flight);
                        }
                    }
                }
            }
            
            // Check for next page
            let has_next_page = json
                .pointer("/result/response/airport/pluginData/schedule/departures/hasMoreData")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            
            if !page_found || !has_next_page || current_page >= max_pages {
                break;
            }
            
            current_page += 1;
        }
        
        info!("Fetched {} departures across multiple pages", departures.len());
        Ok(departures)
    }

    /// Get airport arrivals using legacy API (fetches all pages up to 1000+)
    pub async fn get_arrivals(&self, airport_code: impl ToString) -> Result<Vec<LegacyFlight>> {
        let code = airport_code.to_string();
        let base_url = "https://api.flightradar24.com/common/v1/airport.json";
        
        let mut arrivals = Vec::new();
        let mut current_page = 1;
        let max_pages = 20; // Fetch up to 20 pages (potentially 1000+ flights if perPage is 50+)
        
        loop {
            let url = format!("{}?code={}&plugin[]=schedule&page={}", base_url, code, current_page);
            let legacy_client = reqwest::Client::new();
            let response = legacy_client.get(&url).send().await?;
            
            if !response.status().is_success() {
                return Err(FlightRadarError::InvalidResponse(format!("HTTP {}", response.status())));
            }
            
            // Parse as generic value and extract arrivals data
            let json: serde_json::Value = response.json().await?;
            
            let mut page_found = false;
            if let Some(data_array) = json
                .pointer("/result/response/airport/pluginData/schedule/arrivals/data")
                .and_then(|v| v.as_array())
            {
                page_found = true;
                for item in data_array {
                    // The JSON has {"flight": {...}} but our struct expects fields at top level
                    if let Some(flight_obj) = item.get("flight") {
                        if let Ok(flight) = serde_json::from_value(flight_obj.clone()) {
                            arrivals.push(flight);
                        }
                    }
                }
            }
            
            // Check for next page
            let has_next_page = json
                .pointer("/result/response/airport/pluginData/schedule/arrivals/hasMoreData")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            
            if !page_found || !has_next_page || current_page >= max_pages {
                break;
            }
            
            current_page += 1;
        }
        
        info!("Fetched {} arrivals across multiple pages", arrivals.len());
        Ok(arrivals)
    }

    /// Get aircraft on ground - not available in legacy API
    pub async fn get_on_ground(&self, _airport_code: impl ToString) -> Result<Vec<LegacyFlight>> {
        // Legacy API doesn't have on-ground endpoint, return empty
        Ok(vec![])
    }
}

impl Default for AirportClient {
    fn default() -> Self {
        Self::new()
    }
}
