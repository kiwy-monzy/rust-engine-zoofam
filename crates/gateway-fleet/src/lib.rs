//! App gateway's fleet: in-memory poller cache and tile generator.
//!
//! The fleet data is now held **only in memory**. The poller populates a process-
//! local cache (`LIVE`) and a single Redis bus. The `/fleet/tiles/...` route
//! serves MVT/GeoJSON straight from that cache. No `gateway_bolt_vehicles` /
//! `gateway_marine_vessels` / `gateway_flights` tables are written.

pub mod bus;
pub mod db_ext;
pub mod iconcache;
pub mod live;
pub mod markers;
pub mod poller;
pub mod tiles;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

use crate::live::LiveFleet;

/// Process-wide live fleet cache. The poller updates it on every tick; the
/// MVT/GeoJSON tile endpoint reads from it.
static LIVE: RwLock<Option<LiveFleet>> = RwLock::new(None);

/// Replace the cached live snapshot. Called by the poller on every tick.
pub fn store_live(snapshot: LiveFleet) {
    if let Ok(mut guard) = LIVE.write() {
        *guard = Some(snapshot);
    }
}

/// Borrow the current live snapshot, cloning it out of the lock.
pub fn load_live() -> Option<LiveFleet> {
    LIVE.read().ok().and_then(|g| g.clone())
}

/// Vault secret entry — same wire shape as `api-types::VaultEntry`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    pub id: i64,
    pub service: String,
    pub kind: String,
    pub name: String,
    pub meta: serde_json::Value,
    pub is_active: bool,
    pub has_secret: bool,
    pub updated_at: String,
}

/// One vehicle, same shape as `api-types::BoltVehicle` — held only in memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoltVehicle {
    pub id: String,
    pub name: Option<String>,
    pub category: String,
    pub sub_category: Option<String>,
    pub lat: f64,
    pub lng: f64,
    pub heading: Option<f64>,
    pub vehicle_type: Option<String>,
    pub icon_url: Option<String>,
    pub raw: serde_json::Value,
    pub fetched_at: String,
}

impl BoltVehicle {
    pub fn fetched_at_dt(&self) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(&self.fetched_at)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now())
    }
}

// Re-export fleet domain types so routes don't need a separate `fleet` dep just to name them.
pub use fleet::{all, grid, Area, Catch, Credential, Error as FleetError, Kind, Source, Vehicle};
