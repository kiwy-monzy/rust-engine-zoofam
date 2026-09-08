//! FlightRadar24 — the gRPC-web feed the site itself uses.
//!
//! The wire format is protobuf over `application/grpc-web+proto`, so the
//! message types here are generated at build time from [`protos/`] rather than
//! hand-written. That is also why this module is behind a feature: it needs
//! `protoc`, which a WASM build has no use for.
//!
//! No airport table is compiled in. An earlier version carried several hundred
//! lines of hardcoded airports per country, which went stale the moment a
//! runway changed; [`airports`] asks the live API instead.

pub mod airport_models;
pub mod airports;
pub mod client;
pub mod models;
pub mod search;
pub mod live;

/// Message types generated from the `.proto` definitions at build time.
pub mod proto {
    #![allow(clippy::doc_overindented_list_items, missing_docs)]

    pub mod common {
        include!(concat!(env!("OUT_DIR"), "/common.rs"));
    }
    pub mod live_feed {
        include!(concat!(env!("OUT_DIR"), "/live_feed.rs"));
    }
    pub mod top_flights {
        include!(concat!(env!("OUT_DIR"), "/top_flights.rs"));
    }
    pub mod health {
        include!(concat!(env!("OUT_DIR"), "/health.rs"));
    }
    pub mod flight_details {
        include!(concat!(env!("OUT_DIR"), "/flight_details.rs"));
    }
    pub mod follow_flight {
        include!(concat!(env!("OUT_DIR"), "/follow_flight.rs"));
    }
    pub mod historic_trail {
        include!(concat!(env!("OUT_DIR"), "/historic_trail.rs"));
    }
    pub mod live_flight_status {
        include!(concat!(env!("OUT_DIR"), "/live_flight_status.rs"));
    }
    pub mod live_trail {
        include!(concat!(env!("OUT_DIR"), "/live_trail.rs"));
    }
    pub mod nearest_flights {
        include!(concat!(env!("OUT_DIR"), "/nearest_flights.rs"));
    }
    pub mod playback_flight {
        include!(concat!(env!("OUT_DIR"), "/playback_flight.rs"));
    }
    pub mod fetch_search_index {
        include!(concat!(env!("OUT_DIR"), "/fetch_search_index.rs"));
    }
}

pub use client::FlightRadar24Client;
pub use models::FlightInfo;
pub use search::{Activity, FlightSearcher, FlightStats};

pub use proto::common::{
    DataSource, EmsInfo, ExtraFlightInfo, Flight, Icon, RestrictionVisibility,
    Route as FlightRoute, Service, Stats, Status, TrafficType,
};
pub use proto::fetch_search_index::{FetchSearchIndexRequest, FetchSearchIndexResponse};
pub use proto::flight_details::{FlightDetailsRequest, FlightDetailsResponse};
pub use proto::follow_flight::{FollowFlightRequest, FollowFlightResponse};
pub use proto::health::{Ping as HealthPing, Pong as HealthPong};
pub use proto::historic_trail::{HistoricTrailRequest, HistoricTrailResponse};
pub use proto::live_feed::{
    AirlineFilter, AirlineFilterType, AirportFilter, AirportFilterType, Filter, LiveFeedRequest,
    LiveFeedResponse, LocationBoundaries, PlaybackRequest, PlaybackResponse, VisibilitySettings,
};
pub use proto::live_flight_status::{LiveFlightsStatusRequest, LiveFlightsStatusResponse};
pub use proto::live_trail::{LiveTrailRequest, LiveTrailResponse};
pub use proto::nearest_flights::{
    Geolocation, NearbyFlight, NearestFlightsRequest, NearestFlightsResponse,
};
pub use proto::playback_flight::{PlaybackFlightRequest, PlaybackFlightResponse};
pub use proto::top_flights::{TopFlightsRequest, TopFlightsResponse};

/// How to reach the feed, and who to say we are.
///
/// `device_id` used to be a captured browser fingerprint hardcoded into the
/// request headers. That is somebody's identifier, it ties every deployment of
/// this library to one browser session, and it is exactly the kind of thing
/// that should not sit in a source tree — so it is a field with an empty
/// default, sent only when a caller sets it.
#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: String,
    pub timeout_seconds: u64,
    pub user_agent: String,
    /// Sent as `fr24-device-id` when non-empty.
    pub device_id: String,
    /// Sent as `fr24-platform`.
    pub platform: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_url: "https://data-feed.flightradar24.com/fr24.feed.api.v1.Feed".to_string(),
            timeout_seconds: 30,
            user_agent: concat!("fleet/", env!("CARGO_PKG_VERSION")).to_string(),
            device_id: String::new(),
            platform: "web".to_string(),
        }
    }
}

/// What the FlightRadar24 feed can go wrong with.
///
/// Its own enum for the same reason [`crate::bolt::BoltError`] is: it is one
/// upstream's vocabulary. [`From`] lifts it into the shared error at the module
/// boundary.
#[derive(Debug, thiserror::Error)]
pub enum FlightRadarError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Protocol buffer error: {0}")]
    ProtobufError(#[from] prost::DecodeError),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("No flights found")]
    NoFlightsFound,
    #[error("Rate limited")]
    RateLimited,
}

pub type FlightResult<T> = std::result::Result<T, FlightRadarError>;

impl From<FlightRadarError> for crate::Error {
    fn from(e: FlightRadarError) -> Self {
        match e {
            FlightRadarError::HttpError(e) => crate::Error::Transport(e),
            FlightRadarError::ProtobufError(e) => crate::Error::Protobuf(e),
            FlightRadarError::RateLimited => crate::Error::RateLimited { source_id: "flights" },
            // "No flights found" is a legitimate empty result upstream, but it
            // arrives here as an error; callers get it as an empty catch rather
            // than a fault, so it must not masquerade as a broken feed.
            FlightRadarError::NoFlightsFound => crate::Error::Malformed {
                source_id: "flights",
                detail: "the feed returned no flights".to_string(),
            },
            FlightRadarError::InvalidResponse(detail) => {
                crate::Error::Malformed { source_id: "flights", detail }
            }
        }
    }
}
