//! What every fleet source has in common.
//!
//! The four look nothing alike underneath — Bolt mints a session from a phone
//! OTP, MarineTraffic wants a Cloudflare cookie pasted in, flights and SGR need
//! no account at all — but a caller only ever asks them three questions:
//!
//! 1. *what credential do you need, and have you got it?*
//! 2. *fetch what you can see from here.*
//! 3. *what came back?*
//!
//! [`Source`] is those three questions. Implement it and a new source appears
//! in every caller — poller, admin UI, map — without any of them learning its
//! name.

use std::collections::BTreeMap;

#[cfg(feature = "net")]
use crate::error::Result;
use crate::model::Vehicle;

/// What a source needs before it can fetch anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Credential {
    /// Public data. Flights and SGR timetables need no account.
    None,
    /// An opaque header string an operator pastes in, stored under
    /// `(service, name)`. MarineTraffic's Cloudflare clearance is one.
    Cookie {
        service: &'static str,
        name: &'static str,
    },
    /// A session the caller establishes and then keeps — Bolt's phone OTP mints
    /// one, and it is refreshed rather than re-typed.
    Session {
        service: &'static str,
        name: &'static str,
    },
}

impl Credential {
    /// Where this credential is stored, if it is stored at all.
    pub fn vault_key(&self) -> Option<(&'static str, &'static str)> {
        match self {
            Credential::None => None,
            Credential::Cookie { service, name } | Credential::Session { service, name } => {
                Some((*service, *name))
            }
        }
    }

    /// Whether an operator supplies this by hand.
    ///
    /// A cookie is pasted; a session is negotiated. Offering a paste box for a
    /// session would invite someone to type something that cannot work.
    pub fn is_pasted(&self) -> bool {
        matches!(self, Credential::Cookie { .. })
    }
}

/// A geographic window to fetch. Sources that ignore it — SGR timetables, a
/// global flight feed — simply do not read it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Area {
    pub lat: f64,
    pub lng: f64,
}

/// What one fetch produced.
#[derive(Debug, Default)]
pub struct Catch {
    pub vehicles: Vec<Vehicle>,
    /// `icon id → absolute URL`, as advertised by this poll.
    ///
    /// Kept separate from the vehicles because it is a *per-poll dictionary*:
    /// Bolt hands out a fresh id→URL map with every response and the ids are
    /// only meaningful within it, which is why a vehicle's icon cannot be
    /// resolved from a table baked in at compile time.
    pub icons: BTreeMap<String, String>,
}

/// One live source of vehicles.
///
/// Object-safe on purpose: callers hold `Vec<Box<dyn Source>>` and walk it, so
/// adding a source is a registration rather than another arm in three separate
/// `match`es.
#[cfg(feature = "net")]
#[async_trait::async_trait]
pub trait Source: Send + Sync {
    /// Stable key. Names the rows it stores and the credential it owns.
    fn id(&self) -> &'static str;

    /// What to call it in a user interface.
    fn label(&self) -> &'static str;

    /// What kind of thing this source tracks.
    fn kind(&self) -> crate::model::Kind;

    fn credential(&self) -> Credential;

    /// Fetch around `area`, given the credential's stored secret.
    ///
    /// `secret` is whatever was stored for [`Credential::vault_key`], or `None`
    /// for a public source. A source that needs one and did not get one must
    /// return [`crate::Error::MissingCredential`] — never an empty catch. "No
    /// credential" and "nothing in this area" must not look alike.
    async fn fetch(&self, area: Area, secret: Option<&str>) -> Result<Catch>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_pasted_credentials_offer_a_paste_box() {
        assert!(Credential::Cookie { service: "marinetraffic", name: "cf_cookie" }.is_pasted());
        assert!(!Credential::Session { service: "bolt", name: "session" }.is_pasted());
        assert!(!Credential::None.is_pasted());
    }

    #[test]
    fn public_sources_own_no_stored_credential() {
        assert_eq!(Credential::None.vault_key(), None);
        assert_eq!(
            Credential::Cookie { service: "marinetraffic", name: "cf_cookie" }.vault_key(),
            Some(("marinetraffic", "cf_cookie"))
        );
    }
}
