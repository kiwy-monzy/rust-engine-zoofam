//! FlightRadar24 live flights — **keyless**, via the public `feed.js` JSON
//! (not the protobuf LiveFeed, which would drag prost/protoc into the build).
//!
//! `feed.js` returns a flat object: two metadata keys (`full_count`,
//! `version`) plus one entry per flight whose value is a positional array
//! `[icao24, lat, lon, track, altitude, speed, squawk, radar, type, reg, ts,
//!  origin, dest, flight, …, callsign, …]`. We read the few fields the map needs.

use crate::error::{Error, Result};

use crate::model::{Kind, Vehicle};

// `data-live` 302-redirects datacenter IPs to a dead legacy feed; `data-cloud`
// answers with real data when the request carries the web client's headers
// (origin/referer + a browser UA). The protobuf client in `super::client`
// avoids the block the same way — the transport differs (it speaks gRPC-web)
// but the host and the headers are the trick in both.
const FEED_BASE: &str = "https://data-cloud.flightradar24.com/zones/fcgi/feed.js";
const UA: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

/// Fetch the flights within a box around a point (`span` degrees each side).
pub async fn fetch(lat: f64, lng: f64, span: f64) -> Result<Vec<Vehicle>> {
    let (n, s) = (lat + span, lat - span);
    let (w, e) = (lng - span, lng + span);
    let url = format!(
        "{FEED_BASE}?bounds={n:.3},{s:.3},{w:.3},{e:.3}\
         &faa=1&satellite=1&mlat=1&flarm=1&adsb=1&gnd=1&air=1&vehicles=1\
         &estimated=1&maxage=14400&gliders=1&stats=0"
    );

    let client = reqwest::Client::builder()
        .gzip(true)
        .build()
        ?;
    let raw: serde_json::Value = client
        .get(&url)
        .header("user-agent", UA)
        .header("accept", "application/json")
        .header("accept-language", "en-US,en;q=0.9")
        .header("origin", "https://www.flightradar24.com")
        .header("referer", "https://www.flightradar24.com/")
        .send()
        .await
        ?
        .json()
        .await
        .map_err(|e| Error::Malformed { source_id: "flights", detail: e.to_string() })?;

    let now = crate::now_ms();
    Ok(parse(&raw, now))
}

fn parse(raw: &serde_json::Value, now: i64) -> Vec<Vehicle> {
    let obj = match raw.as_object() {
        Some(o) => o,
        None => return Vec::new(),
    };
    obj.iter()
        .filter(|(k, _)| !matches!(k.as_str(), "full_count" | "version" | "stats"))
        .filter_map(|(id, v)| {
            let a = v.as_array()?;
            let lat = a.get(1).and_then(|x| x.as_f64())?;
            let lng = a.get(2).and_then(|x| x.as_f64())?;
            let heading = a.get(3).and_then(|x| x.as_f64());
            let ac_type = a.get(8).and_then(|x| x.as_str()).filter(|s| !s.is_empty());
            let flight = a
                .get(16)
                .and_then(|x| x.as_str())
                .filter(|s| !s.is_empty())
                .or_else(|| a.get(13).and_then(|x| x.as_str()))
                .filter(|s| !s.is_empty());
            Some(Vehicle {
                id: id.clone(),
                name: flight.map(|s| s.to_string()),
                source_id: "flights".to_string(),
                kind: Kind::Aircraft,
                sub_category: ac_type.map(|s| s.to_string()),
                lat,
                lng,
                heading,
                vehicle_type: ac_type.map(|s| s.to_string()),
                icon_url: None,
                raw: v.clone(),
                fetched_at: now,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_feed_and_skips_metadata() {
        let raw = serde_json::json!({
            "full_count": 12000,
            "version": 5,
            "abc123": ["4CA1FA", -6.85, 39.28, 270.0, 35000, 450, "1234", "T-EADS",
                        "A320", "5H-XYZ", 1699999999, "JNB", "DAR", "SA123", 0, -64, "SAA123"],
            "def456": ["x", -6.7, 39.1]
        });
        let v = parse(&raw, 1_756_000_000_000);
        assert_eq!(v.len(), 2);
        let first = v.iter().find(|f| f.id == "abc123").unwrap();
        assert_eq!(first.source_id, "flights");
        assert_eq!(first.kind, crate::model::Kind::Aircraft);
        assert_eq!(first.vehicle_type.as_deref(), Some("A320"));
        assert_eq!(first.name.as_deref(), Some("SAA123"));
        assert!((first.lat + 6.85).abs() < 1e-9);
    }
}

/// One flight's recorded track.
///
/// FlightRadar24 serves history as **protobuf over HTTP**, not JSON — there is
/// no URL to borrow the way the vessel track was borrowed from MarineTraffic,
/// so this goes through the generated protobuf client in `super::client`. That
/// crate ships its own `protoc` via `protoc-bin-vendored`, so the build needs
/// no system protobuf compiler.
///
/// Live trail first, falling back to the historic one: a flight still in the
/// air has no historic record yet, and a landed one has no live trail. Asking
/// for the wrong one returns an error rather than an empty track, so trying
/// both is what makes a single endpoint work for either.
pub async fn history(flight_id: u32) -> Result<serde_json::Value> {
    let client = super::client::FlightRadar24Client::new();

    if let Ok(live) = client.get_live_trail(flight_id).await {
        if let Ok(v) = serde_json::to_value(&live) {
            if has_points(&v) {
                return Ok(json_with_source(v, "live"));
            }
        }
    }

    let historic = client
        .get_historic_trail(flight_id)
        .await
        .map_err(|e| Error::Malformed { source_id: "flights", detail: e.to_string() })?;
    let v = serde_json::to_value(&historic)
        .map_err(|e| Error::Malformed { source_id: "flights", detail: e.to_string() })?;
    Ok(json_with_source(v, "historic"))
}

/// Whether a trail actually carries positions.
///
/// An empty-but-successful response is the normal answer for a flight the feed
/// has nothing on, and it must not be mistaken for a usable track — that is
/// what decides the live/historic fallback.
fn has_points(v: &serde_json::Value) -> bool {
    v.as_object()
        .and_then(|o| o.values().find_map(|x| x.as_array()))
        .is_some_and(|a| !a.is_empty())
}

fn json_with_source(mut v: serde_json::Value, source: &str) -> serde_json::Value {
    if let Some(o) = v.as_object_mut() {
        o.insert("source".into(), serde_json::Value::String(source.into()));
    }
    v
}

#[cfg(test)]
mod history_tests {
    use super::*;
    use serde_json::json;

    /// The live/historic fallback hangs on this, so an empty trail must not
    /// read as a usable one.
    #[test]
    fn a_trail_with_no_points_is_not_usable() {
        assert!(!has_points(&json!({ "trail": [] })));
        assert!(!has_points(&json!({})));
        assert!(!has_points(&json!({ "flight_id": 1 })));
        assert!(has_points(&json!({ "trail": [{ "lat": 1.0 }] })));
    }

    #[test]
    fn the_source_is_recorded_on_the_answer() {
        let v = json_with_source(json!({ "trail": [] }), "historic");
        assert_eq!(v["source"], "historic");
        // A non-object answer is passed through rather than mangled.
        assert_eq!(json_with_source(json!([1, 2]), "live"), json!([1, 2]));
    }
}
