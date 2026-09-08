//! `.pkpass` packaging + PKCS#7 detached signing (via OpenSSL), ported from the
//! proven ehealth-apple-wallet implementation.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use openssl::hash::{hash, MessageDigest};
use openssl::pkcs7::{Pkcs7, Pkcs7Flags};
use openssl::pkey::{PKey, Private};
use openssl::stack::Stack;
use openssl::x509::X509;

use crate::assets::{load_assets, Asset};
use crate::error::{WalletError, WalletResult};

/// The Pass Type ID certificate, its private key, and the WWDR intermediate.
pub struct SigningKeys {
    pub public_cert: X509,
    pub private_key: PKey<Private>,
    pub intermediate_certs: Stack<X509>,
}

fn read_first(dir: &Path, candidates: &[&str]) -> WalletResult<Vec<u8>> {
    for name in candidates {
        let path = dir.join(name);
        if let Ok(data) = std::fs::read(&path) {
            return Ok(data);
        }
    }
    Err(WalletError::Keys(format!(
        "none of {:?} found in {}",
        candidates,
        dir.display()
    )))
}

impl SigningKeys {
    /// Load the signing material from a directory. Accepts the usual filenames
    /// produced by `cer.bat` / downloaded from Apple. Certs are DER (`.cer`),
    /// the key is PKCS#8 DER (`pass.key`).
    pub fn load_from_dir(dir: &Path) -> WalletResult<Self> {
        let mut intermediate_certs = Stack::new()?;
        intermediate_certs.push(X509::from_der(&read_first(
            dir,
            &["AppleWWDRCAG4.cer", "AppleWWDRCA.cer"],
        )?)?)?;

        let public_cert = X509::from_der(&read_first(
            dir,
            &["pass.cer", "Apple Developer Certificate.cer"],
        )?)?;
        let private_key = PKey::private_key_from_der(&read_first(dir, &["pass.key"])?)?;

        Ok(Self {
            public_cert,
            private_key,
            intermediate_certs,
        })
    }
}

/// Signs `pass.json` + assets into a complete `.pkpass` archive.
pub struct Signer {
    keys: SigningKeys,
    assets: Vec<Asset>,
}

impl Signer {
    pub fn new(keys: SigningKeys, assets_dir: Option<&Path>) -> Self {
        Self {
            keys,
            assets: load_assets(assets_dir),
        }
    }

    /// Build a signed `.pkpass` from already-serialized `pass.json` bytes.
    pub fn sign(&self, pass_json: &[u8]) -> WalletResult<Vec<u8>> {
        let mut manifest = BTreeMap::<String, String>::new();
        let mut buf = Vec::new();
        let mut archive = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        let opts = zip::write::SimpleFileOptions::default();

        // pass.json
        archive.start_file("pass.json", opts)?;
        archive.write_all(pass_json)?;
        manifest.insert("pass.json".into(), sha1_hex(pass_json)?);

        // image assets
        for asset in &self.assets {
            archive.start_file(asset.name.clone(), opts)?;
            archive.write_all(&asset.bytes)?;
            manifest.insert(asset.name.clone(), sha1_hex(&asset.bytes)?);
        }

        // manifest.json
        let manifest_bytes = serde_json::to_vec(&manifest)?;
        archive.start_file("manifest.json", opts)?;
        archive.write_all(&manifest_bytes)?;

        // signature (PKCS#7 detached over manifest.json)
        let pkcs7 = Pkcs7::sign(
            self.keys.public_cert.as_ref(),
            self.keys.private_key.as_ref(),
            self.keys.intermediate_certs.as_ref(),
            &manifest_bytes,
            Pkcs7Flags::DETACHED | Pkcs7Flags::NOCRL | Pkcs7Flags::BINARY,
        )?;
        let signature = pkcs7.to_der()?;
        archive.start_file("signature", opts)?;
        archive.write_all(&signature)?;

        // finish() consumes the writer and returns the inner Cursor; dropping it
        // immediately releases the &mut borrow on `buf`.
        let _ = archive.finish()?;
        Ok(buf)
    }
}

fn sha1_hex(data: &[u8]) -> WalletResult<String> {
    Ok(hex::encode(hash(MessageDigest::sha1(), data)?))
}
