//! **fleet** — one library for every live source the gateway follows.
//!
//! Bolt taxis, MarineTraffic vessels, FlightRadar24 aircraft and Tanzania SGR
//! trains, each of which used to be its own crate with its own error type, its
//! own HTTP client and its own idea of what a tracked thing is. They are four
//! modules here, and they all answer to one trait.
//!
//! # Layout
//!
//! | Module | What it talks to | Feature |
//! |---|---|---|
//! | [`bolt`] | Bolt's rider API (phone OTP, nearby vehicles) | `bolt` |
//! | [`marine`] | AIS marker geometry, and the MarineTraffic tile feed | `geometry` / `marine` |
//! | [`flight`] | FlightRadar24's gRPC-web feed | `flight` |
//! | [`sgr`] | Tanzania SGR timetables and seating | `sgr` |
//!
//! The shared layer is [`Source`] (what a source *is*), [`Vehicle`] (what it
//! produces) and [`Error`] (how it fails).
//!
//! # Why the features
//!
//! [`marine::geometry`] is pure arithmetic and compiles anywhere, including to
//! WASM in a browser. Everything else needs an async HTTP client, and the
//! flight feed additionally needs a protobuf compiler at build time. A map
//! client wanting only vessel shapes takes:
//!
//! ```toml
//! fleet = { path = "../vendor/fleet", default-features = false, features = ["geometry"] }
//! ```
//!
//! # Credentials
//!
//! Nothing here stores a secret and nothing here has one compiled in. A source
//! declares what it needs through [`Credential`]; the caller supplies it to
//! [`Source::fetch`] and decides where it lives.

#![forbid(unsafe_code)]

pub mod error;
#[cfg(feature = "grid")]
pub mod grid;
pub mod model;
#[cfg(feature = "proto")]
pub mod wire;
pub mod source;

#[cfg(feature = "bolt")]
pub mod bolt;
#[cfg(feature = "flight")]
pub mod flight;
#[cfg(feature = "geometry")]
pub mod marine;
#[cfg(feature = "sgr")]
pub mod sgr;

#[cfg(feature = "net")]
mod sources;

pub use error::{Error, Result};
pub use model::{Kind, Vehicle};
pub use source::{Area, Catch, Credential};

#[cfg(feature = "net")]
pub use source::Source;
#[cfg(feature = "net")]
pub use sources::all;

/// Unix milliseconds, now.
///
/// One place rather than a `chrono` call at each source, so every vehicle in a
/// catch carries a timestamp from the same clock and in the same unit.
#[cfg(feature = "net")]
pub(crate) fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}
