//! The four sources, behind one trait.
//!
//! Each is a thin adapter over the module that already knows how to talk to it.
//! The point is not the code they hold — it is that a caller walking
//! [`all()`] never learns any of their names.

use crate::error::{Error, Result};
use crate::model::Kind;
use crate::source::{Area, Catch, Credential, Source};

/// MarineTraffic tile zoom. 7 covers a country-sized area in one request;
/// higher zooms return fewer vessels over a smaller box.
const MARINE_ZOOM: u32 = 7;

/// Degrees of latitude and longitude either side of the point for the flight
/// feed — about 330 km, roughly one FIR.
const FLIGHT_SPAN_DEG: f64 = 3.0;

/// Bolt taxis. Needs a phone-OTP session, which the caller mints and keeps.
pub struct Bolt;

#[async_trait::async_trait]
impl Source for Bolt {
    fn id(&self) -> &'static str {
        "bolt"
    }
    fn label(&self) -> &'static str {
        "Bolt"
    }
    fn kind(&self) -> Kind {
        Kind::Taxi
    }
    fn credential(&self) -> Credential {
        Credential::Session { service: "bolt", name: "session" }
    }

    async fn fetch(&self, area: Area, secret: Option<&str>) -> Result<Catch> {
        let secret = secret.ok_or(Error::MissingCredential {
            source_id: "bolt",
            what: "a phone sign-in",
        })?;
        let session: crate::bolt::Session =
            serde_json::from_str(secret).map_err(|e| Error::Unauthorized {
                source_id: "bolt",
                detail: format!("the stored session is unreadable ({e}) — sign in again"),
            })?;

        let (vehicles, icons) = crate::bolt::service::fetch(&session, area.lat, area.lng).await?;
        Ok(Catch {
            vehicles: vehicles
                .iter()
                .map(crate::bolt::service::to_vehicle)
                .collect(),
            icons,
        })
    }
}

/// MarineTraffic vessels. Needs a Cloudflare clearance cookie, pasted in.
pub struct Marine;

#[async_trait::async_trait]
impl Source for Marine {
    fn id(&self) -> &'static str {
        "marine"
    }
    fn label(&self) -> &'static str {
        "MarineTraffic"
    }
    fn kind(&self) -> Kind {
        Kind::Vessel
    }
    fn credential(&self) -> Credential {
        Credential::Cookie { service: "marinetraffic", name: "cf_cookie" }
    }

    async fn fetch(&self, area: Area, secret: Option<&str>) -> Result<Catch> {
        // A missing cookie is reported, not returned as an empty sea — telling
        // those two apart is exactly what used to be impossible here.
        let (vehicles, _raw) =
            crate::marine::client::fetch(secret, area.lat, area.lng, MARINE_ZOOM).await?;
        Ok(Catch { vehicles, ..Default::default() })
    }
}

/// FlightRadar24 aircraft. Public.
pub struct Flights;

#[async_trait::async_trait]
impl Source for Flights {
    fn id(&self) -> &'static str {
        "flights"
    }
    fn label(&self) -> &'static str {
        "Flights"
    }
    fn kind(&self) -> Kind {
        Kind::Aircraft
    }
    fn credential(&self) -> Credential {
        Credential::None
    }

    async fn fetch(&self, area: Area, _secret: Option<&str>) -> Result<Catch> {
        Ok(Catch {
            vehicles: crate::flight::live::fetch(area.lat, area.lng, FLIGHT_SPAN_DEG).await?,
            ..Default::default()
        })
    }
}

/// SGR stations and timetables. Public, and not vehicles — it contributes
/// nothing to a map's vehicle layer, only to the summary.
pub struct Sgr;

#[async_trait::async_trait]
impl Source for Sgr {
    fn id(&self) -> &'static str {
        "sgr"
    }
    fn label(&self) -> &'static str {
        "SGR"
    }
    fn kind(&self) -> Kind {
        Kind::Train
    }
    fn credential(&self) -> Credential {
        Credential::None
    }

    async fn fetch(&self, _area: Area, _secret: Option<&str>) -> Result<Catch> {
        // Stations are places, not moving vehicles; fetching proves the feed is
        // reachable and the result is summarised rather than mapped.
        crate::sgr::public::stations().await?;
        Ok(Catch::default())
    }
}

/// Every source this library knows about, in the order a UI should list them.
pub fn all() -> Vec<Box<dyn Source>> {
    vec![Box::new(Bolt), Box::new(Marine), Box::new(Flights), Box::new(Sgr)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_source_has_a_distinct_id_and_a_label() {
        let sources = all();
        let mut ids: Vec<&str> = sources.iter().map(|s| s.id()).collect();
        let total = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), total, "two sources share an id");
        assert!(sources.iter().all(|s| !s.label().is_empty()));
    }

    /// Marker geometry is chosen from the kind, so no two sources may claim to
    /// carry the same sort of thing.
    #[test]
    fn every_source_carries_a_distinct_kind() {
        let sources = all();
        let mut kinds: Vec<&str> = sources.iter().map(|s| s.kind().as_str()).collect();
        let total = kinds.len();
        kinds.sort_unstable();
        kinds.dedup();
        assert_eq!(kinds.len(), total);
    }

    #[test]
    fn credentials_point_at_the_entries_that_exist() {
        let sources = all();
        let by = |id: &str| sources.iter().find(|s| s.id() == id).unwrap().credential();

        assert_eq!(by("marine").vault_key(), Some(("marinetraffic", "cf_cookie")));
        assert!(by("marine").is_pasted(), "the cookie is pasted by hand");

        assert_eq!(by("bolt").vault_key(), Some(("bolt", "session")));
        assert!(!by("bolt").is_pasted(), "the Bolt session is negotiated, not typed");

        assert_eq!(by("flights").vault_key(), None);
        assert_eq!(by("sgr").vault_key(), None);
    }

    /// A source that needs a credential and has none must say so, rather than
    /// return an empty catch that reads as "nothing here".
    #[tokio::test]
    async fn a_missing_credential_is_an_error_not_an_empty_result() {
        let area = Area { lat: -6.83, lng: 39.30 };

        let err = Bolt.fetch(area, None).await.unwrap_err();
        assert!(err.needs_operator());
        assert!(!err.is_retryable(), "retrying will not conjure a session");
        assert!(matches!(err, Error::MissingCredential { source_id: "bolt", .. }));

        let err = Bolt.fetch(area, Some("not json")).await.unwrap_err();
        assert!(err.to_string().contains("sign in again"), "unhelpful: {err}");
    }
}
