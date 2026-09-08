//! MarineTraffic vessel fetch — implemented directly against the mobile tile
//! JSON API rather than pulling the `marinetraffic` crate, which drags in
//! prost/protoc and native-tls and would break the gateway's rustls-only musl
//! build. The Cloudflare cookie comes from the vault (`service = "marinetraffic"`).
//!
//! MarineTraffic's raw JSON is undocumented and shifts; the parser reads it
//! leniently (several key spellings) and keeps the raw record on each row, so a
//! field that moves degrades to a missing column rather than a failed fetch.

use std::f64::consts::PI;
use std::sync::{Arc, Mutex, OnceLock};

use reqwest::cookie::{CookieStore, Jar};
use reqwest::Url;

use crate::error::{Error, Result};

use crate::model::{Kind, Vehicle};

/// Shared jar so a warmed `__cf_bm` is reused across tile/detail calls instead
/// of hitting the homepage on every poll. Replaced when Cloudflare blocks us.
static COOKIE_JAR: OnceLock<Mutex<Arc<Jar>>> = OnceLock::new();

fn jar_slot() -> &'static Mutex<Arc<Jar>> {
    COOKIE_JAR.get_or_init(|| Mutex::new(Arc::new(Jar::default())))
}

fn cookie_jar() -> Arc<Jar> {
    jar_slot()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
}

fn reset_cookie_jar() {
    *jar_slot().lock().unwrap_or_else(|e| e.into_inner()) = Arc::new(Jar::default());
}

fn marine_url() -> Url {
    Url::parse("https://www.marinetraffic.com/").expect("static URL")
}

fn jar_cookie_header() -> Option<String> {
    CookieStore::cookies(&*cookie_jar(), &marine_url())
        .and_then(|v| v.to_str().ok().map(|s| s.to_string()))
}

const TILE_BASE: &str = "https://www.marinetraffic.com/getData/get_data_json_4_mob";
const USER_AGENT: &str = "MarineTraffic/5.1.10 (iOS)";
/// A fixed constant the MarineTraffic mobile client sends on every tile
/// request; the endpoint rejects calls without it. Not a credential and not
/// tied to any account — unlike a session cookie, which is why that one comes
/// from the caller and this one does not.
const VESSEL_IMAGE: &str = "339f89c84a559f573636a47ff8daed0d3308";

/// Slippy-tile x/y for a lon/lat at a zoom.
fn tile_xy(lng: f64, lat: f64, z: u32) -> (u32, u32) {
    let n = 2f64.powi(z as i32);
    let x = (((lng + 180.0) / 360.0) * n).floor();
    let latr = lat.to_radians();
    let y = ((1.0 - (latr.tan() + 1.0 / latr.cos()).ln() / PI) / 2.0 * n).floor();
    let max = (n as i64 - 1).max(0);
    (
        (x as i64).clamp(0, max) as u32,
        (y as i64).clamp(0, max) as u32,
    )
}

/// Fetch the vessels in the tile covering a point. `z` is the MarineTraffic
/// tile zoom (7 is a good city-scale default); x/y are computed at `z-1` to
/// match its tiling.
///
/// **Auto-cookie**: with `cookie = None`, a cookie jar is warmed up against the
/// MarineTraffic homepage first, so Cloudflare sets `__cf_bm` and the tile call
/// carries it — no manual cookie needed. A stored vault cookie (which may also
/// hold `cf_clearance` for the sites that demand the full challenge) overrides
/// the jar when present.
pub async fn fetch(
    cookie: Option<&str>,
    lat: f64,
    lng: f64,
    z: u32,
) -> Result<(Vec<Vehicle>, serde_json::Value)> {
    match fetch_tiles(cookie, lat, lng, z, false).await {
        Err(Error::Blocked { .. }) if cookie.is_none() => {
            // Cached `__cf_bm` went stale — drop the jar and warm a new one.
            reset_cookie_jar();
            fetch_tiles(None, lat, lng, z, true).await
        }
        other => other,
    }
}

async fn fetch_tiles(
    cookie: Option<&str>,
    lat: f64,
    lng: f64,
    z: u32,
    force_warm: bool,
) -> Result<(Vec<Vehicle>, serde_json::Value)> {
    let (x, y) = tile_xy(lng, lat, z.saturating_sub(1).max(1));
    let cb = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let url = format!("{TILE_BASE}/z:{z}/X:{x}/Y:{y}/station:0?cb={cb}");

    let jar = cookie_jar();
    let client = reqwest::Client::builder()
        .gzip(true)
        .cookie_provider(jar)
        .build()?;

    // Warm up: one homepage GET makes Cloudflare hand back a `__cf_bm` cookie,
    // which the jar then replays on the tile request. Skip when a vault cookie
    // is supplied or the jar already has one from a previous poll.
    let need_warm = cookie.is_none() && (force_warm || jar_cookie_header().is_none());
    if need_warm {
        let _ = client
            .get("https://www.marinetraffic.com/")
            .header("user-agent", USER_AGENT)
            .send()
            .await;
    }

    let mut req = client
        .get(&url)
        .header("accept", "application/json")
        .header("origin", "https://www.marinetraffic.com")
        .header("is_mobile_v2", "true")
        .header("referer", "https://www.marinetraffic.com/")
        .header("is-mobile", "true")
        .header("user-agent", USER_AGENT)
        .header("x-app-version", "5.1.10")
        .header("x-requested-with", "XMLHttpRequest")
        .header("vessel-image", VESSEL_IMAGE);
    if let Some(c) = cookie {
        req = req.header("cookie", c);
    }
    let resp = req.send().await?;

    let status = resp.status();

    if !status.is_success() {
        // The status is checked *before* parsing. Cloudflare's challenge is
        // sometimes a JSON body, which used to parse cleanly and then yield
        // zero rows — so a blocked fetch was indistinguishable from an empty
        // sea, and the operator was left staring at a map with no ships and no
        // error. Typed variants keep those apart for callers too.
        return Err(match status.as_u16() {
            429 => Error::RateLimited { source_id: "marine" },
            code => Error::Blocked { source_id: "marine", status: code },
        });
    }

    let raw: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| Error::Malformed { source_id: "marine", detail: e.to_string() })?;

    let now = crate::now_ms();
    let vessels = parse(&raw, now);
    Ok((vessels, raw))
}

/// Parse the vessel rows leniently. The mobile API nests them under
/// `data.rows`; older/desktop shapes put them directly in `data`.
fn parse(raw: &serde_json::Value, now: i64) -> Vec<Vehicle> {
    let data = raw
        .get("data")
        .and_then(|d| d.get("rows"))
        .and_then(|r| r.as_array())
        .or_else(|| raw.get("data").and_then(|d| d.as_array()))
        .cloned()
        .unwrap_or_default();

    data.iter()
        .filter_map(|item| {
            let lat = num(item, &["LAT", "lat", "Y", "y"])?;
            let lng = num(item, &["LON", "lon", "LNG", "lng", "X", "x"])?;
            let id = text(item, &["SHIP_ID", "ship_id", "MMSI", "mmsi", "id"])
                .unwrap_or_else(|| format!("mt-{lat:.5},{lng:.5}"));
            let heading = num(item, &["HEADING", "heading_deg", "COURSE", "course_deg", "cog"]);
            // TYPE_NAME ("Tanker", "Cargo") reads better than the numeric SHIPTYPE.
            let stype = text(item, &["TYPE_NAME", "SHIPTYPE", "shiptype", "TYPE", "type", "ship_type"]);
            Some(Vehicle {
                id,
                name: text(item, &["SHIPNAME", "name", "NAME"]),
                source_id: "marine".to_string(),
                kind: Kind::Vessel,
                sub_category: stype.clone(),
                lat,
                lng,
                heading,
                vehicle_type: stype,
                icon_url: None,
                raw: item.clone(),
                fetched_at: now,
            })
        })
        .collect()
}

fn num(v: &serde_json::Value, keys: &[&str]) -> Option<f64> {
    for k in keys {
        if let Some(x) = v.get(k) {
            if let Some(f) = x.as_f64() {
                return Some(f);
            }
            if let Some(s) = x.as_str() {
                if let Ok(f) = s.parse::<f64>() {
                    return Some(f);
                }
            }
        }
    }
    None
}

fn text(v: &serde_json::Value, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(x) = v.get(k) {
            if let Some(s) = x.as_str() {
                if !s.is_empty() {
                    return Some(s.to_string());
                }
            } else if x.is_number() {
                return Some(x.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_mobile_data_rows_shape() {
        // The real mobile API: rows of string-typed fields under data.rows.
        let raw = serde_json::json!({
            "type": 1,
            "data": { "areaShips": 1, "rows": [
                { "SHIP_ID": "abc", "SHIPNAME": "MV Test", "LAT": "-6.774", "LON": "39.354",
                  "COURSE": "358", "HEADING": null, "TYPE_NAME": "Tanker", "SHIPTYPE": "8" }
            ]}
        });
        let v = parse(&raw, 1_756_000_000_000);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].id, "abc");
        assert_eq!(v[0].name.as_deref(), Some("MV Test"));
        assert_eq!(v[0].vehicle_type.as_deref(), Some("Tanker"));
        assert!((v[0].lat + 6.774).abs() < 1e-3);
        assert_eq!(v[0].heading, Some(358.0)); // HEADING null → COURSE
    }

    #[test]
    fn parses_flat_data_array_fallback() {
        let raw = serde_json::json!({
            "data": [ { "ship_id": "9", "lat": "-6.9", "lng": "39.1" } ]
        });
        let v = parse(&raw, 1_756_000_000_000);
        assert_eq!(v.len(), 1);
        assert!((v[0].lat + 6.9).abs() < 1e-9);
    }

    #[test]
    fn tiles_are_in_range() {
        let (x, y) = tile_xy(39.3, -6.8, 6);
        assert!(x < 64 && y < 64);
    }
}

/// One vessel's detail page: photo, voyage, ports, dimensions.
///
/// Fetched on demand rather than per poll — this is a request per vessel, and
/// there are thousands on screen. The map asks for it when someone selects a
/// ship.
///
/// The URL shapes are taken from the `marinetraffic` crate; the crate itself is
/// not a dependency here because it pulls `prost-build` and `native-tls`, and
/// OpenSSL against a static musl build is a fight this does not need for two
/// GET requests.
pub async fn vessel_detail(ship_id: &str, cookie: Option<&str>) -> Result<serde_json::Value> {
    let url = format!(
        "https://www.marinetraffic.com/en/reports/getMobileInfoWindow?asset_type=vessels&id={}",
        urlencode(ship_id)
    );
    fetch_json(&url, cookie).await
}

/// A vessel's recent track, as the map's history overlay draws it.
///
/// `from` and `to` are `YYYY-MM-DD HH:MM`; MarineTraffic wants them as path
/// segments with the space percent-encoded, which is why they are not query
/// parameters.
pub async fn vessel_track(
    ship_id: &str,
    from: &str,
    to: &str,
    cookie: Option<&str>,
) -> Result<serde_json::Value> {
    let url = format!(
        "https://www.marinetraffic.com/map/gettrackjson/shipid:{}/stdate:{}/endate:{}/trackorigin:livetrack",
        urlencode(ship_id),
        from.replace(' ', "%20"),
        to.replace(' ', "%20"),
    );
    fetch_json(&url, cookie).await
}

/// Percent-encodes the few characters that could smuggle a path segment.
///
/// The id goes into the *path*, so a bare `/` or `..` would redirect the
/// request somewhere else entirely.
fn urlencode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            other => format!("%{:02X}", other as u32 as u8),
        })
        .collect()
}

async fn fetch_json(url: &str, cookie: Option<&str>) -> Result<serde_json::Value> {
    let client = reqwest::Client::builder()
        .cookie_provider(cookie_jar())
        .build()?;
    let mut req = client
        .get(url)
        .header("accept", "application/json")
        .header("referer", "https://www.marinetraffic.com/")
        .header("user-agent", USER_AGENT)
        .header("x-requested-with", "XMLHttpRequest");
    if let Some(c) = cookie {
        req = req.header("cookie", c);
    }
    let resp = req
        .send()
        .await
        ?;

    // Same reasoning as the tile fetch: check the status before parsing, or a
    // Cloudflare challenge that happens to be JSON reads as a valid answer.
    let status = resp.status();
    if !status.is_success() {
        // The status is checked *before* parsing. Cloudflare's challenge is
        // sometimes a JSON body, which used to parse cleanly and then yield
        // zero rows — so a blocked fetch was indistinguishable from an empty
        // sea, and the operator was left staring at a map with no ships and no
        // error. Typed variants keep those apart for callers too.
        return Err(match status.as_u16() {
            429 => Error::RateLimited { source_id: "marine" },
            code => Error::Blocked { source_id: "marine", status: code },
        });
    }
    resp.json()
        .await
        .map_err(|e| Error::Malformed { source_id: "marine", detail: e.to_string() })
}

#[cfg(test)]
mod detail_tests {
    use super::*;

    /// The id lands in the URL path, so anything that could end the segment has
    /// to be encoded or the request goes to a different endpoint.
    #[test]
    fn ids_cannot_smuggle_a_path_segment() {
        assert_eq!(urlencode("123456"), "123456");
        assert_eq!(urlencode("a-b_c.d~e"), "a-b_c.d~e");
        // `.` is unreserved, so `..` survives — harmless, because the `/` that
        // would make it a traversal is encoded and cannot end the segment.
        assert_eq!(urlencode("../admin"), "..%2Fadmin");
        assert!(!urlencode("../admin").contains('/'), "path separator survived");
        assert_eq!(urlencode("1/2"), "1%2F2");
        assert_eq!(urlencode("a b"), "a%20b");
        assert!(!urlencode("x?y=1").contains('?'), "query smuggling");
    }
}
