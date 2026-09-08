//! Embedded base64 icons for bolt taxi types
//! These are decoded at runtime to avoid file path issues

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;

/// Get the base64 encoded icon data for a given icon name
/// Returns None if the icon is not found
pub fn get_icon_base64(icon_name: &str) -> Option<Vec<u8>> {
    let b64 = match icon_name {
        "basic" => BASIC_ICON,
        "bolt" => BOLT_ICON,
        "boda" => BODA_ICON,
        "bajaji" => BAJAJI_ICON,
        "electric" => ELECTRIC_ICON,
        "xl" => XL_ICON,
        "bolt_airport" => BOLT_AIRPORT_ICON,
        _ => return None,
    };

    // The .b64 asset files are MIME-wrapped (newline every 76 chars) which the
    // strict STANDARD decoder rejects — strip all whitespace first.
    let clean: String = b64.split_whitespace().collect();
    BASE64.decode(clean).ok()
}

// Base64 encoded PNG icons (32x32 pixels each)
// These are embedded to avoid file path resolution issues

const BASIC_ICON: &str = include_str!("../../assets/basic.b64");
const BOLT_ICON: &str = include_str!("../../assets/bolt.b64");
const BODA_ICON: &str = include_str!("../../assets/boda.b64");
const BAJAJI_ICON: &str = include_str!("../../assets/bajaji.b64");
const ELECTRIC_ICON: &str = include_str!("../../assets/electric.b64");
const XL_ICON: &str = include_str!("../../assets/xl.b64");
const BOLT_AIRPORT_ICON: &str = include_str!("../../assets/bolt_airport.b64");
