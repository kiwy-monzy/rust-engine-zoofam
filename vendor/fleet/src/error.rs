//! One error type for every source.
//!
//! Each upstream crate used to bring its own — `BoltError`, `FlightRadarError`,
//! `SgrError` — so a caller polling all four had to match on three unrelated
//! enums and invent a fourth to hold them. They collapse here.
//!
//! The distinction that matters to a caller is not *which* source failed but
//! *what kind* of failure it was: a missing credential is fixed by signing in,
//! [`Error::Blocked`] by waiting or by pasting a fresh cookie, and
//! [`Error::Transport`] by retrying. That is why those are separate variants
//! rather than one stringly-typed `ApiError`.

/// What went wrong talking to a source.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The source needs a credential and none was supplied.
    ///
    /// Deliberately distinct from an empty result. Reporting "no vessels" when
    /// the truth was "no cookie" is exactly the fault that made a Cloudflare
    /// block look like an empty sea for weeks.
    #[error("{source_id} needs a credential: {what}")]
    MissingCredential {
        source_id: &'static str,
        what: &'static str,
    },

    /// A credential was supplied but the source rejected it.
    #[error("{source_id} rejected the stored credential: {detail}")]
    Unauthorized {
        source_id: &'static str,
        detail: String,
    },

    /// The source answered, but with a bot check rather than data — a
    /// Cloudflare interstitial, a 403, a challenge page.
    #[error("{source_id} is blocking automated access (HTTP {status})")]
    Blocked {
        source_id: &'static str,
        status: u16,
    },

    /// Asked to slow down.
    #[error("{source_id} is rate limiting (HTTP 429)")]
    RateLimited { source_id: &'static str },

    /// The response arrived but did not parse: a moved field, a changed
    /// envelope, a truncated protobuf.
    #[error("{source_id} sent something unreadable: {detail}")]
    Malformed {
        source_id: &'static str,
        detail: String,
    },

    /// The request never completed.
    #[cfg(feature = "net")]
    #[error("network error: {0}")]
    Transport(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[cfg(feature = "flight")]
    #[error("protobuf error: {0}")]
    Protobuf(#[from] prost::DecodeError),
}

impl Error {
    /// Whether trying the same request again could plausibly succeed.
    ///
    /// Callers use this to decide between backing off and surfacing the fault
    /// to an operator: no amount of retrying supplies a missing cookie.
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::RateLimited { .. } => true,
            #[cfg(feature = "net")]
            Error::Transport(e) => e.is_timeout() || e.is_connect(),
            Error::MissingCredential { .. }
            | Error::Unauthorized { .. }
            | Error::Blocked { .. }
            | Error::Malformed { .. }
            | Error::Json(_) => false,
            #[cfg(feature = "flight")]
            Error::Protobuf(_) => false,
        }
    }

    /// Whether an operator has to do something before this source works again.
    pub fn needs_operator(&self) -> bool {
        matches!(
            self,
            Error::MissingCredential { .. } | Error::Unauthorized { .. } | Error::Blocked { .. }
        )
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_credential_is_never_retried_but_always_reported() {
        let e = Error::MissingCredential { source_id: "bolt", what: "a phone sign-in" };
        assert!(!e.is_retryable(), "retrying will not conjure a session");
        assert!(e.needs_operator());
    }

    #[test]
    fn rate_limiting_is_retried_and_needs_nobody() {
        let e = Error::RateLimited { source_id: "flights" };
        assert!(e.is_retryable());
        assert!(!e.needs_operator(), "waiting is not an operator task");
    }

    /// The message has to name the source: a poller walking four of them logs
    /// these next to each other.
    #[test]
    fn every_message_names_its_source() {
        let cases = [
            Error::MissingCredential { source_id: "marine", what: "a clearance cookie" },
            Error::Blocked { source_id: "marine", status: 403 },
            Error::RateLimited { source_id: "marine" },
            Error::Malformed { source_id: "marine", detail: "no rows".into() },
        ];
        for e in cases {
            assert!(e.to_string().contains("marine"), "unattributed: {e}");
        }
    }
}
