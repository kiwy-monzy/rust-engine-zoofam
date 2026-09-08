//! Low-level HTTP calls to Bolt’s user API. Stateless — pass [`Session`] in/out.

use base64::Engine;
use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CACHE_CONTROL, CONNECTION, CONTENT_TYPE,
};

use crate::bolt::map_view::BoltMapView;
use crate::bolt::types::{Session, SessionStatus};
use super::types::Device;
use super::{BoltError, Result};

const BASE_URL: &str = "https://user.live.boltsvc.net";
const USER_AGENT: &str = "Bolt/13113543 CFNetwork/1568.300.101 Darwin/24.2.0";
const ACCEPT_LANGUAGE: &str = "en-GB,en;q=0.9";
const HOST: &str = "user.live.boltsvc.net";
const NODE_HOST: &str = "node.bolt.eu";

const VERSION: &str = "CI.169.1";
const LANGUAGE: &str = "en-GB";
const BRAND: &str = "bolt";
const DEVICE_TYPE: &str = "iphone";
const DEVICE_OS_VERSION: &str = "iOS13.2";
const DEVICE_NAME: &str = "iPhone16,1";
const GPS_ACCURACY_M: &str = "5.0";

/// Maps user-facing channel to Bolt JSON `type` and `method` fields.
pub fn channel_type_and_method(channel: &str) -> (&'static str, &'static str) {
    match channel.trim().to_ascii_lowercase().as_str() {
        "sms" => ("sms", "sms"),
        "whatsapp" => ("whatsapp", "whatsapp"),
        "phone" | "voice" => ("phone", "voice"),
        _ => ("phone", "voice"),
    }
}

/// Shared unauthenticated client (start verification).
pub fn create_client() -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    headers.insert(reqwest::header::HOST, HeaderValue::from_static(HOST));
    headers.insert(
        reqwest::header::USER_AGENT,
        HeaderValue::from_static(USER_AGENT),
    );
    headers.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        HeaderValue::from_static(ACCEPT_LANGUAGE),
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(Into::into)
}

fn create_auth_client(auth_username: &str, device_uuid: &str) -> Result<reqwest::Client> {
    create_auth_client_for_host(auth_username, device_uuid, HOST)
}

fn create_auth_client_for_host(
    auth_username: &str,
    device_uuid: &str,
    host: &'static str,
) -> Result<reqwest::Client> {
    let token =
        base64::engine::general_purpose::STANDARD.encode(format!("{auth_username}:{device_uuid}"));

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    headers.insert(reqwest::header::HOST, HeaderValue::from_static(host));
    headers.insert(
        reqwest::header::USER_AGENT,
        HeaderValue::from_static(USER_AGENT),
    );
    headers.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        HeaderValue::from_static(ACCEPT_LANGUAGE),
    );
    let auth_value = format!("Basic {token}");
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&auth_value)
            .map_err(|e| BoltError::ApiError(format!("Invalid auth header: {e}")))?,
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(Into::into)
}

fn session_stamp(session: &Session) -> String {
    format!(
        "{}u{}",
        user_id(session),
        chrono::Utc::now().timestamp_millis()
    )
}

fn user_id(session: &Session) -> &str {
    session
        .auth_username
        .strip_prefix("uid_")
        .unwrap_or(session.auth_username.as_str())
}

fn common_query(
    session: &Session,
    lat: f64,
    lng: f64,
    country_iso2: &str,
) -> Vec<(&'static str, String)> {
    vec![
        ("device_os_version", DEVICE_OS_VERSION.to_string()),
        ("deviceType", DEVICE_TYPE.to_string()),
        ("device_name", DEVICE_NAME.to_string()),
        ("deviceId", session.device_id.clone()),
        ("brand", BRAND.to_string()),
        ("version", VERSION.to_string()),
        ("language", LANGUAGE.to_string()),
        ("country", country_iso2.to_ascii_lowercase()),
        ("lat", lat.to_string()),
        ("lng", lng.to_string()),
        ("gps_lat", lat.to_string()),
        ("gps_lng", lng.to_string()),
        ("gps_accuracy_m", GPS_ACCURACY_M.to_string()),
        ("gps_age", "1.0".to_string()),
        ("user_id", user_id(session).to_string()),
        ("distinct_id", format!("client-{}", user_id(session))),
        ("session_id", session_stamp(session)),
        ("rh_session_id", session_stamp(session)),
    ]
}

fn api_url(host: &str, path: &str, query: &[(&str, String)]) -> Result<reqwest::Url> {
    let mut url = reqwest::Url::parse(&format!("https://{host}{path}"))
        .map_err(|e| BoltError::ApiError(format!("Invalid Bolt URL: {e}")))?;
    {
        let mut pairs = url.query_pairs_mut();
        for (k, v) in query {
            pairs.append_pair(k, v);
        }
    }
    Ok(url)
}

#[derive(serde::Serialize)]
struct StartVerificationBody {
    #[serde(rename = "type")]
    r#type: String,
    timezone: String,
    last_known_state: LastKnownState,
    password: String,
    phone_number: String,
    method: String,
}

#[derive(serde::Serialize)]
struct LastKnownState {
    location: Location,
    opened_product: OpenedProduct,
}

#[derive(serde::Serialize)]
struct Location {
    lng: f64,
    lat: f64,
}

#[derive(serde::Serialize)]
struct OpenedProduct {
    product: String,
}

/// Request OTP. `device_uuid` must match the id stored by the app (e.g. libqaul credentials).
/// `channel` is one of `sms`, `phone` (voice call), or `whatsapp`.
pub async fn start_verification(
    client: &reqwest::Client,
    phone: &str,
    device: &Device,
    channel: &str,
) -> Result<Session> {
    let Device { uuid: device_uuid, lat, lng, timezone, .. } = device;
    let (lat, lng) = (*lat, *lng);
    let country = device.country.to_ascii_lowercase();

    let url = format!(
        "{}/profile/verification/start/v2?version={}&language={}&brand={}&deviceType={}&session_id={}&deviceId={}&lng={}&device_os_version={}&gps_lng={}&device_name={}&country={}&gps_lat={}&distinct_id=$device:{}&gps_accuracy_m={}&lat={}&gps_age=1.0",
        BASE_URL,
        VERSION,
        LANGUAGE,
        BRAND,
        DEVICE_TYPE,
        device_uuid,
        device_uuid,
        lng,
        DEVICE_OS_VERSION,
        lng,
        DEVICE_NAME,
        country,
        lat,
        device_uuid,
        GPS_ACCURACY_M,
        lat
    );

    let (api_type, api_method) = channel_type_and_method(channel);
    let body = StartVerificationBody {
        r#type: api_type.to_string(),
        timezone: timezone.to_string(),
        last_known_state: LastKnownState {
            location: Location { lng, lat },
            opened_product: OpenedProduct {
                product: "taxi".to_string(),
            },
        },
        password: device_uuid.to_string(),
        phone_number: phone.to_string(),
        method: api_method.to_string(),
    };

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let response = client
        .post(&url)
        .headers(headers)
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body_text = response.text().await.unwrap_or_default();
        return Err(BoltError::ApiError(format!(
            "Login failed with status: {} - {}",
            status, body_text
        )));
    }

    Ok(Session {
        session_id: device_uuid.to_string(),
        device_id: device_uuid.to_string(),
        auth_username: String::new(),
        phone: phone.to_string(),
        created_at: chrono::Utc::now(),
        status: SessionStatus::Pending,
        verification_channel: api_type.to_string(),
    })
}

/// Confirm OTP; `pending` is typically the return value of [`start_verification`].
pub async fn confirm_verification(
    client: &reqwest::Client,
    pending: &Session,
    otp: &str,
    device: &Device,
) -> Result<Session> {
    let Device { lat, lng, timezone, .. } = device;
    let (lat, lng) = (*lat, *lng);
    let country_iso2 = device.country.as_str();
    let country = country_iso2.to_ascii_lowercase();

    let url = format!(
        "{}/profile/verification/confirm/v3?lng={}&rh_session_id={}&country={}&distinct_id=device:{}&deviceId={}&device_os_version={}&deviceType={}&brand={}&language={}&session_id={}&version={}&gps_lat={}&device_name={}&gps_accuracy_m={}&gps_age=1.0&lat={}&gps_lng={}",
        BASE_URL,
        lng,
        pending.device_id,
        country,
        pending.device_id,
        pending.device_id,
        DEVICE_OS_VERSION,
        DEVICE_TYPE,
        BRAND,
        LANGUAGE,
        pending.session_id,
        VERSION,
        lat,
        DEVICE_NAME,
        GPS_ACCURACY_M,
        lat,
        lng
    );

    #[derive(serde::Serialize)]
    struct ConfirmBody {
        #[serde(rename = "type")]
        r#type: String,
        password: String,
        timezone: String,
        phone_number: String,
        code: String,
        last_known_state: LastKnownState,
    }

    let (api_type, _) = channel_type_and_method(pending.verification_channel.as_str());
    let body = ConfirmBody {
        r#type: api_type.to_string(),
        password: pending.device_id.clone(),
        timezone: timezone.to_string(),
        phone_number: pending.phone.clone(),
        code: otp.to_string(),
        last_known_state: LastKnownState {
            location: Location { lng, lat },
            opened_product: OpenedProduct {
                product: "taxi".to_string(),
            },
        },
    };

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let response = client
        .post(&url)
        .headers(headers)
        .json(&body)
        .send()
        .await?;

    let text = response.text().await?;
    let parsed: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| BoltError::ApiError(format!("Failed to parse confirm response: {e}")))?;

    let code = parsed.get("code").and_then(|v| v.as_i64()).unwrap_or(-1);
    let message = parsed
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("(no message)");
    if code != 0 || !message.eq_ignore_ascii_case("OK") {
        return Err(BoltError::ApiError(format!(
            "Verification failed: {message} (code {code})"
        )));
    }

    // Bolt has moved this field between releases — `data.auth.auth_username`
    // and `data.auth_username` have both been correct — so search the payload
    // for it instead of pinning one path and breaking on the next reshuffle.
    let data = parsed.get("data").unwrap_or(&parsed);
    let auth_username = find_string(data, "auth_username").ok_or_else(|| {
        // Naming only the missing field made this undiagnosable from a log: you
        // could not tell a moved field from a rejected code. Describe what did
        // arrive instead — key names only, never values, because this payload
        // carries session tokens.
        BoltError::ApiError(format!(
            "No auth_username in response. Confirm reported success (code 0, \
             {message}) but no such field appears anywhere under `data`. \
             Keys received: {}",
            describe_shape(data, 3)
        ))
    })?;

    Ok(Session {
        session_id: pending.session_id.clone(),
        device_id: pending.device_id.clone(),
        auth_username,
        phone: pending.phone.clone(),
        created_at: chrono::Utc::now(),
        status: SessionStatus::Active,
        verification_channel: pending.verification_channel.clone(),
    })
}

fn expand_viewport_if_needed(v: BoltMapView) -> BoltMapView {
    let v = v.normalized();
    let min_span = 0.005_f64;
    let mut sw_lat = v.sw_lat;
    let mut ne_lat = v.ne_lat;
    let mut sw_lng = v.sw_lng;
    let mut ne_lng = v.ne_lng;

    if (ne_lat - sw_lat).abs() < min_span {
        let mid = (sw_lat + ne_lat) * 0.5;
        sw_lat = mid - min_span;
        ne_lat = mid + min_span;
    }
    if (ne_lng - sw_lng).abs() < min_span {
        let mid = (sw_lng + ne_lng) * 0.5;
        sw_lng = mid - min_span;
        ne_lng = mid + min_span;
    }

    BoltMapView {
        center_lat: v.center_lat,
        center_lng: v.center_lng,
        sw_lat,
        sw_lng,
        ne_lat,
        ne_lng,
    }
}

/// Poll taxi vehicles for an **active** session and map viewport.
pub async fn fetch_vehicles(
    session: &Session,
    view: &BoltMapView,
) -> Result<Vec<crate::bolt::vehicle::Vehicle>> {
    let auth_client = create_auth_client(&session.auth_username, &session.device_id)?;
    let v = expand_viewport_if_needed(*view);
    let lat = v.center_lat;
    let lng = v.center_lng;

    let url = format!(
        "{}/mobility/search/poll?version={}&rh_session_id={}&language={}&gps_lat={}&device_name={}&distinct_id=client-001&device_os_version={}&gps_accuracy_m={}&deviceId={}&deviceType={}&gps_lng={}&brand={}&lat={}&user_id={}&session_id={}&gps_age=0.1&lng={}",
        BASE_URL,
        VERSION,
        session.session_id,
        LANGUAGE,
        lat,
        DEVICE_NAME,
        DEVICE_OS_VERSION,
        GPS_ACCURACY_M,
        session.device_id,
        DEVICE_TYPE,
        lng,
        BRAND,
        lat,
        session.auth_username,
        session.session_id,
        lng
    );

    let body = serde_json::json!({
        "order_handle": {},
        "stage": "category_selection",
        "pickup_stop": { "lat": lat, "lng": lng },
        "viewport": {
            "south_west": { "lat": v.sw_lat, "lng": v.sw_lng },
            "north_east": { "lat": v.ne_lat, "lng": v.ne_lng }
        },
        "destination_stops": [{ "lat": lat + 0.005, "lng": lng + 0.005 }],
        "payment_method": { "type": "default", "id": "cash" }
    });

    let response = auth_client.post(&url).json(&body).send().await?;
    let raw: serde_json::Value = response.json().await?;
    Ok(parse_poll(raw)?.0)
}

/// Like [`fetch_vehicles`], but also returns the `icon_id → image URL` map the
/// poll response carries.
///
/// `icon_id` is a **zone-specific numeric id**: the same "boda" is a different
/// number in Dar es Salaam than in Nairobi, and the crate's built-in table only
/// covers the handful seen so far. Anything outside it used to render as
/// "Unknown" with no artwork at all. The poll response advertises the real icon
/// URLs alongside the vehicles, so the honest source of icons is the response
/// itself, not a hardcoded list.
pub async fn fetch_vehicles_and_icons(
    session: &Session,
    view: &BoltMapView,
) -> Result<(Vec<crate::bolt::vehicle::Vehicle>, std::collections::BTreeMap<String, String>)> {
    let auth_client = create_auth_client(&session.auth_username, &session.device_id)?;
    let v = expand_viewport_if_needed(*view);
    let url = poll_url(session, v.center_lat, v.center_lng);
    let body = poll_body(&v);

    let response = auth_client.post(&url).json(&body).send().await?;
    let raw: serde_json::Value = response.json().await?;
    parse_poll(raw)
}

/// Splits one poll payload into the vehicles and the icon URLs it advertises.
fn parse_poll(
    raw: serde_json::Value,
) -> Result<(Vec<crate::bolt::vehicle::Vehicle>, std::collections::BTreeMap<String, String>)> {
    let icons = collect_icon_urls(&raw);

    let vehicle_response: crate::bolt::vehicle::VehicleResponse = serde_json::from_value(raw)
        .map_err(|e| BoltError::ApiError(format!("Failed to parse vehicle response: {e}")))?;

    let vehicles = vehicle_response
        .data
        .and_then(|d| d.vehicles)
        .and_then(|v| v.taxi)
        .map(|taxi_map| {
            taxi_map
                .into_iter()
                .flat_map(|(category, vs)| {
                    // The map key is the category group ("boda", "premium", …);
                    // carry it onto each vehicle as a type fallback.
                    vs.into_iter().map(move |v| {
                        let mut veh = crate::bolt::vehicle::Vehicle::from(v);
                        veh.category = category.clone();
                        veh
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok((vehicles, icons))
}

/// Walks the payload for objects that pair an icon id with an image URL.
///
/// Bolt has not kept these in one place between releases — they have appeared
/// under the category list, under a per-type block and inline on the vehicle —
/// so this looks for the *pairing* rather than a fixed path: any object that
/// carries something id-shaped and something url-shaped contributes an entry.
fn collect_icon_urls(v: &serde_json::Value) -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    walk_icons(v, &mut out);
    out
}

fn walk_icons(v: &serde_json::Value, out: &mut std::collections::BTreeMap<String, String>) {
    match v {
        serde_json::Value::Object(map) => {
            let id = ["icon_id", "iconId", "id", "type_id", "category_id"]
                .iter()
                .find_map(|k| match map.get(*k) {
                    Some(serde_json::Value::String(s)) if !s.is_empty() => Some(s.clone()),
                    Some(serde_json::Value::Number(n)) => Some(n.to_string()),
                    _ => None,
                });
            if let Some(id) = id {
                if let Some(url) = icon_url_of(map) {
                    out.entry(id).or_insert(url);
                }
            }
            for child in map.values() {
                walk_icons(child, out);
            }
        }
        serde_json::Value::Array(items) => {
            for child in items {
                walk_icons(child, out);
            }
        }
        _ => {}
    }
}

/// An `http(s)` image URL on this object — either a string field whose name
/// hints at an image, or a nested `{ url | image_url | src | png | svg }`.
fn icon_url_of(map: &serde_json::Map<String, serde_json::Value>) -> Option<String> {
    let hints = |k: &str| {
        let k = k.to_ascii_lowercase();
        k.contains("icon") || k.contains("image") || k.contains("logo") || k.contains("pin")
    };
    for (k, v) in map {
        if hints(k) {
            if let Some(s) = v.as_str() {
                if s.starts_with("http") {
                    return Some(s.to_string());
                }
            }
        }
    }
    for (k, v) in map {
        if hints(k) {
            if let Some(obj) = v.as_object() {
                for key in ["url", "image_url", "src", "png", "svg"] {
                    if let Some(s) = obj.get(key).and_then(|x| x.as_str()) {
                        if s.starts_with("http") {
                            return Some(s.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

fn poll_url(session: &Session, lat: f64, lng: f64) -> String {
    format!(
        "{}/mobility/search/poll?version={}&rh_session_id={}&language={}&gps_lat={}&device_name={}&distinct_id=client-001&device_os_version={}&gps_accuracy_m={}&deviceId={}&deviceType={}&gps_lng={}&brand={}&lat={}&user_id={}&session_id={}&gps_age=0.1&lng={}",
        BASE_URL,
        VERSION,
        session.session_id,
        LANGUAGE,
        lat,
        DEVICE_NAME,
        DEVICE_OS_VERSION,
        GPS_ACCURACY_M,
        session.device_id,
        DEVICE_TYPE,
        lng,
        BRAND,
        lat,
        session.auth_username,
        session.session_id,
        lng
    )
}

fn poll_body(v: &BoltMapView) -> serde_json::Value {
    let (lat, lng) = (v.center_lat, v.center_lng);
    serde_json::json!({
        "order_handle": {},
        "stage": "category_selection",
        "pickup_stop": { "lat": lat, "lng": lng },
        "viewport": {
            "south_west": { "lat": v.sw_lat, "lng": v.sw_lng },
            "north_east": { "lat": v.ne_lat, "lng": v.ne_lng }
        },
        "destination_stops": [{ "lat": lat + 0.005, "lng": lng + 0.005 }],
        "payment_method": { "type": "default", "id": "cash" }
    })
}

pub async fn search_locations(
    session: &Session,
    search_string: &str,
    lat: f64,
    lng: f64,
    country_iso2: &str,
) -> Result<serde_json::Value> {
    let auth_client = create_auth_client(&session.auth_username, &session.device_id)?;
    let mut query = common_query(session, lat, lng, country_iso2);
    query.push(("search_string", search_string.to_string()));
    query.push(("external_search", "false".to_string()));
    let url = api_url(HOST, "/rides/search/getDropoffSuggestions", &query)?;
    let response = auth_client.get(url).send().await?;
    Ok(response.json().await?)
}

pub async fn address_details(
    session: &Session,
    place_id: &str,
    source: &str,
    lat: f64,
    lng: f64,
    country_iso2: &str,
) -> Result<serde_json::Value> {
    let auth_client =
        create_auth_client_for_host(&session.auth_username, &session.device_id, NODE_HOST)?;
    let mut query = common_query(session, lat, lng, country_iso2);
    query.push(("place_id", place_id.to_string()));
    query.push(("source", source.to_string()));
    let url = api_url(
        NODE_HOST,
        "/user/user/findExternalSourceAddressDetails",
        &query,
    )?;
    let response = auth_client.get(url).send().await?;
    Ok(response.json().await?)
}

pub async fn ride_options(
    session: &Session,
    pickup: serde_json::Value,
    destination: serde_json::Value,
    country_iso2: &str,
    timezone: &str,
) -> Result<serde_json::Value> {
    let lat = pickup["lat"].as_f64().unwrap_or(0.0);
    let lng = pickup["lng"].as_f64().unwrap_or(0.0);
    let auth_client = create_auth_client(&session.auth_username, &session.device_id)?;
    let query = common_query(session, lat, lng, country_iso2);
    let url = api_url(HOST, "/rides/search/getRideOptions", &query)?;
    let body = serde_json::json!({
        "pickup_stop": {
            "is_confirmed": false,
            "lat": lat,
            "lng": lng,
            "full_address": pickup.get("fullAddress").or_else(|| pickup.get("full_address")).cloned().unwrap_or(serde_json::Value::Null),
            "place_id": pickup.get("placeId").or_else(|| pickup.get("place_id")).cloned().unwrap_or(serde_json::Value::Null),
            "address": pickup.get("address").cloned().unwrap_or(serde_json::Value::Null)
        },
        "payment_method": { "id": "cash", "type": "default" },
        "campaign_code": {},
        "destination_stops": [{
            "lat": destination["lat"].as_f64().unwrap_or(lat),
            "lng": destination["lng"].as_f64().unwrap_or(lng),
            "full_address": destination.get("fullAddress").or_else(|| destination.get("full_address")).cloned().unwrap_or(serde_json::Value::Null),
            "place_id": destination.get("placeId").or_else(|| destination.get("place_id")).cloned().unwrap_or(serde_json::Value::Null),
            "address": destination.get("address").cloned().unwrap_or(serde_json::Value::Null)
        }],
        "timezone": timezone
    });
    let response = auth_client.post(url).json(&body).send().await?;
    Ok(response.json().await?)
}

pub async fn pickup_data(
    session: &Session,
    pickup: serde_json::Value,
    destination: serde_json::Value,
    category_id: Option<String>,
    country_iso2: &str,
    timezone: &str,
) -> Result<serde_json::Value> {
    let lat = pickup["lat"].as_f64().unwrap_or(0.0);
    let lng = pickup["lng"].as_f64().unwrap_or(0.0);
    let auth_client =
        create_auth_client_for_host(&session.auth_username, &session.device_id, NODE_HOST)?;
    let query = common_query(session, lat, lng, country_iso2);
    let url = api_url(NODE_HOST, "/gateway/v1/getPickupData", &query)?;
    let body = serde_json::json!({
        "pickup_stop": {
            "lat": lat,
            "lng": lng,
            "full_address": pickup.get("fullAddress").or_else(|| pickup.get("full_address")).cloned().unwrap_or(serde_json::Value::Null),
            "place_id": pickup.get("placeId").or_else(|| pickup.get("place_id")).cloned().unwrap_or(serde_json::Value::Null),
            "address": pickup.get("address").cloned().unwrap_or(serde_json::Value::Null),
            "is_precise": true
        },
        "destination_stops": [{
            "lat": destination["lat"].as_f64().unwrap_or(lat),
            "lng": destination["lng"].as_f64().unwrap_or(lng),
            "full_address": destination.get("fullAddress").or_else(|| destination.get("full_address")).cloned().unwrap_or(serde_json::Value::Null),
            "place_id": destination.get("placeId").or_else(|| destination.get("place_id")).cloned().unwrap_or(serde_json::Value::Null),
            "address": destination.get("address").cloned().unwrap_or(serde_json::Value::Null)
        }],
        "reason": "confirm_pickup_pin_init",
        "timezone": timezone,
        "is_final": true,
        "is_scheduled_ride": false,
        "payment_method": { "id": "cash", "type": "default" },
        "category_id": category_id
    });
    let response = auth_client.post(url).json(&body).send().await?;
    Ok(response.json().await?)
}

/// Finds the first string value stored under `key`, at any depth.
///
/// Bolt's payload shapes shift between releases; addressing a field by name
/// rather than by path means a field that moves one level deeper still works,
/// and a field that genuinely disappears is reported as missing rather than
/// silently read as null.
fn find_string(value: &serde_json::Value, key: &str) -> Option<String> {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(s) = map.get(key).and_then(|v| v.as_str()) {
                if !s.is_empty() {
                    return Some(s.to_string());
                }
            }
            map.values().find_map(|v| find_string(v, key))
        }
        serde_json::Value::Array(items) => items.iter().find_map(|v| find_string(v, key)),
        _ => None,
    }
}

/// Renders the *shape* of a payload — key names and nesting, no values.
///
/// Deliberately values-free: this is written into error messages and logs, and
/// a confirm payload carries session tokens.
fn describe_shape(value: &serde_json::Value, depth: usize) -> String {
    match value {
        serde_json::Value::Object(map) if depth > 0 => {
            let inner: Vec<String> = map
                .iter()
                .map(|(k, v)| match v {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        format!("{k}: {}", describe_shape(v, depth - 1))
                    }
                    _ => k.clone(),
                })
                .collect();
            format!("{{{}}}", inner.join(", "))
        }
        serde_json::Value::Object(_) => "{…}".into(),
        serde_json::Value::Array(items) if depth > 0 => match items.first() {
            Some(first) => format!("[{}; {}]", describe_shape(first, depth - 1), items.len()),
            None => "[]".into(),
        },
        serde_json::Value::Array(items) => format!("[…; {}]", items.len()),
        serde_json::Value::Null => "null".into(),
        _ => "_".into(),
    }
}

#[cfg(test)]
mod confirm_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_field_is_found_wherever_bolt_puts_it() {
        let nested = json!({ "auth": { "auth_username": "uid_123" } });
        assert_eq!(find_string(&nested, "auth_username").as_deref(), Some("uid_123"));
        let flat = json!({ "auth_username": "uid_456" });
        assert_eq!(find_string(&flat, "auth_username").as_deref(), Some("uid_456"));
        let deep = json!({ "user": { "profile": { "auth_username": "uid_789" } } });
        assert_eq!(find_string(&deep, "auth_username").as_deref(), Some("uid_789"));
    }

    #[test]
    fn an_absent_or_blank_field_is_not_a_value() {
        assert_eq!(find_string(&json!({ "other": "x" }), "auth_username"), None);
        assert_eq!(find_string(&json!({ "auth_username": "" }), "auth_username"), None);
    }

    /// The shape description goes into logs, so it must never leak a token.
    #[test]
    fn the_shape_description_carries_no_values() {
        let payload = json!({ "auth": { "refresh_token": "SECRET-TOKEN", "id": 7 } });
        let shape = describe_shape(&payload, 3);
        assert!(!shape.contains("SECRET-TOKEN"), "leaked a value: {shape}");
        assert!(shape.contains("refresh_token"), "lost the key names: {shape}");
    }
}

#[cfg(test)]
mod icon_tests {
    use super::*;
    use serde_json::json;

    /// The pairing can sit anywhere; what matters is that an id and a URL are
    /// on the same object.
    #[test]
    fn icon_urls_are_found_wherever_they_are_paired() {
        let payload = json!({
            "data": {
                "categories": [
                    { "id": "8749", "icon": "https://cdn.bolt.eu/bolt.png" },
                    { "category_id": 278, "image_url": "https://cdn.bolt.eu/boda.png" }
                ],
                "types": { "inner": { "icon_id": "9172", "logo": { "url": "https://cdn.bolt.eu/ev.png" } } }
            }
        });
        let icons = collect_icon_urls(&payload);
        assert_eq!(icons.get("8749").map(String::as_str), Some("https://cdn.bolt.eu/bolt.png"));
        // A numeric id must key the same as a string one.
        assert_eq!(icons.get("278").map(String::as_str), Some("https://cdn.bolt.eu/boda.png"));
        // Nested under a url-bearing object, arbitrarily deep.
        assert_eq!(icons.get("9172").map(String::as_str), Some("https://cdn.bolt.eu/ev.png"));
    }

    /// A zone id the built-in table has never seen must still get artwork —
    /// that is the whole point of reading the response instead of a fixed list.
    #[test]
    fn an_unknown_zone_id_still_yields_an_icon() {
        assert!(
            crate::bolt::vehicle::get_icon_filename_opt("55501").is_none(),
            "fixture must be an id the built-in table does not know"
        );
        let icons = collect_icon_urls(&json!({
            "categories": [{ "icon_id": "55501", "icon": "https://cdn.bolt.eu/nairobi-boda.png" }]
        }));
        assert_eq!(
            icons.get("55501").map(String::as_str),
            Some("https://cdn.bolt.eu/nairobi-boda.png")
        );
    }

    #[test]
    fn non_image_urls_and_relative_paths_are_ignored() {
        let icons = collect_icon_urls(&json!({
            "categories": [
                { "id": "1", "deeplink": "https://bolt.eu/ride" },
                { "id": "2", "icon": "/local/path.png" },
                { "id": "3", "icon": "" }
            ]
        }));
        assert!(icons.is_empty(), "picked up a non-icon or relative url: {icons:?}");
    }

    #[test]
    fn the_first_url_for_an_id_wins_and_parsing_survives_junk() {
        let icons = collect_icon_urls(&json!({
            "a": [{ "id": "7", "icon": "https://x/one.png" }],
            "b": [{ "id": "7", "icon": "https://x/two.png" }]
        }));
        assert_eq!(icons.get("7").map(String::as_str), Some("https://x/one.png"));
        assert!(collect_icon_urls(&json!(null)).is_empty());
        assert!(collect_icon_urls(&json!([1, "x", true])).is_empty());
    }
}
