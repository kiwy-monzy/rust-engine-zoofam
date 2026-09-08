use super::{Config, FlightRadarError, FlightResult};
use super::{
    Flight, LiveFeedRequest, LiveFeedResponse, LocationBoundaries, VisibilitySettings,
    DataSource, Service, TrafficType, Filter, AirlineFilter, AirlineFilterType, AirportFilter, AirportFilterType,
    RestrictionVisibility, TopFlightsRequest, TopFlightsResponse, Geolocation,
    NearestFlightsRequest, NearestFlightsResponse,
    FlightDetailsRequest, FlightDetailsResponse,
    LiveFlightsStatusRequest, LiveFlightsStatusResponse,
    LiveTrailRequest, LiveTrailResponse,
    HistoricTrailRequest, HistoricTrailResponse,
    PlaybackFlightRequest, PlaybackFlightResponse,
    FetchSearchIndexRequest, FetchSearchIndexResponse,
};
use prost::Message;
use reqwest::Client;
use tracing::{debug, info, warn};

const GRPC_FRAME_HEADER_SIZE: usize = 5;
const GRPC_COMPRESSION_FLAG: u8 = 0x00;

pub struct FlightRadar24Client {
    client: Client,
    config: Config,
}

impl FlightRadar24Client {
    pub fn new() -> Self {
        Self::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .user_agent(&config.user_agent)
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    fn extract_grpc_payload(data: &[u8]) -> FlightResult<Vec<u8>> {
        // Empty response - return empty payload
        if data.is_empty() {
            return Ok(Vec::new());
        }

        // Check for gRPC trailer at the end (which indicates an error or empty response)
        let trailer_start = data
            .windows("grpc-status:".len())
            .rposition(|window| window == b"grpc-status:");

        // If we find a trailer before the header, the response is just trailers (empty)
        if let Some(trailer_pos) = trailer_start {
            if trailer_pos < GRPC_FRAME_HEADER_SIZE {
                return Ok(Vec::new());
            }
        }

        let trailer_start = trailer_start.unwrap_or(data.len());
        let payload = &data[..trailer_start];

        // Handle empty payload after trailer extraction
        if payload.len() < GRPC_FRAME_HEADER_SIZE {
            return Ok(Vec::new());
        }

        let compression_flag = payload[0];
        if compression_flag != GRPC_COMPRESSION_FLAG {
            return Err(FlightRadarError::InvalidResponse(
                "Compressed payload not supported".to_string(),
            ));
        }

        let msg_length = u32::from_be_bytes([
            payload[1],
            payload[2],
            payload[3],
            payload[4],
        ]) as usize;

        // Zero-length message
        if msg_length == 0 {
            return Ok(Vec::new());
        }

        if payload.len() < GRPC_FRAME_HEADER_SIZE + msg_length {
            return Err(FlightRadarError::InvalidResponse(
                "Incomplete gRPC message".to_string(),
            ));
        }

        Ok(payload[GRPC_FRAME_HEADER_SIZE..GRPC_FRAME_HEADER_SIZE + msg_length].to_vec())
    }

    fn encode_grpc_message(msg: &impl Message) -> Vec<u8> {
        let msg_bytes = msg.encode_to_vec();
        let mut buffer = Vec::with_capacity(GRPC_FRAME_HEADER_SIZE + msg_bytes.len());
        
        // Compression flag (0x00 = no compression)
        buffer.push(GRPC_COMPRESSION_FLAG);
        
        // Message length (4 bytes, big-endian)
        buffer.extend_from_slice(&(msg_bytes.len() as u32).to_be_bytes());
        
        // Message payload
        buffer.extend_from_slice(&msg_bytes);
        
        buffer
    }

    async fn send_protobuf<T: Message + Default>(&self, method: &str, body: Vec<u8>) -> FlightResult<T> {
        let url = format!("{}/{}", self.config.base_url, method);
        debug!("Sending request to {}", url);

        // Initial delay to avoid rate limiting
        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

        const MAX_RETRIES: u32 = 3;
        const INITIAL_DELAY_MS: u64 = 2000;

        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                let base_delay = INITIAL_DELAY_MS * (2_u64.pow(attempt - 1));
                let jitter = rand::random::<u64>() % 1000;
                let delay = base_delay + jitter;
                info!("Retry attempt {}/{} after {}ms delay", attempt + 1, MAX_RETRIES, delay);
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }

            // The feed is gRPC-web, so it wants the headers a browser would
            // send. `origin`/`referer` are what the endpoint checks; without
            // them it answers with a redirect to a dead legacy feed.
            let mut request = self.client
                .post(&url)
                .header("content-type", "application/grpc-web+proto")
                .header("accept", "*/*")
                .header("accept-language", "en-US,en;q=0.9")
                .header("fr24-platform", &self.config.platform)
                .header("origin", "https://www.flightradar24.com")
                .header("referer", "https://www.flightradar24.com/")
                .header("x-grpc-web", "1")
                .header("x-user-agent", "grpc-web-javascript/0.1");

            // Only when the caller supplied one. A captured device id used to
            // be compiled in here: it is somebody's browser fingerprint, it
            // pins every deployment of this library to one session, and it has
            // no business living in a source tree.
            if !self.config.device_id.is_empty() {
                request = request.header("fr24-device-id", &self.config.device_id);
            }

            let response = request
                .body(body.clone())
                .send()
                .await?;

            if response.status() == 429 {
                warn!("Rate limited (429), will retry...");
                continue;
            }
            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                if attempt < MAX_RETRIES - 1 {
                    warn!("HTTP {}, will retry...", status);
                    continue;
                }
                return Err(FlightRadarError::InvalidResponse(
                    format!("HTTP {}: {}", status, text)
                ));
            }

            let bytes = response.bytes().await?;
            let payload = Self::extract_grpc_payload(&bytes)?;
            return Ok(T::decode(&*payload)?);
        }

        Err(FlightRadarError::RateLimited)
    }

    pub async fn get_live_feed(&self, request: &LiveFeedRequest) -> FlightResult<LiveFeedResponse> {
        let resp: LiveFeedResponse = self.send_protobuf("LiveFeed", Self::encode_grpc_message(request)).await?;
        info!("Received live feed with {} flights", resp.flights_list.len());
        Ok(resp)
    }

    pub async fn get_top_flights(&self, limit: u32) -> FlightResult<TopFlightsResponse> {
        let request = TopFlightsRequest { limit };
        let resp: TopFlightsResponse = self.send_protobuf("TopFlights", Self::encode_grpc_message(&request)).await?;
        info!("Received top flights with {} entries", resp.scoreboard_list.len());
        Ok(resp)
    }

    pub async fn get_nearest_flights(&self, lat: f32, lon: f32, radius: u32, limit: u32) -> FlightResult<NearestFlightsResponse> {
        let request = NearestFlightsRequest {
            location: Some(Geolocation { lat, lon }),
            radius,
            limit,
        };
        let resp: NearestFlightsResponse = self.send_protobuf("NearestFlights", Self::encode_grpc_message(&request)).await?;
        info!("Found {} nearest flights", resp.flights_list.len());
        Ok(resp)
    }

    pub async fn get_flight_details(&self, flight_id: u32, verbose: bool) -> FlightResult<FlightDetailsResponse> {
        let request = FlightDetailsRequest {
            flight_id,
            restriction_mode: RestrictionVisibility::RvNotVisible as i32,
            verbose,
        };
        let resp: FlightDetailsResponse = self.send_protobuf("FlightDetails", Self::encode_grpc_message(&request)).await?;
        Ok(resp)
    }

    pub async fn get_live_flight_status(&self, flight_ids: Vec<u32>) -> FlightResult<LiveFlightsStatusResponse> {
        let request = LiveFlightsStatusRequest {
            flight_ids_list: flight_ids,
        };
        let resp: LiveFlightsStatusResponse = self.send_protobuf("LiveFlightsStatus", Self::encode_grpc_message(&request)).await?;
        Ok(resp)
    }

    pub async fn get_live_trail(&self, flight_id: u32) -> FlightResult<LiveTrailResponse> {
        let request = LiveTrailRequest { flight_id };
        let resp: LiveTrailResponse = self.send_protobuf("LiveTrail", Self::encode_grpc_message(&request)).await?;
        Ok(resp)
    }

    pub async fn get_historic_trail(&self, flight_id: u32) -> FlightResult<HistoricTrailResponse> {
        let request = HistoricTrailRequest { flight_id };
        let resp: HistoricTrailResponse = self.send_protobuf("HistoricTrail", Self::encode_grpc_message(&request)).await?;
        Ok(resp)
    }

    pub async fn get_playback_flight(&self, flight_id: u32, timestamp: u64) -> FlightResult<PlaybackFlightResponse> {
        let request = PlaybackFlightRequest {
            flight_id,
            timestamp,
            restriction_mode: RestrictionVisibility::RvNotVisible as i32,
        };
        let resp: PlaybackFlightResponse = self.send_protobuf("PlaybackFlight", Self::encode_grpc_message(&request)).await?;
        Ok(resp)
    }

    pub async fn fetch_search_index(&self) -> FlightResult<FetchSearchIndexResponse> {
        let request = FetchSearchIndexRequest {};
        let resp: FetchSearchIndexResponse = self.send_protobuf("FetchSearchIndex", Self::encode_grpc_message(&request)).await?;
        Ok(resp)
    }

    pub async fn health_check(&self) -> FlightResult<crate::flight::proto::health::Pong> {
        let request = crate::flight::proto::health::Ping { a: 1, b: 2 };
        let resp: crate::flight::proto::health::Pong = self.send_protobuf("Echo", Self::encode_grpc_message(&request)).await?;
        Ok(resp)
    }

    // --- Convenience search methods ---

    pub async fn search_by_airline(&self, icao_code: &str) -> FlightResult<Vec<Flight>> {
        let mut request = self.default_live_feed_request();
        request.filters_list = Some(Filter {
            airlines_list: vec![AirlineFilter {
                icao: icao_code.to_string(),
                r#type: AirlineFilterType::OperatedBy as i32,
            }],
            ..Default::default()
        });
        let response = self.get_live_feed(&request).await?;
        Ok(response.flights_list)
    }

    pub async fn search_by_callsign(&self, callsign: &str) -> FlightResult<Vec<Flight>> {
        let mut request = self.default_live_feed_request();
        request.filters_list = Some(Filter {
            callsigns_list: vec![callsign.to_string()],
            ..Default::default()
        });
        let response = self.get_live_feed(&request).await?;
        Ok(response.flights_list)
    }

    pub async fn search_by_airport(&self, airport_code: &str, filter_type: AirportFilterType) -> FlightResult<Vec<Flight>> {
        let mut request = self.default_live_feed_request();
        request.filters_list = Some(Filter {
            airports_list: vec![AirportFilter {
                iata: airport_code.to_string(),
                country_id: 0,
                r#type: filter_type as i32,
            }],
            ..Default::default()
        });
        let response = self.get_live_feed(&request).await?;
        Ok(response.flights_list)
    }

    pub async fn search_by_destination(&self, destination: &str) -> FlightResult<Vec<Flight>> {
        let mut request = self.default_live_feed_request();
        request.filters_list = Some(Filter {
            destinations_list: vec![AirportFilter {
                iata: destination.to_string(),
                country_id: 0,
                r#type: AirportFilterType::Both as i32,
            }],
            ..Default::default()
        });
        let response = self.get_live_feed(&request).await?;
        Ok(response.flights_list)
    }

    pub fn default_live_feed_request(&self) -> LiveFeedRequest {
        LiveFeedRequest {
            bounds: Some(LocationBoundaries {
                north: 49.0,
                south: -35.0,
                west: -20.0,
                east: 60.0,
            }),
            settings: Some(VisibilitySettings {
                sources_list: vec![
                    DataSource::DsAdsb as i32,
                    DataSource::DsMlat as i32,
                    DataSource::DsFlarm as i32,
                    DataSource::DsFaa as i32,
                    DataSource::DsEstimated as i32,
                    DataSource::DsSatellite as i32,
                    DataSource::DsUat as i32,
                    DataSource::DsSpidertracks as i32,
                    DataSource::DsAus as i32,
                    DataSource::DsAieron as i32,
                ],
                services_list: vec![
                    Service::SvcPassenger as i32,
                    Service::SvcCargo as i32,
                    Service::SvcMilitaryAndGovernment as i32,
                    Service::SvcBusinessJets as i32,
                    Service::SvcGeneralAviation as i32,
                    Service::SvcHelicopters as i32,
                    Service::SvcDrones as i32,
                    Service::SvcGroundVehicles as i32,
                ],
                traffic_type: TrafficType::TtAll as i32,
                only_restricted: Some(false),
            }),
            stats: Some(true),
            limit: Some(1500),
            maxage: Some(14400),
            ..Default::default()
        }
    }
}

impl Default for FlightRadar24Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for FlightRadar24Client {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            config: self.config.clone(),
        }
    }
}
