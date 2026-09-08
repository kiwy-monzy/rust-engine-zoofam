//! SGR (Tanzania Standard Gauge Railway) — the TRC TICIDIS public API.
//!
//! SGR is a listing/booking service, **not a map fleet**: it has stations and
//! schedules, no live vehicle positions. So it is not binned onto the H3 grid;
//! it rides along in the unified `/api/fleet/all` payload as its own dataset,
//! polled like the rest. Only the keyless `Public/*` reads are used here.

use crate::error::{Error, Result};

const STATIONS_URL: &str =
    "https://sgrticket-api.trc.co.tz/TICIDIS/api/v1/Public/GetTripAssignedStations";

/// The boarding/landing stations, as the raw TICIDIS response.
pub async fn stations() -> Result<serde_json::Value> {
    let client = reqwest::Client::builder().build()?;
    let v = client
        .get(STATIONS_URL)
        .header("accept", "application/json")
        .send()
        .await?
        .json()
        .await
        .map_err(|e| Error::Malformed { source_id: "sgr", detail: e.to_string() })?;
    Ok(v)
}
