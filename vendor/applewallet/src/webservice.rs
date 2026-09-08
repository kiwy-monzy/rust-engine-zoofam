//! Apple Wallet pass-update web service: data model + a pluggable store.
//!
//! Endpoint shapes (mount these in your HTTP server, see mvt-server):
//!   POST   /v1/devices/{deviceId}/registrations/{passType}/{serial}   (register)
//!   DELETE /v1/devices/{deviceId}/registrations/{passType}/{serial}   (unregister)
//!   GET    /v1/devices/{deviceId}/registrations/{passType}?passesUpdatedSince=tag
//!   GET    /v1/passes/{passType}/{serial}                             (latest pass)
//!   POST   /v1/log
//!
//! All authenticated requests carry `Authorization: ApplePass <authToken>`.

use std::collections::HashMap;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A stored, updatable pass: its unsigned `pass.json` body plus the per-pass
/// auth token Wallet must present. We re-sign on demand at download time.
#[derive(Debug, Clone)]
pub struct PassRecord {
    pub pass_type: String,
    pub serial: String,
    pub auth_token: String,
    pub pass_json: Vec<u8>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DeviceRegistration {
    pub device_id: String,
    pub pass_type: String,
    pub serial: String,
    pub push_token: String,
}

#[derive(Debug, Serialize)]
pub struct SerialList {
    #[serde(rename = "lastUpdated")]
    pub last_updated: String,
    #[serde(rename = "serialNumbers")]
    pub serial_numbers: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegistrationBody {
    #[serde(rename = "pushToken")]
    pub push_token: String,
}

#[derive(Debug, Deserialize)]
pub struct LogBody {
    pub logs: Vec<String>,
}

/// Storage backend for passes + device registrations. Implement this over a DB
/// for production; [`InMemoryStore`] is provided for dev and tests.
pub trait PassStore: Send + Sync {
    fn upsert_pass(&self, record: PassRecord);
    fn get_pass(&self, pass_type: &str, serial: &str) -> Option<PassRecord>;

    fn register_device(&self, reg: DeviceRegistration);
    fn unregister_device(&self, device_id: &str, pass_type: &str, serial: &str);

    /// Serials registered to `device_id` for `pass_type`, updated strictly after
    /// `since` (RFC3339 tag) — or all if `since` is `None`. Returns the serials
    /// and the newest tag to hand back to Wallet.
    fn serials_updated_since(
        &self,
        device_id: &str,
        pass_type: &str,
        since: Option<&str>,
    ) -> (Vec<String>, String);

    /// All push tokens registered for a given pass (used to fan out APNs pushes).
    fn push_tokens_for_serial(&self, pass_type: &str, serial: &str) -> Vec<String>;
}

#[derive(Default)]
pub struct InMemoryStore {
    passes: Mutex<HashMap<(String, String), PassRecord>>,
    registrations: Mutex<Vec<DeviceRegistration>>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PassStore for InMemoryStore {
    fn upsert_pass(&self, record: PassRecord) {
        self.passes
            .lock()
            .unwrap()
            .insert((record.pass_type.clone(), record.serial.clone()), record);
    }

    fn get_pass(&self, pass_type: &str, serial: &str) -> Option<PassRecord> {
        self.passes
            .lock()
            .unwrap()
            .get(&(pass_type.to_string(), serial.to_string()))
            .cloned()
    }

    fn register_device(&self, reg: DeviceRegistration) {
        let mut regs = self.registrations.lock().unwrap();
        if let Some(existing) = regs.iter_mut().find(|r| {
            r.device_id == reg.device_id && r.pass_type == reg.pass_type && r.serial == reg.serial
        }) {
            existing.push_token = reg.push_token;
        } else {
            regs.push(reg);
        }
    }

    fn unregister_device(&self, device_id: &str, pass_type: &str, serial: &str) {
        self.registrations.lock().unwrap().retain(|r| {
            !(r.device_id == device_id && r.pass_type == pass_type && r.serial == serial)
        });
    }

    fn serials_updated_since(
        &self,
        device_id: &str,
        pass_type: &str,
        since: Option<&str>,
    ) -> (Vec<String>, String) {
        let since_dt = since.and_then(|s| DateTime::parse_from_rfc3339(s).ok());
        let regs = self.registrations.lock().unwrap();
        let passes = self.passes.lock().unwrap();

        let mut serials = Vec::new();
        let mut newest = since_dt.map(|d| d.with_timezone(&Utc)).unwrap_or_else(|| {
            DateTime::<Utc>::from_timestamp(0, 0).unwrap()
        });

        for reg in regs.iter() {
            if reg.device_id != device_id || reg.pass_type != pass_type {
                continue;
            }
            if let Some(rec) = passes.get(&(pass_type.to_string(), reg.serial.clone())) {
                let include = match since_dt {
                    Some(s) => rec.updated_at > s.with_timezone(&Utc),
                    None => true,
                };
                if include {
                    serials.push(reg.serial.clone());
                    if rec.updated_at > newest {
                        newest = rec.updated_at;
                    }
                }
            }
        }
        (serials, newest.to_rfc3339())
    }

    fn push_tokens_for_serial(&self, pass_type: &str, serial: &str) -> Vec<String> {
        self.registrations
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.pass_type == pass_type && r.serial == serial)
            .map(|r| r.push_token.clone())
            .collect()
    }
}
