//! The protobuf form of the domain model.
//!
//! [`crate::model::Vehicle`] is the type Rust callers hold: real `Option`s and
//! a `serde_json::Value` for the raw record. This module is
//! the same information as it goes over a wire or into a cache — generated from
//! `protos/fleet/fleet.proto`, so a Bolt taxi and an SGR train encode
//! identically even though one is JSON upstream and the other is not.
//!
//! Two types rather than one because they answer to different masters. The
//! proto file is a contract with other programs and may not change shape
//! casually; the Rust struct is ours and should stay pleasant to use. The
//! conversions below are the only place the two meet.

#![allow(missing_docs)]

include!(concat!(env!("OUT_DIR"), "/fleet.rs"));

use crate::model;

impl From<model::Kind> for Kind {
    fn from(k: model::Kind) -> Self {
        match k {
            model::Kind::Taxi => Kind::Taxi,
            model::Kind::Vessel => Kind::Vessel,
            model::Kind::Aircraft => Kind::Aircraft,
            model::Kind::Train => Kind::Train,
        }
    }
}

impl Kind {
    /// The Rust enum, or `None` for `KIND_UNSPECIFIED`.
    ///
    /// Unspecified is a real possibility on the wire — proto3 gives every enum
    /// a zero value and an older peer that has not heard of a kind sends it —
    /// so it is surfaced rather than quietly mapped to some default.
    pub fn to_model(self) -> Option<model::Kind> {
        match self {
            Kind::Unspecified => None,
            Kind::Taxi => Some(model::Kind::Taxi),
            Kind::Vessel => Some(model::Kind::Vessel),
            Kind::Aircraft => Some(model::Kind::Aircraft),
            Kind::Train => Some(model::Kind::Train),
        }
    }
}

impl From<&model::Vehicle> for Vehicle {
    fn from(v: &model::Vehicle) -> Self {
        Vehicle {
            id: v.id.clone(),
            source_id: v.source_id.clone(),
            kind: Kind::from(v.kind) as i32,
            lat: v.lat,
            lng: v.lng,
            heading: v.heading,
            name: v.name.clone(),
            sub_category: v.sub_category.clone(),
            vehicle_type: v.vehicle_type.clone(),
            icon_url: v.icon_url.clone(),
            // Serialising cannot fail for a `Value`, but an empty payload is a
            // better outcome than a panic in a poller if it somehow did.
            raw: serde_json::to_vec(&v.raw).unwrap_or_default(),
            fetched_at: v.fetched_at,
        }
    }
}

impl Vehicle {
    /// The Rust form.
    ///
    /// A raw payload that does not parse becomes `null` rather than an error:
    /// the position is the point of the record, and losing a whole vehicle
    /// because its opaque extra data was truncated would be the wrong trade.
    pub fn to_model(&self) -> Option<model::Vehicle> {
        let kind = Kind::try_from(self.kind).ok()?.to_model()?;
        Some(model::Vehicle {
            id: self.id.clone(),
            source_id: self.source_id.clone(),
            kind,
            name: self.name.clone(),
            sub_category: self.sub_category.clone(),
            lat: self.lat,
            lng: self.lng,
            heading: self.heading,
            vehicle_type: self.vehicle_type.clone(),
            icon_url: self.icon_url.clone(),
            raw: serde_json::from_slice(&self.raw).unwrap_or(serde_json::Value::Null),
            fetched_at: self.fetched_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> model::Vehicle {
        model::Vehicle {
            id: "9128".into(),
            source_id: "marine".into(),
            kind: model::Kind::Vessel,
            name: Some("MV Uhuru".into()),
            sub_category: Some("Tanker".into()),
            lat: -6.83,
            lng: 39.30,
            heading: Some(45.0),
            vehicle_type: Some("80".into()),
            icon_url: None,
            raw: serde_json::json!({ "mmsi": "677000000" }),
            fetched_at: 1_756_000_000_000,
        }
    }

    #[test]
    fn a_vehicle_survives_the_round_trip() {
        let before = sample();
        let after = Vehicle::from(&before).to_model().expect("decodes");
        assert_eq!(after.id, before.id);
        assert_eq!(after.kind, before.kind);
        assert_eq!(after.name, before.name);
        assert_eq!(after.heading, before.heading);
        assert_eq!(after.raw, before.raw);
        assert_eq!(after.fetched_at, before.fetched_at);
    }

    #[test]
    fn it_encodes_and_decodes_as_protobuf() {
        use prost::Message as _;
        let wire = Vehicle::from(&sample());
        let bytes = wire.encode_to_vec();
        let back = Vehicle::decode(&bytes[..]).expect("decodes");
        assert_eq!(back, wire);
    }

    /// An undirected vehicle must not arrive pointing north.
    #[test]
    fn an_absent_heading_stays_absent() {
        let mut v = sample();
        v.heading = None;
        let back = Vehicle::from(&v).to_model().unwrap();
        assert_eq!(back.heading, None);
    }

    /// A peer that has not heard of this kind sends zero; that is not a taxi.
    #[test]
    fn an_unspecified_kind_is_not_silently_defaulted() {
        let mut wire = Vehicle::from(&sample());
        wire.kind = Kind::Unspecified as i32;
        assert!(wire.to_model().is_none());
    }
}
