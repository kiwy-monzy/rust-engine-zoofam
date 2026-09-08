//! The one shape every source produces.
//!
//! A Bolt taxi, a bulk carrier, an A320 and an SGR train have almost nothing in
//! common as records — different ids, different units, different notions of
//! "heading". What a map needs from all four is identical, and that is what
//! [`Vehicle`] holds. Anything source-specific survives in [`Vehicle::raw`]
//! rather than growing a column that three of the four sources leave empty.

use serde::{Deserialize, Serialize};

/// What kind of thing is moving.
///
/// A closed enum rather than the free-text `category` string this replaces: the
/// map picks a marker shape from it, and a typo used to mean a silently
/// dot-shaped ship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Taxi,
    Vessel,
    Aircraft,
    Train,
}

impl Kind {
    /// The stable wire name. Clients switch marker geometry on this.
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Taxi => "taxi",
            Kind::Vessel => "vessel",
            Kind::Aircraft => "aircraft",
            Kind::Train => "train",
        }
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One thing, somewhere, at one moment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    /// Unique within its source, not globally. Callers that mix sources
    /// qualify it with [`Vehicle::source_id`].
    pub id: String,
    /// Which source produced this.
    pub source_id: String,
    pub kind: Kind,
    /// Display name: a vessel's name, a flight's callsign. Taxis have none.
    pub name: Option<String>,
    /// The source's own sub-classification — "Tanker", "Bolt XL".
    pub sub_category: Option<String>,
    pub lat: f64,
    pub lng: f64,
    /// Degrees clockwise from north. `None` when the source did not say, which
    /// is not the same as zero — a marker drawn at 0° that should have been
    /// undirected points every ship on the map due north.
    pub heading: Option<f64>,
    /// The source's type token, kept as text because its meaning is the
    /// source's, not ours.
    pub vehicle_type: Option<String>,
    /// Artwork the source advertised for this vehicle, as an absolute URL.
    ///
    /// Left as a URL on purpose. Downloading it and inlining it is a caller's
    /// decision — a server does it once and serves a `data:` URI; a native
    /// client may just fetch it.
    pub icon_url: Option<String>,
    /// The source record verbatim, for anything not columned above.
    pub raw: serde_json::Value,
    /// Unix milliseconds: when this library observed the vehicle, not when the
    /// source claims it moved.
    ///
    /// A plain integer rather than a `chrono` type, so the domain model stays
    /// dependency-free — the WASM map client compiles this module and has no
    /// use for a clock — and so it matches `fleet.proto` field for field.
    pub fetched_at: i64,
}

impl Vehicle {
    /// A globally unique key: source and id together.
    ///
    /// Two sources can and do use the same numeric id for different things, so
    /// anything keying a map or a database row wants this rather than `id`.
    pub fn key(&self) -> String {
        format!("{}:{}", self.source_id, self.id)
    }

    /// Whether the position is a real one.
    ///
    /// Sources emit `(0, 0)` for "unknown" surprisingly often, and Null Island
    /// is not a place any of this fleet visits. Out-of-range values come from
    /// truncated records.
    pub fn has_position(&self) -> bool {
        (-90.0..=90.0).contains(&self.lat)
            && (-180.0..=180.0).contains(&self.lng)
            && !(self.lat == 0.0 && self.lng == 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(lat: f64, lng: f64) -> Vehicle {
        Vehicle {
            id: "1".into(),
            source_id: "marine".into(),
            kind: Kind::Vessel,
            name: None,
            sub_category: None,
            lat,
            lng,
            heading: None,
            vehicle_type: None,
            icon_url: None,
            raw: serde_json::Value::Null,

            fetched_at: 0,
        }
    }

    #[test]
    fn null_island_is_not_a_position() {
        assert!(!at(0.0, 0.0).has_position(), "(0,0) means 'unknown'");
        assert!(at(-6.83, 39.30).has_position());
    }

    #[test]
    fn out_of_range_coordinates_are_rejected() {
        assert!(!at(91.0, 0.5).has_position());
        assert!(!at(0.5, 181.0).has_position());
    }

    /// Ids collide across sources; the key must not.
    #[test]
    fn the_key_is_qualified_by_source() {
        let mut a = at(1.0, 1.0);
        a.source_id = "bolt".into();
        let mut b = at(1.0, 1.0);
        b.source_id = "marine".into();
        assert_eq!(a.id, b.id);
        assert_ne!(a.key(), b.key());
    }

    #[test]
    fn kind_round_trips_through_its_wire_name() {
        for k in [Kind::Taxi, Kind::Vessel, Kind::Aircraft, Kind::Train] {
            let json = serde_json::to_string(&k).unwrap();
            assert_eq!(json, format!("\"{}\"", k.as_str()));
            assert_eq!(serde_json::from_str::<Kind>(&json).unwrap(), k);
        }
    }
}
