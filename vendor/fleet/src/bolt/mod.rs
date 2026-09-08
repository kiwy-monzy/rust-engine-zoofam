//! Bolt Taxi — the rider-facing API.
//!
//! Stateless by design: this module never stores a session. It mints one from a
//! phone OTP ([`client::start_verification`] then [`client::confirm_verification`])
//! and hands it back for the caller to keep wherever it keeps secrets.
//!
//! Vehicle artwork is *not* baked in. Bolt hands out a fresh `icon id → URL`
//! map with every poll and the ids differ between zones, so a table compiled in
//! here would be right in one city and wrong in the next — which is why
//! [`client::fetch_vehicles_and_icons`] returns both together.

pub mod categories;
pub mod client;
pub mod country_infer;
pub mod icons;
pub mod map_view;
pub mod types;
pub mod vehicle;
pub mod service;

pub use categories::{RideCategory, parse_categories};
pub use map_view::BoltMapView;
pub use types::{Device, Session, SessionStatus};
pub use vehicle::Vehicle as BoltVehicle;

/// What Bolt's own API can go wrong with.
///
/// Kept as its own enum rather than folded into [`crate::Error`] because it is
/// the vocabulary of one upstream, and the client below is a faithful port of
/// it. [`From`] lifts it into the shared error at the module boundary, so
/// callers outside `bolt` never see it.
#[derive(Debug, thiserror::Error)]
pub enum BoltError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Bolt API: {0}")]
    ApiError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, BoltError>;

impl From<BoltError> for crate::Error {
    fn from(e: BoltError) -> Self {
        match e {
            BoltError::HttpError(e) => crate::Error::Transport(e),
            BoltError::JsonError(e) => crate::Error::Json(e),
            // Bolt reports a dead or rejected session as an ordinary API error
            // with a message, so the distinction has to be read out of the text
            // — it is the difference between "sign in again" and "try later".
            BoltError::ApiError(detail)
                if detail.contains("auth")
                    || detail.contains("token")
                    || detail.contains("session") =>
            {
                crate::Error::Unauthorized { source_id: "bolt", detail }
            }
            BoltError::ApiError(detail) => crate::Error::Malformed { source_id: "bolt", detail },
            BoltError::IoError(e) => crate::Error::Malformed {
                source_id: "bolt",
                detail: e.to_string(),
            },
        }
    }
}
