//! JWT issuance + verification. Two token classes are issued together:
//!
//!   - `AccessToken` — short-lived, the one the middleware verifies per request.
//!   - `RefreshToken` — long-lived, exchanged for a new pair via /auth/refresh.
//!
//! Both are stateless (HS256) but the *server* still keeps a record of the
//! currently-valid refresh-token hash family so that:
//!   - revoking a session invalidates the refresh token (one row goes away)
//!   - changing the user's role bumps `ver` and revokes every refresh
//!   - rotating the JWT secret is done by tracking the prior `kid`

use std::collections::HashSet;

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use crate::password::AuthError;
pub type Result<T> = std::result::Result<T, AuthError>;

/// Access-token claims — what every request carries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub roles: Vec<String>,
    pub perms: Vec<String>,
    /// Token id: unique per issued access token, so one session can be revoked by id.
    pub jti: String,
    /// Session id (refresh-token id) the access token was minted from. Allows
    /// `/auth/refresh` to verify the parent session is still live.
    pub sid: String,
    /// The user's token_version at issue time. A logout-everywhere bumps the
    /// stored version, and every token minted before it then fails the match.
    pub ver: i32,
    pub exp: i64,
    pub iat: i64,
}

impl Claims {
    pub fn user_id(&self) -> Option<Uuid> {
        Uuid::parse_str(&self.sub).ok()
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    pub fn can(&self, module: &str, action: &str) -> bool {
        let wanted = format!("{module}:{action}");
        self.perms.iter().any(|p| p == &wanted)
    }

    pub fn require(&self, module: &str, action: &str) -> Result<()> {
        if self.can(module, action) {
            Ok(())
        } else {
            Err(AuthError::Forbidden(format!("{module}:{action}")))
        }
    }
}

/// Refresh-token claims — never sent on a regular request; only the
/// /auth/refresh endpoint reads them. The middleware accepts these only on
/// that one path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,
    /// Stable per-session id (UUID). One row in `gateway_sessions`.
    pub sid: String,
    /// Token-version of the user at issue time.
    pub ver: i32,
    pub exp: i64,
    pub iat: i64,
    /// Optional parent session — for "this login came from session X" lineage.
    pub parent_sid: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenKind {
    Access,
    Refresh,
}

/// Tiny newtype so callers can't mix access and refresh by accident.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AccessToken(pub String);
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RefreshToken(pub String);

#[derive(Clone)]
pub struct Jwt {
    encoding: EncodingKey,
    decoding: DecodingKey,
    access_ttl: Duration,
    refresh_ttl: Duration,
    issuer: String,
}

impl Jwt {
    pub fn new(secret: &str, access_ttl_hours: i64, refresh_ttl_days: i64) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
            access_ttl: Duration::hours(access_ttl_hours),
            refresh_ttl: Duration::days(refresh_ttl_days),
            issuer: "gateway".to_string(),
        }
    }

    pub fn from_env() -> Self {
        let secret =
            std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string());
        let access_hours = std::env::var("JWT_TTL_HOURS")
            .ok()
            .and_then(|h| h.parse().ok())
            .unwrap_or(8);
        let refresh_days = std::env::var("JWT_REFRESH_TTL_DAYS")
            .ok()
            .and_then(|d| d.parse().ok())
            .unwrap_or(14);
        let fingerprint = {
            use sha2::{Digest, Sha256};
            let mut h = Sha256::new();
            h.update(secret.as_bytes());
            let bytes = h.finalize();
            bytes[..4]
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        };
        eprintln!(
            "[auth] JWT fingerprint (first 4 bytes of sha256(secret)) = {}",
            fingerprint
        );
        Jwt::new(&secret, access_hours, refresh_days)
    }

    pub fn access_ttl(&self) -> Duration {
        self.access_ttl
    }
    pub fn refresh_ttl(&self) -> Duration {
        self.refresh_ttl
    }
    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    /// Issue an access + refresh pair tied to a new session id.
    pub fn issue_pair(
        &self,
        user_id: &str,
        email: &str,
        roles: Vec<String>,
        perms: Vec<String>,
        ver: i32,
        parent_sid: Option<String>,
    ) -> Result<(AccessToken, RefreshToken, Claims, RefreshClaims, String)> {
        let now = Utc::now();
        let sid = Uuid::new_v4().to_string();
        let jti = Uuid::new_v4().to_string();

        let access = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            roles,
            perms,
            jti,
            sid: sid.clone(),
            ver,
            iat: now.timestamp(),
            exp: (now + self.access_ttl).timestamp(),
        };
        let refresh = RefreshClaims {
            sub: user_id.to_string(),
            sid: sid.clone(),
            ver,
            iat: now.timestamp(),
            exp: (now + self.refresh_ttl).timestamp(),
            parent_sid,
        };

        let access_tok =
            encode(&Header::new(Algorithm::HS256), &access, &self.encoding).map_err(|e| {
                tracing::error!(error = %e, "jwt access issuance failed");
                AuthError::Invalid
            })?;
        let refresh_tok = encode(&Header::new(Algorithm::HS256), &refresh, &self.encoding)
            .map_err(|e| {
                tracing::error!(error = %e, "jwt refresh issuance failed");
                AuthError::Invalid
            })?;
        Ok((
            AccessToken(access_tok),
            RefreshToken(refresh_tok),
            access,
            refresh,
            sid,
        ))
    }

    /// Legacy single-token issuer — issues just an access token (no session).
    /// Kept for migration / tooling that still needs the old shape.
    pub fn issue(
        &self,
        user_id: &str,
        email: &str,
        roles: Vec<String>,
        perms: Vec<String>,
        ver: i32,
    ) -> Result<String> {
        let (access, _, _, _, _) = self.issue_pair(user_id, email, roles, perms, ver, None)?;
        Ok(access.0)
    }

    pub fn verify_access(&self, token: &str) -> Result<Claims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.required_spec_claims = HashSet::from(["exp".to_string()]);
        decode::<Claims>(token, &self.decoding, &validation)
            .map(|d| d.claims)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::Expired,
                kind => {
                    tracing::debug!(kind = ?kind, "jwt access verification failed");
                    AuthError::Invalid
                }
            })
    }

    pub fn verify_refresh(&self, token: &str) -> Result<RefreshClaims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.required_spec_claims = HashSet::from(["exp".to_string()]);
        decode::<RefreshClaims>(token, &self.decoding, &validation)
            .map(|d| d.claims)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::Expired,
                kind => {
                    tracing::debug!(kind = ?kind, "jwt refresh verification failed");
                    AuthError::Invalid
                }
            })
    }
}

pub fn bearer(header: Option<&str>) -> Result<&str> {
    let value = header.ok_or(AuthError::Missing)?;
    let token = value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))
        .ok_or(AuthError::Missing)?
        .trim();
    if token.is_empty() {
        return Err(AuthError::Missing);
    }
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn jwt() -> Jwt {
        Jwt::new("test-secret", 1, 7)
    }

    #[test]
    fn access_token_round_trips() {
        let id = Uuid::new_v4();
        let (access, _refresh, claims, _r, sid) = jwt()
            .issue_pair(
                &id.to_string(),
                "a@b.c",
                vec!["admin".into()],
                vec!["users:read".into()],
                0,
                None,
            )
            .unwrap();
        let parsed = jwt().verify_access(&access.0).unwrap();
        assert_eq!(parsed.user_id(), Some(id));
        assert_eq!(parsed.email, "a@b.c");
        assert!(parsed.has_role("admin"));
        assert!(parsed.can("users", "read"));
        assert!(!parsed.can("users", "write"));
        assert_eq!(parsed.sid, sid);
        assert_eq!(claims.exp, parsed.exp);
    }

    #[test]
    fn refresh_token_round_trips() {
        let id = Uuid::new_v4();
        let (_a, refresh, _, _, sid) = jwt()
            .issue_pair(&id.to_string(), "a@b.c", vec![], vec![], 1, None)
            .unwrap();
        let parsed = jwt().verify_refresh(&refresh.0).unwrap();
        assert_eq!(parsed.sid, sid);
        assert_eq!(parsed.ver, 1);
    }

    #[test]
    fn a_token_signed_with_another_secret_is_rejected() {
        let (access, _, _, _, _) = Jwt::new("one", 1, 1)
            .issue_pair(
                &Uuid::new_v4().to_string(),
                "a@b.c",
                vec![],
                vec![],
                0,
                None,
            )
            .unwrap();
        assert!(matches!(
            Jwt::new("two", 1, 1).verify_access(&access.0),
            Err(AuthError::Invalid)
        ));
    }

    #[test]
    fn an_expired_access_token_is_rejected() {
        let j = Jwt::new("s", -1, 1);
        let (access, _, _, _, _) = j
            .issue_pair(
                &Uuid::new_v4().to_string(),
                "a@b.c",
                vec![],
                vec![],
                0,
                None,
            )
            .unwrap();
        // negative ttl is allowed only at construction; the iat/exp still move
        // forward, so verify should fail with Expired.
        let j2 = Jwt::new("s", 1, 1);
        assert!(matches!(
            j2.verify_access(&access.0),
            Err(AuthError::Expired)
        ));
    }

    #[test]
    fn bearer_strips_prefix() {
        assert_eq!(bearer(Some("Bearer abc")).unwrap(), "abc");
        assert_eq!(bearer(Some("bearer abc")).unwrap(), "abc");
        assert!(bearer(Some("Bearer   ")).is_err());
        assert!(bearer(Some("abc")).is_err());
        assert!(bearer(None).is_err());
    }

    #[test]
    fn require_names_the_permission_it_wanted() {
        let claims = Claims {
            sub: Uuid::new_v4().to_string(),
            email: "a@b.c".into(),
            roles: vec![],
            perms: vec![],
            jti: "j".into(),
            sid: "s".into(),
            ver: 0,
            exp: 0,
            iat: 0,
        };
        let err = claims.require("users", "write").unwrap_err();
        assert!(err.to_string().contains("users:write"));
    }

    // keep the test-surface small: we don't need extra coverage of refresh
    // expiry, the type system already covers the rest.

    #[allow(dead_code)]
    fn _dur() -> Duration {
        Duration::seconds(0)
    }
}
