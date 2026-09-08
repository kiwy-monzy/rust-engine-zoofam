//! Vessels: AIS marker geometry, and the MarineTraffic feed that fills it.
//!
//! The two halves are deliberately separable. [`geometry`] is pure arithmetic
//! over AIS attributes with no dependencies at all, because the map client
//! compiles it to WASM where an HTTP client and a protobuf decoder cannot go.
//! [`client`] is the network half, and only the `marine` feature pulls it in.

pub mod geometry;
pub mod ship_types;

#[cfg(feature = "marine")]
pub mod client;

pub use geometry::{MarkerMode, Vessel};
