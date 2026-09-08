//! # applewallet
//!
//! Apple Wallet (`.pkpass`) generator, signer and pass-update web service with
//! APNs push, extracted from the ehealth-apple-wallet project so it can be
//! reused (e.g. mounted in mvt-server's admin routes).
//!
//! ## Quick start
//! ```no_run
//! use applewallet::{PassKit, WalletConfig, samples};
//! let kit = PassKit::new(WalletConfig::default()).unwrap();
//! let pass = samples::sample_event_ticket();
//! let issued = kit.issue_updatable(&pass).unwrap();   // signed, updatable
//! std::fs::write("event.pkpass", issued.pkpass).unwrap();
//! ```
//!
//! The high-level [`PassKit`] handles config stamping, signing, an in-memory
//! pass/registration [`store`], QR generation and APNs pushes. Drop in your own
//! [`webservice::PassStore`] to persist to a database.

pub mod apns;
pub mod assets;
pub mod builder;
pub mod config;
pub mod error;
pub mod model;
pub mod samples;
pub mod sign;
pub mod webservice;

use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

pub use config::{ApnsEnvironment, WalletConfig};
pub use error::{WalletError, WalletResult};
pub use model::PKPass;

use apns::ApnsClient;
use sign::{Signer, SigningKeys};
use webservice::{InMemoryStore, PassRecord, PassStore};

/// A freshly issued, signed pass plus the credentials Wallet needs for updates.
pub struct IssuedPass {
    pub serial: String,
    pub pass_type: String,
    pub auth_token: String,
    pub pkpass: Vec<u8>,
}

/// High-level wallet engine: config + signer + store + (optional) APNs.
pub struct PassKit {
    pub config: WalletConfig,
    signer: Signer,
    store: Arc<dyn PassStore>,
    apns: Option<ApnsClient>,
}

impl PassKit {
    /// Load signing material from `config.priv_dir` and use an in-memory store.
    pub fn new(config: WalletConfig) -> WalletResult<Self> {
        Self::with_store(config, Arc::new(InMemoryStore::new()))
    }

    /// Same, but with a caller-supplied (e.g. DB-backed) store.
    pub fn with_store(config: WalletConfig, store: Arc<dyn PassStore>) -> WalletResult<Self> {
        let keys = SigningKeys::load_from_dir(&config.priv_dir)?;
        let signer = Signer::new(keys, config.assets_dir.as_deref());
        // APNs is optional: if the cert can't be turned into a client identity
        // we still serve/sign passes, just without live push.
        let apns = ApnsClient::from_priv_dir(&config.priv_dir, config.apns_environment).ok();
        Ok(Self {
            config,
            signer,
            store,
            apns,
        })
    }

    pub fn store(&self) -> &Arc<dyn PassStore> {
        &self.store
    }

    pub fn apns_ready(&self) -> bool {
        self.apns.is_some()
    }

    /// Stamp config-derived identity onto a pass. When `updatable`, also wires
    /// the update web service URL and a fresh per-pass auth token (returned).
    fn stamp(&self, pass: &PKPass, updatable: bool) -> (PKPass, String) {
        let mut p = pass.clone();
        p.type_id = self.config.pass_type_id.clone();
        p.team_id = self.config.team_id.clone();
        let auth = if updatable {
            let token = Uuid::new_v4().simple().to_string();
            p.web_service_url = Some(self.config.web_service_url());
            p.authentication_token = Some(token.clone());
            token
        } else {
            String::new()
        };
        (p, auth)
    }

    /// Sign a one-off pass (no update web service).
    pub fn sign_pass(&self, pass: &PKPass) -> WalletResult<Vec<u8>> {
        let (p, _) = self.stamp(pass, false);
        let body = serde_json::to_vec(&p)?;
        self.signer.sign(&body)
    }

    /// Sign raw `pass.json` bytes as-is (used when editing JSON in the admin UI).
    pub fn sign_json(&self, pass_json: &[u8]) -> WalletResult<Vec<u8>> {
        self.signer.sign(pass_json)
    }

    /// Issue an updatable pass: stamp + store the record + sign. The returned
    /// `auth_token`/`serial` let Wallet register for and fetch updates.
    pub fn issue_updatable(&self, pass: &PKPass) -> WalletResult<IssuedPass> {
        let (p, auth) = self.stamp(pass, true);
        let body = serde_json::to_vec(&p)?;
        self.store.upsert_pass(PassRecord {
            pass_type: p.type_id.clone(),
            serial: p.serial.clone(),
            auth_token: auth.clone(),
            pass_json: body.clone(),
            updated_at: Utc::now(),
        });
        let bytes = self.signer.sign(&body)?;
        Ok(IssuedPass {
            serial: p.serial,
            pass_type: p.type_id,
            auth_token: auth,
            pkpass: bytes,
        })
    }

    /// Re-sign a stored pass (for the web service "get latest pass" endpoint).
    pub fn sign_stored(&self, pass_type: &str, serial: &str) -> WalletResult<Vec<u8>> {
        let rec = self
            .store
            .get_pass(pass_type, serial)
            .ok_or_else(|| WalletError::UnknownSerial(serial.to_string()))?;
        self.signer.sign(&rec.pass_json)
    }

    /// Replace a stored pass body (e.g. new balance/seat) and bump its update
    /// time so Wallet sees it as changed. Follow with [`PassKit::push_update`].
    pub fn update_pass_json(&self, pass_type: &str, serial: &str, new_json: &[u8]) -> WalletResult<()> {
        let mut rec = self
            .store
            .get_pass(pass_type, serial)
            .ok_or_else(|| WalletError::UnknownSerial(serial.to_string()))?;
        rec.pass_json = new_json.to_vec();
        rec.updated_at = Utc::now();
        self.store.upsert_pass(rec);
        Ok(())
    }

    /// Check the `Authorization: ApplePass <token>` against a stored pass.
    pub fn verify_auth(&self, pass_type: &str, serial: &str, token: &str) -> bool {
        self.store
            .get_pass(pass_type, serial)
            .map(|r| r.auth_token == token)
            .unwrap_or(false)
    }

    /// Push an APNs update to every device registered for this pass. Returns the
    /// number of tokens successfully notified.
    pub async fn push_update(&self, pass_type: &str, serial: &str) -> WalletResult<usize> {
        let apns = self
            .apns
            .as_ref()
            .ok_or_else(|| WalletError::Apns("APNs client not configured".into()))?;
        let tokens = self.store.push_tokens_for_serial(pass_type, serial);
        let mut ok = 0;
        for token in tokens {
            match apns.push(&token, pass_type).await {
                Ok(_) => ok += 1,
                Err(e) => eprintln!("[applewallet] APNs push failed for {serial}: {e}"),
            }
        }
        Ok(ok)
    }

    /// Render an SVG QR code for arbitrary data (e.g. a LAN download URL) so a
    /// phone can scan it to add the pass.
    pub fn qr_svg(&self, data: &str) -> WalletResult<String> {
        use qrcode::render::svg;
        use qrcode::QrCode;
        let code = QrCode::new(data.as_bytes()).map_err(|e| WalletError::Qr(e.to_string()))?;
        Ok(code
            .render::<svg::Color>()
            .min_dimensions(220, 220)
            .quiet_zone(true)
            .build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::path::PathBuf;

    fn test_config() -> WalletConfig {
        // Tests run with CWD = crate dir; the workspace signing material lives
        // two levels up in the repo root `priv/`.
        WalletConfig {
            priv_dir: PathBuf::from("../../priv"),
            ..WalletConfig::default()
        }
    }

    #[test]
    fn signs_a_real_pkpass() {
        let kit = PassKit::new(test_config()).expect("load signing material");
        let pass = samples::sample_event_ticket();
        let bytes = kit.sign_pass(&pass).expect("sign");

        // It must be a valid zip containing the three required members.
        let mut zip =
            zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("open zip");
        let names: Vec<String> = (0..zip.len())
            .map(|i| zip.by_index(i).unwrap().name().to_string())
            .collect();
        assert!(names.contains(&"pass.json".to_string()));
        assert!(names.contains(&"manifest.json".to_string()));
        assert!(names.contains(&"signature".to_string()));
        assert!(names.iter().any(|n| n == "icon.png"));

        // pass.json must carry the stamped team/type identifiers.
        let mut pj = String::new();
        zip.by_name("pass.json").unwrap().read_to_string(&mut pj).unwrap();
        assert!(pj.contains("passTypeIdentifier"));
        assert!(pj.contains("teamIdentifier"));
    }

    #[test]
    fn apns_client_builds_from_certs() {
        // The Pass Type ID cert must form a valid rustls client identity for
        // certificate-based APNs push.
        let dir = std::path::PathBuf::from("../../priv");
        apns::ApnsClient::from_priv_dir(&dir, ApnsEnvironment::Production)
            .expect("APNs client builds from priv dir");
    }

    #[test]
    fn issues_updatable_pass_and_qr() {
        let kit = PassKit::new(test_config()).expect("load signing material");
        let issued = kit.issue_updatable(&samples::sample_coupon()).expect("issue");
        assert!(!issued.auth_token.is_empty());
        assert!(!issued.pkpass.is_empty());
        // stored + re-signable
        kit.sign_stored(&issued.pass_type, &issued.serial).expect("resign");
        assert!(kit.verify_auth(&issued.pass_type, &issued.serial, &issued.auth_token));
        // QR renders
        let svg = kit.qr_svg("http://192.168.1.10:5887/wallet/pass/x.pkpass").unwrap();
        assert!(svg.contains("<svg"));
    }
}
