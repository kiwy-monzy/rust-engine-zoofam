//! In-memory snapshot of every live fleet source the poller has fetched.
//!
//! The poller replaces this whole struct on every tick. Tile generation and
//! admin reads both take a read lock via [`crate::load_live`].

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::BoltVehicle;

/// Per-source update timestamp (RFC 3339).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceStamp {
    pub source: String,
    pub fetched_at: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveFleet {
    pub updated_at: String,
    pub bolt: Vec<BoltVehicle>,
    pub marine: Vec<BoltVehicle>,
    pub flights: Vec<BoltVehicle>,
    pub sources: Vec<SourceStamp>,
}

impl LiveFleet {
    pub fn empty() -> Self {
        Self {
            updated_at: Utc::now().to_rfc3339(),
            bolt: Vec::new(),
            marine: Vec::new(),
            flights: Vec::new(),
            sources: Vec::new(),
        }
    }

    pub fn from_poll(
        bolt: Vec<BoltVehicle>,
        marine: Vec<BoltVehicle>,
        flights: Vec<BoltVehicle>,
    ) -> Self {
        let updated_at = Utc::now().to_rfc3339();
        let mut sources = Vec::new();
        if !bolt.is_empty() {
            sources.push(SourceStamp {
                source: "bolt".to_string(),
                fetched_at: bolt
                    .iter()
                    .map(|v| v.fetched_at.clone())
                    .max()
                    .unwrap_or_else(|| updated_at.clone()),
                count: bolt.len(),
            });
        }
        if !marine.is_empty() {
            sources.push(SourceStamp {
                source: "marine".to_string(),
                fetched_at: marine
                    .iter()
                    .map(|v| v.fetched_at.clone())
                    .max()
                    .unwrap_or_else(|| updated_at.clone()),
                count: marine.len(),
            });
        }
        if !flights.is_empty() {
            sources.push(SourceStamp {
                source: "flights".to_string(),
                fetched_at: flights
                    .iter()
                    .map(|v| v.fetched_at.clone())
                    .max()
                    .unwrap_or_else(|| updated_at.clone()),
                count: flights.len(),
            });
        }
        Self {
            updated_at,
            bolt,
            marine,
            flights,
            sources,
        }
    }

    /// Return the subset of vehicles belonging to one named source.
    pub fn source(&self, name: &str) -> &[BoltVehicle] {
        match name {
            "bolt" => &self.bolt,
            "marine" => &self.marine,
            "flights" => &self.flights,
            _ => &[],
        }
    }

    /// All known source ids in this snapshot.
    pub fn source_ids() -> &'static [&'static str] {
        &["bolt", "marine", "flights"]
    }

    /// Last fetch time as `DateTime<Utc>` (best effort).
    pub fn updated_at_dt(&self) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(&self.updated_at)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now())
    }
}
