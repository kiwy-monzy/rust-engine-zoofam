//! Runtime configuration for the wallet engine.

use std::path::PathBuf;

/// APNs environment. Wallet push always uses production unless you are testing
/// with a sandbox-provisioned device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApnsEnvironment {
    Production,
    Sandbox,
}

impl ApnsEnvironment {
    pub fn host(self) -> &'static str {
        match self {
            ApnsEnvironment::Production => "api.push.apple.com",
            ApnsEnvironment::Sandbox => "api.sandbox.push.apple.com",
        }
    }
}

#[derive(Debug, Clone)]
pub struct WalletConfig {
    /// e.g. `pass.tz.ticketevent.pass.sample` — must match the signing cert.
    pub pass_type_id: String,
    /// Apple Developer Team ID, e.g. `29S6Z4Y4MS`.
    pub team_id: String,
    /// Directory holding `pass.cer` / `Apple Developer Certificate.cer`,
    /// `pass.key` (PKCS#8 DER) and `AppleWWDRCAG4.cer`.
    pub priv_dir: PathBuf,
    /// Directory of pass image assets (icon/logo …). Falls back to the assets
    /// embedded in the binary when the path is missing/empty.
    pub assets_dir: Option<PathBuf>,
    /// Public base URL the iPhone can reach (LAN IP in dev), e.g.
    /// `http://192.168.1.20:5887`. Used for the pass-update `webServiceURL`
    /// and for the QR/download links.
    pub base_url: String,
    /// Path prefix the wallet web service is mounted under, e.g. `/wallet`.
    pub web_service_path: String,
    pub apns_environment: ApnsEnvironment,
}

impl WalletConfig {
    /// Full `webServiceURL` embedded in updatable passes.
    pub fn web_service_url(&self) -> String {
        format!(
            "{}{}",
            self.base_url.trim_end_matches('/'),
            self.web_service_path
        )
    }
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            pass_type_id: "pass.tz.ticketevent.pass.sample".to_string(),
            team_id: "29S6Z4Y4MS".to_string(),
            priv_dir: PathBuf::from("priv"),
            assets_dir: None,
            base_url: "http://127.0.0.1:5887".to_string(),
            web_service_path: "/wallet".to_string(),
            apns_environment: ApnsEnvironment::Production,
        }
    }
}
