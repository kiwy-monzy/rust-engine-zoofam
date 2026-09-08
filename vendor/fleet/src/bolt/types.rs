//! Value types shared by the Bolt HTTP client. **No persistence** — callers
//! (e.g. libqaul) store sessions in their own databases.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Bolt API session / device identity used across start → confirm → poll.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub device_id: String,
    pub auth_username: String,
    pub phone: String,
    pub created_at: DateTime<Utc>,
    pub status: SessionStatus,
    /// Bolt JSON `type` for start/confirm: `sms`, `phone`, or `whatsapp`.
    #[serde(default = "default_verification_channel")]
    pub verification_channel: String,
}

fn default_verification_channel() -> String {
    "phone".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    Pending,
    Active,
    Expired,
}

/// Who and where we are claiming to be, for one sign-in.
///
/// These five values used to travel as loose arguments through
/// `start_verification`, `confirm_verification` and everything that called
/// them — eight and seven parameters respectively, four of them `f64`s and
/// `&str`s in a row that the compiler was happy to see transposed. They are one
/// thing conceptually (a device, somewhere) and are one type now.
#[derive(Debug, Clone)]
pub struct Device {
    /// Stable per-installation identifier. Bolt ties the session to it, so it
    /// must be the same on `start` and `confirm` and should persist for as long
    /// as the session does.
    pub uuid: String,
    pub lat: f64,
    pub lng: f64,
    /// ISO-3166-1 alpha-2. Lowercased before it reaches the API.
    pub country: String,
    /// IANA name, e.g. `Africa/Dar_es_Salaam`.
    pub timezone: String,
}
