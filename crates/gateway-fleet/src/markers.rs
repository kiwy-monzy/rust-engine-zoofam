//! Turning stored vehicles into map markers.
//!
//! `/api/fleet/all` returns whole vehicle records — every column plus the raw
//! source document — which is right for the admin table and far too heavy for
//! a map that repaints every eight seconds with thousands of them. A [`Marker`]
//! is the drawable subset: where it is, which way it points, what colour and
//! how big.
//!
//! The classification is done **here** rather than in the client for the same
//! reason the layer style is served rather than hardcoded: every client has to
//! reach the same answer, and one that decided its own colours would drift the
//! moment either side changed.
//!
//! Nothing here needs a schema change. The extra source fields — ship type,
//! beam, altitude — already ride along in each row's `raw` document, so they
//! are read back out at request time instead of being columned.

use crate::BoltVehicle;
use serde::Serialize;

use fleet::marine::ship_types;

/// One drawable vehicle.
#[derive(Debug, Clone, Serialize)]
pub struct Marker {
    pub id: String,
    pub lat: f64,
    pub lng: f64,
    /// `ship`, `flight` or `bolt` — the domain, for filtering and the legend.
    pub kind: String,
    /// AIS class, aircraft type, or Bolt category. Shown on hover.
    pub class: String,
    /// Vessel name, flight callsign, or Bolt vehicle label — whatever the
    /// source called it. A marker without a name is a dot you cannot ask about.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Overall length in metres (AIS `A + B`). Vessels only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length_m: Option<f64>,
    /// Beam in metres (AIS `C + D`). Vessels only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width_m: Option<f64>,
    /// Speed over ground in knots — decides whether a vessel is under way or
    /// at anchor, which changes the marker shape.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed_kn: Option<f64>,
    /// Course over ground, when it differs from heading.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub course: Option<f64>,
    /// Raw AIS ship type, so the client can size and shape the hull itself.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ship_type: Option<u32>,
    /// Bow to the AIS antenna, metres (AIS `A`). With [`Self::w_left`] this
    /// places the hull correctly around the reported position; without them the
    /// hull is merely centred on it, which is wrong by up to half a ship.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l_fore: Option<f64>,
    /// Port side to the AIS antenna, metres (AIS `C`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub w_left: Option<f64>,
    /// Degrees clockwise from north, when the source knew.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading: Option<f64>,
    /// `#rrggbb`, from the AIS class for vessels and the altitude band for
    /// flights.
    pub color: String,
    /// Diameter in screen pixels.
    pub size: f32,
    /// Feet. Flights only, and the reason for their colour.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub altitude: Option<f64>,
    /// Key into the icon table returned alongside these, when the source gave
    /// an icon. Not the image itself — see [`Markers::icons`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

/// A marker set plus the icon images its members point at.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Markers {
    pub markers: Vec<Marker>,
    /// `icon key -> data: URI`.
    ///
    /// Bolt sends the same base64 image for every vehicle of a type, and there
    /// can be thousands on screen. Sending it once per *type* rather than once
    /// per vehicle is the difference between a payload that is mostly
    /// duplicated PNG and one that is mostly positions.
    pub icons: std::collections::BTreeMap<String, String>,
}

/// Reads a number out of the stored source document, tolerating the string
/// forms these APIs mix in.
fn raw_num(v: &serde_json::Value, keys: &[&str]) -> Option<f64> {
    for k in keys {
        match v.get(k) {
            Some(serde_json::Value::Number(n)) => return n.as_f64(),
            Some(serde_json::Value::String(s)) => {
                if let Ok(f) = s.trim().parse::<f64>() {
                    return Some(f);
                }
            }
            _ => {}
        }
    }
    None
}

/// The vessel's beam in metres, if the source carried one.
fn beam_of(v: &BoltVehicle) -> Option<f64> {
    raw_num(&v.raw, &["WIDTH", "width", "width_m", "BREADTH", "beam"])
}

/// The aircraft's altitude in feet, if the source carried one.
fn altitude_of(v: &BoltVehicle) -> Option<f64> {
    raw_num(
        &v.raw,
        &["alt", "ALT", "altitude", "ALTITUDE", "baro_altitude"],
    )
}

/// The AIS ship type code, if the source carried one.
fn shiptype_of(v: &BoltVehicle) -> Option<u32> {
    raw_num(&v.raw, &["SHIPTYPE", "shiptype", "ship_type", "TYPE"]).map(|n| n as u32)
}

/// Builds the marker for one stored vehicle.
///
/// `category` on the row is the domain for flights and Bolt, but for vessels it
/// is the literal string `"ship"` — the AIS class has to come back out of the
/// raw document.
pub fn marker(v: &BoltVehicle) -> Marker {
    // Both spellings: rows stored before the fleet library unified these carry
    // the old domain names, and they keep arriving until the next poll upserts
    // over them.
    let kind = match v.category.as_str() {
        "aircraft" | "flight" => "flight",
        "taxi" | "bolt" => "bolt",
        "vessel" | "ship" => "ship",
        other => other,
    };
    let (class, color, size, altitude, icon) = match kind {
        "flight" => {
            let alt = altitude_of(v);
            (
                v.vehicle_type.clone().unwrap_or_else(|| "aircraft".into()),
                ship_types::altitude_color(alt).to_string(),
                6.5_f32,
                alt,
                None,
            )
        }
        "bolt" => (
            v.vehicle_type
                .clone()
                .or_else(|| v.sub_category.clone())
                .unwrap_or_else(|| "bolt".into()),
            "#34d399".to_string(),
            7.0_f32,
            None,
            // Bolt's icon is a base64 data URI shared by every vehicle of a
            // type, so the type is the key.
            v.icon_url
                .as_ref()
                .and_then(|_| v.vehicle_type.clone().or_else(|| v.sub_category.clone())),
        ),
        // Vessels, and anything unrecognised, are classified as AIS.
        _ => {
            let code = shiptype_of(v);
            let class = code
                .map(|c| ship_types::ais_category(c).to_string())
                .or_else(|| v.sub_category.clone())
                .unwrap_or_else(|| "unknown".into());
            let color = match code {
                Some(c) => ship_types::ais_color(c).to_string(),
                None => ship_types::category_color(&class).to_string(),
            };
            (class, color, ship_types::beam_size(beam_of(v)), None, None)
        }
    };

    // The dimensions the client needs to draw a true-scale hull. Only vessels
    // carry them; a flight or a taxi has no beam worth drawing.
    let (length_m, width_m, ship_type) = match kind {
        "flight" | "bolt" => (None, None, None),
        // MarineTraffic sets INVALID_DIMENSIONS when the AIS dimensions are
        // junk. Drawing a hull from those puts a wrong-sized ship on the chart
        // with no hint that it is wrong; better to fall back to the arrow,
        // which claims nothing about size.
        _ if raw_num(&v.raw, &["INVALID_DIMENSIONS"]).unwrap_or(0.0) != 0.0 => {
            (None, None, shiptype_of(v))
        }
        _ => (
            raw_num(&v.raw, &["LENGTH", "length", "length_m", "LOA"]).filter(|l| *l > 0.0),
            beam_of(v).filter(|w| *w > 0.0),
            shiptype_of(v),
        ),
    };

    Marker {
        id: v.id.clone(),
        lat: v.lat,
        lng: v.lng,
        kind: kind.to_string(),
        class,
        // Fall back to the source's own label, then to nothing — an empty
        // string would render as a blank tooltip rather than no tooltip.
        name: v
            .name
            .clone()
            .filter(|n| !n.trim().is_empty())
            .or_else(|| raw_num(&v.raw, &["MMSI"]).map(|m| format!("MMSI {}", m as u64))),
        length_m,
        width_m,
        speed_kn: raw_num(&v.raw, &["SPEED", "speed", "speed_kn", "sog", "SOG"]),
        course: raw_num(&v.raw, &["COURSE", "course", "course_deg", "cog", "COG"]),
        ship_type,
        l_fore: raw_num(&v.raw, &["L_FORE", "l_fore"]).filter(|_| length_m.is_some()),
        w_left: raw_num(&v.raw, &["W_LEFT", "w_left"]).filter(|_| width_m.is_some()),
        heading: v.heading,
        color,
        size,
        altitude,
        icon,
    }
}

/// Builds the whole marker set, collecting each distinct icon once.
pub fn build<'a>(vehicles: impl IntoIterator<Item = &'a BoltVehicle>) -> Markers {
    let mut out = Markers::default();
    for v in vehicles {
        let m = marker(v);
        if let (Some(key), Some(url)) = (m.icon.as_ref(), v.icon_url.as_ref()) {
            if !url.is_empty() && !out.icons.contains_key(key) {
                out.icons.insert(key.clone(), url.clone());
            }
        }
        out.markers.push(m);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn vehicle(category: &str, raw: serde_json::Value) -> BoltVehicle {
        BoltVehicle {
            id: "v1".into(),
            name: None,
            category: category.into(),
            sub_category: None,
            lat: -6.8,
            lng: 39.3,
            heading: Some(90.0),
            vehicle_type: None,
            icon_url: None,
            raw,
            fetched_at: "2026-08-24T00:00:00Z".into(),
        }
    }

    #[test]
    fn a_vessel_is_coloured_and_sized_from_its_ais_record() {
        let tanker = vehicle("ship", json!({ "SHIPTYPE": 8, "WIDTH": 60 }));
        let m = marker(&tanker);
        assert_eq!(m.class, "tanker");
        assert_eq!(m.color, ship_types::ais_color(8));
        assert!(
            m.size > ship_types::beam_size(None),
            "a wide beam draws larger"
        );
        assert_eq!(m.altitude, None, "vessels have no altitude");
    }

    /// MarineTraffic sends numbers as strings about as often as numbers.
    #[test]
    fn string_numbers_in_the_raw_document_still_parse() {
        let m = marker(&vehicle("ship", json!({ "SHIPTYPE": "7", "WIDTH": "32" })));
        assert_eq!(m.class, "cargo");
        assert!(m.size > ship_types::beam_size(None));
    }

    #[test]
    fn a_flight_is_coloured_by_altitude() {
        let low = marker(&vehicle("flight", json!({ "alt": 800 })));
        let high = marker(&vehicle("flight", json!({ "alt": 35000 })));
        assert_eq!(low.kind, "flight");
        assert_eq!(low.altitude, Some(800.0));
        assert_ne!(low.color, high.color, "altitude must change the colour");
    }

    #[test]
    fn a_vehicle_with_nothing_useful_still_draws() {
        let m = marker(&vehicle("ship", json!({})));
        assert_eq!(m.class, "unknown");
        assert!(m.size >= 5.0, "must stay clickable");
        assert!(m.color.starts_with('#'));
    }

    /// The whole point of the icon table: one copy of each image, not one per
    /// vehicle.
    #[test]
    fn bolt_icons_are_sent_once_per_type_not_once_per_vehicle() {
        let mut a = vehicle("bolt", json!({}));
        a.vehicle_type = Some("taxi".into());
        a.icon_url = Some("data:image/png;base64,AAAA".into());
        let mut b = a.clone();
        b.id = "v2".into();

        let set = build([&a, &b]);
        assert_eq!(set.markers.len(), 2);
        assert_eq!(set.icons.len(), 1, "the image was repeated");
        assert_eq!(set.icons.get("taxi").unwrap(), "data:image/png;base64,AAAA");
        assert_eq!(set.markers[0].icon.as_deref(), Some("taxi"));
    }

    #[test]
    fn a_bolt_vehicle_without_an_icon_references_none() {
        let mut v = vehicle("bolt", json!({}));
        v.vehicle_type = Some("taxi".into());
        let set = build([&v]);
        assert!(set.icons.is_empty());
        // The marker may still name its type, but nothing points at an image
        // that was never sent.
        assert!(set.icons.get("taxi").is_none());
    }
}

/// Convert a [`fleet::Vehicle`] into the stored/wire row.
///
/// The boundary between the fleet library's domain model and this gateway's
/// wire contract. Two things change shape here: [`fleet::Kind`] flattens to the
/// `category` string the database column and the client already use, and the
/// millisecond timestamp becomes RFC 3339, because `api-types` is deliberately
/// dependency-free and has no date type of its own.
pub fn to_row(v: &fleet::Vehicle) -> BoltVehicle {
    BoltVehicle {
        id: v.id.clone(),
        name: v.name.clone(),
        category: v.kind.as_str().to_string(),
        sub_category: v.sub_category.clone(),
        lat: v.lat,
        lng: v.lng,
        heading: v.heading,
        vehicle_type: v.vehicle_type.clone(),
        icon_url: v.icon_url.clone(),
        raw: v.raw.clone(),
        fetched_at: chrono::DateTime::from_timestamp_millis(v.fetched_at)
            .unwrap_or_else(chrono::Utc::now)
            .to_rfc3339(),
    }
}

#[cfg(test)]
mod row_tests {
    use super::*;

    fn vehicle(kind: fleet::Kind) -> fleet::Vehicle {
        fleet::Vehicle {
            id: "1".into(),
            source_id: "marine".into(),
            kind,
            name: None,
            sub_category: None,
            lat: -6.83,
            lng: 39.30,
            heading: None,
            vehicle_type: None,
            icon_url: None,
            raw: serde_json::Value::Null,
            fetched_at: 1_756_000_000_000,
        }
    }

    /// `marker` switches on this string, so the two must agree — a mismatch
    /// silently downgrades every vessel to a plain dot.
    #[test]
    fn the_category_is_a_kind_that_marker_recognises() {
        for kind in [
            fleet::Kind::Taxi,
            fleet::Kind::Vessel,
            fleet::Kind::Aircraft,
            fleet::Kind::Train,
        ] {
            let row = to_row(&vehicle(kind));
            assert_eq!(row.category, kind.as_str());
        }
    }

    #[test]
    fn the_timestamp_becomes_rfc_3339() {
        let row = to_row(&vehicle(fleet::Kind::Vessel));
        assert!(
            row.fetched_at.starts_with("2025-"),
            "got {}",
            row.fetched_at
        );
        assert!(chrono::DateTime::parse_from_rfc3339(&row.fetched_at).is_ok());
    }
}
