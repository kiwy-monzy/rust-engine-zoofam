//! **fleet** — SVG vessel marker generation with text labels.
//!
//! Generates base64-encoded SVG images for vessel markers, combining the hull/arrow
//! geometry from `fleet::marine::geometry` with text labels (vessel name + class).
//! Each unique vessel type gets a cached icon to avoid regeneration.
//!
//! # Features
//!
//! - `redis-cache`: Enables Redis-backed icon caching for persistent icon storage
//!   across gateway restarts. Without this feature, icons are cached in-memory only.

pub mod icons;
pub mod markers;

pub use icons::{build_cache, generate_svg, IconCache};
pub use markers::{build, build_with_icons, Marker, Markers};
