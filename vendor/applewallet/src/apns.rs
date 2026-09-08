//! APNs push for Wallet updates.
//!
//! Wallet uses **certificate-based** APNs: the Pass Type ID certificate is the
//! client identity, the `apns-topic` is the pass type id, and the payload is an
//! empty `{}` — it just nudges the device to call the web service for the new
//! pass. We send over HTTP/2 to `api.push.apple.com`.

use std::path::Path;

use openssl::pkey::{PKey, Private};
use openssl::x509::X509;

use crate::config::ApnsEnvironment;
use crate::error::{WalletError, WalletResult};

pub struct ApnsClient {
    client: reqwest::Client,
    host: &'static str,
}

impl ApnsClient {
    /// Build from a single PEM bundle containing the certificate chain followed
    /// by the PKCS#8 private key (the form reqwest's rustls identity expects).
    pub fn from_pem_bundle(bundle_pem: &[u8], env: ApnsEnvironment) -> WalletResult<Self> {
        let identity = reqwest::Identity::from_pem(bundle_pem)
            .map_err(|e| WalletError::Apns(format!("identity: {e}")))?;
        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .identity(identity)
            .build()
            .map_err(|e| WalletError::Apns(format!("client: {e}")))?;
        Ok(Self {
            client,
            host: env.host(),
        })
    }

    /// Build the APNs identity straight from the DER signing material in
    /// `priv_dir` (`pass.cer` + `pass.key` + WWDR), converting to PEM in memory.
    pub fn from_priv_dir(dir: &Path, env: ApnsEnvironment) -> WalletResult<Self> {
        let read = |names: &[&str]| -> WalletResult<Vec<u8>> {
            for n in names {
                if let Ok(b) = std::fs::read(dir.join(n)) {
                    return Ok(b);
                }
            }
            Err(WalletError::Apns(format!("missing {names:?} in {}", dir.display())))
        };

        let leaf = X509::from_der(&read(&["pass.cer", "Apple Developer Certificate.cer"])?)?;
        let wwdr = X509::from_der(&read(&["AppleWWDRCAG4.cer", "AppleWWDRCA.cer"])?)?;
        let key: PKey<Private> = PKey::private_key_from_der(&read(&["pass.key"])?)?;

        // reqwest's rustls identity wants one PEM: cert chain (leaf first) then
        // the PKCS#8 private key.
        let mut bundle = leaf.to_pem()?;
        bundle.extend_from_slice(&wwdr.to_pem()?);
        bundle.extend_from_slice(&key.private_key_to_pem_pkcs8()?);

        Self::from_pem_bundle(&bundle, env)
    }

    /// Send the empty Wallet push to one device token for `topic` (the pass
    /// type id). Returns the APNs `apns-id`, or an error with the status/reason.
    pub async fn push(&self, device_token: &str, topic: &str) -> WalletResult<String> {
        let url = format!("https://{}/3/device/{}", self.host, device_token);
        let resp = self
            .client
            .post(&url)
            .header("apns-topic", topic)
            .header("apns-push-type", "background")
            .header("apns-priority", "5")
            .body("{}")
            .send()
            .await
            .map_err(|e| WalletError::Apns(format!("send: {e}")))?;

        if resp.status().is_success() {
            let id = resp
                .headers()
                .get("apns-id")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();
            Ok(id)
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(WalletError::Apns(format!("APNs {status}: {body}")))
        }
    }
}
