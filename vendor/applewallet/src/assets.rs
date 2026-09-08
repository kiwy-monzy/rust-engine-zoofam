//! Pass image assets (icon/logo …). The default set is embedded in the binary
//! so a pass can always be signed; an on-disk directory can override it.

use std::path::Path;

use include_dir::{include_dir, Dir};

static EMBEDDED: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets");

/// One named file destined for the `.pkpass` archive root.
pub struct Asset {
    pub name: String,
    pub bytes: Vec<u8>,
}

/// Load the embedded assets, optionally overlaid by files in `dir`.
/// Files in `dir` with the same name win; extra files are added.
pub fn load_assets(dir: Option<&Path>) -> Vec<Asset> {
    let mut out: Vec<Asset> = EMBEDDED
        .files()
        .map(|f| Asset {
            name: f.path().file_name().unwrap().to_string_lossy().to_string(),
            bytes: f.contents().to_vec(),
        })
        .collect();

    if let Some(dir) = dir {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let name = match path.file_name() {
                    Some(n) => n.to_string_lossy().to_string(),
                    None => continue,
                };
                // Only image assets belong in the archive.
                if !name.to_ascii_lowercase().ends_with(".png") {
                    continue;
                }
                if let Ok(bytes) = std::fs::read(&path) {
                    match out.iter_mut().find(|a| a.name == name) {
                        Some(existing) => existing.bytes = bytes,
                        None => out.push(Asset { name, bytes }),
                    }
                }
            }
        }
    }
    out
}
