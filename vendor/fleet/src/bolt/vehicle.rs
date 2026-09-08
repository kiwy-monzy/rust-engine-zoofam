use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: String,
    pub lat: f64,
    pub lng: f64,
    pub bearing: f64,
    pub icon_id: String,
    /// Category group key from the poll response (e.g. "boda", "taxi", "premium").
    /// Used as a human-friendly fallback name when `icon_id` is not in our table.
    #[serde(default)]
    pub category: String,
}

/// Known icon mapping from API `icon_id` to a local icon filename, or `None`
/// when the id has never been seen before (so callers can record + fall back).
pub fn get_icon_filename_opt(icon_id: &str) -> Option<&'static str> {
    Some(match icon_id {
        "178" => "bajaji",
        "201" => "xl",
        "231" => "basic",
        "278" => "boda",
        "8749" => "bolt",
        "9172" => "electric",
        "9195" => "bolt_airport",
        _ => return None,
    })
}

/// Known vehicle-type display name from `icon_id`, or `None` if unknown.
pub fn get_vehicle_type_opt(icon_id: &str) -> Option<&'static str> {
    Some(match icon_id {
        "178" => "Bajaji",
        "201" => "XL",
        "231" => "Basic",
        "278" => "Boda",
        "8749" => "Send Motorbike",
        "9172" => "Electric",
        "9195" => "Bolt Airport",
        _ => return None,
    })
}

/// Back-compat: icon filename, defaulting unknown ids to the generic "basic" icon.
pub fn get_icon_filename(icon_id: &str) -> &'static str {
    get_icon_filename_opt(icon_id).unwrap_or("basic")
}

/// Back-compat: vehicle type name, defaulting unknown ids to "Unknown".
pub fn get_vehicle_type(icon_id: &str) -> &'static str {
    get_vehicle_type_opt(icon_id).unwrap_or("Unknown")
}

/// Turn a raw category key like "boda_boda" / "premium" into a display name.
pub fn prettify_category(key: &str) -> String {
    let cleaned = key.trim().replace(['_', '-'], " ");
    let mut out = String::with_capacity(cleaned.len());
    for (i, word) in cleaned.split_whitespace().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(&chars.as_str().to_ascii_lowercase());
        }
    }
    if out.is_empty() {
        "Vehicle".to_string()
    } else {
        out
    }
}

#[derive(Deserialize, Debug)]
pub struct VehicleResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<VehicleData>,
}

#[derive(Deserialize, Debug)]
pub struct VehicleData {
    pub vehicles: Option<Vehicles>,
}

#[derive(Deserialize, Debug)]
pub struct Vehicles {
    pub taxi: Option<std::collections::HashMap<String, Vec<ApiVehicle>>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiVehicle {
    pub id: String,
    pub lat: f64,
    pub lng: f64,
    pub bearing: f64,
    pub icon_id: String,
}

impl From<ApiVehicle> for Vehicle {
    fn from(api: ApiVehicle) -> Self {
        Self {
            id: api.id,
            lat: api.lat,
            lng: api.lng,
            bearing: api.bearing,
            icon_id: api.icon_id,
            category: String::new(),
        }
    }
}
