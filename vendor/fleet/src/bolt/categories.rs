//! Normalisation of Bolt ride-option / pickup-data responses into a clean,
//! typed list of ride categories the UI can render directly.
//!
//! Bolt's `getRideOptions` and `getPickupData` responses are deeply nested and
//! their exact shape varies by region / app version, so instead of hard-coding
//! one path we walk the JSON recursively and pick out every "category-like"
//! object — one that carries an id/name together with pricing, an ETA, or an
//! icon. Each becomes a [`RideCategory`] with the display name, the human price
//! string, a numeric price range, the currency, the pickup ETA, and the
//! category's icon URL (so the UI shows the real Bolt artwork per category).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One ride category ready for display (name + price + eta + icon).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RideCategory {
    /// Stable category id (used to request a precise per-category estimate).
    pub id: String,
    /// Display name, e.g. "Economy", "Bolt XL", "Boda".
    pub name: String,
    /// Human-readable price as Bolt formats it, e.g. "TZS 6,500".
    pub price_text: Option<String>,
    /// Numeric low/high estimate (same currency), when available.
    pub price_min: Option<f64>,
    pub price_max: Option<f64>,
    /// ISO currency / symbol Bolt returned (e.g. "TZS").
    pub currency: Option<String>,
    /// Human ETA text ("3 min") when present.
    pub eta_text: Option<String>,
    /// Pickup ETA in seconds, when present.
    pub eta_seconds: Option<f64>,
    /// Absolute URL of the category icon, when Bolt provides one.
    pub icon_url: Option<String>,
}

impl RideCategory {
    fn is_useful(&self) -> bool {
        !self.name.is_empty()
            && (self.price_text.is_some()
                || self.price_min.is_some()
                || self.eta_seconds.is_some()
                || self.eta_text.is_some()
                || self.icon_url.is_some())
    }
}

fn as_str(v: &Value) -> Option<String> {
    v.as_str().map(|s| s.to_string()).filter(|s| !s.is_empty())
}

fn first_str(obj: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|k| obj.get(*k).and_then(as_str))
}

fn first_f64(obj: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter().find_map(|k| obj.get(*k).and_then(num))
}

/// Coerce a JSON number, or a numeric string ("6500", "6,500.0"), to f64.
fn num(v: &Value) -> Option<f64> {
    if let Some(n) = v.as_f64() {
        return Some(n);
    }
    let s = v.as_str()?;
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();
    cleaned.parse::<f64>().ok()
}

/// Pull a price string out of any of the common Bolt shapes.
fn price_text(obj: &Value, currency: &Option<String>) -> Option<String> {
    if let Some(s) = first_str(
        obj,
        &[
            "price_str",
            "price_text",
            "formatted_price",
            "display_price",
            "price_string",
            "estimated_price_str",
        ],
    ) {
        return Some(s);
    }
    // A plain string `price` field.
    if let Some(s) = obj.get("price").and_then(as_str) {
        return Some(s);
    }
    let cur = currency.clone().unwrap_or_default();
    let low = first_f64(
        obj,
        &["price_min", "min_price", "estimated_price_min", "low_price"],
    );
    let high = first_f64(
        obj,
        &["price_max", "max_price", "estimated_price_max", "high_price"],
    );
    match (low, high) {
        (Some(lo), Some(hi)) if (hi - lo).abs() > f64::EPSILON => {
            Some(format!("{cur} {}-{}", fmt_amount(lo), fmt_amount(hi)).trim().to_string())
        }
        (Some(lo), _) => Some(format!("{cur} {}", fmt_amount(lo)).trim().to_string()),
        _ => {
            let amount = first_f64(obj, &["amount", "value", "estimated_price", "total"]);
            amount.map(|a| format!("{cur} {}", fmt_amount(a)).trim().to_string())
        }
    }
}

fn fmt_amount(a: f64) -> String {
    if a.fract().abs() < f64::EPSILON {
        // Thousands separators for whole amounts.
        let n = a as i64;
        let s = n.abs().to_string();
        let mut out = String::new();
        for (i, ch) in s.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                out.push(',');
            }
            out.push(ch);
        }
        let body: String = out.chars().rev().collect();
        if n < 0 {
            format!("-{body}")
        } else {
            body
        }
    } else {
        format!("{a:.2}")
    }
}

/// Find an icon URL directly on this object (a string url field, or a nested
/// `{ url|image_url|... }` under an icon/image/logo-ish key).
fn icon_url(obj: &Value) -> Option<String> {
    let map = obj.as_object()?;
    // Direct string url fields whose key hints at an image.
    for (k, v) in map {
        let kl = k.to_ascii_lowercase();
        if (kl.contains("icon") || kl.contains("image") || kl.contains("logo"))
            && v.is_string()
        {
            if let Some(s) = as_str(v) {
                if s.starts_with("http") {
                    return Some(s);
                }
            }
        }
    }
    // Nested object holding a url.
    for (k, v) in map {
        let kl = k.to_ascii_lowercase();
        if kl.contains("icon") || kl.contains("image") || kl.contains("logo") {
            if let Some(s) = first_str(v, &["url", "image_url", "src", "png", "svg"]) {
                if s.starts_with("http") {
                    return Some(s);
                }
            }
        }
    }
    None
}

fn build_category(obj: &Value) -> Option<RideCategory> {
    let id = first_str(obj, &["category_id", "id", "product_id", "key"])
        .or_else(|| obj.get("product").and_then(|p| first_str(p, &["id", "category_id"])));
    let name = first_str(obj, &["name", "title", "display_name", "label"]).or_else(|| {
        obj.get("product")
            .or_else(|| obj.get("category"))
            .and_then(|p| first_str(p, &["name", "title"]))
    });
    // Must look like a category: have at least an id or a name.
    let name = name?;
    let id = id.unwrap_or_else(|| name.clone());

    let currency = first_str(
        obj,
        &["currency", "currency_code", "currency_symbol", "ccy"],
    );

    // Pricing may live on the object itself or in a nested `price`/`pricing`/
    // `estimate` block — check both.
    let price_obj = obj
        .get("price")
        .filter(|v| v.is_object())
        .or_else(|| obj.get("pricing"))
        .or_else(|| obj.get("estimate"))
        .unwrap_or(obj);
    let currency = currency.or_else(|| {
        first_str(
            price_obj,
            &["currency", "currency_code", "currency_symbol", "ccy"],
        )
    });
    let price_text = price_text(price_obj, &currency).or_else(|| price_text(obj, &currency));
    let price_min = first_f64(
        price_obj,
        &["price_min", "min_price", "estimated_price_min", "low_price"],
    );
    let price_max = first_f64(
        price_obj,
        &["price_max", "max_price", "estimated_price_max", "high_price"],
    );

    let eta_seconds = first_f64(obj, &["eta", "pickup_eta", "eta_seconds", "pickup_eta_seconds"]);
    let eta_text = first_str(obj, &["eta_text", "duration_text", "pickup_eta_text"]).or_else(|| {
        eta_seconds.map(|s| format!("{} min", (s / 60.0).round().max(1.0) as i64))
    });

    let icon_url = icon_url(obj);

    let cat = RideCategory {
        id,
        name,
        price_text,
        price_min,
        price_max,
        currency,
        eta_text,
        eta_seconds,
        icon_url,
    };
    if cat.is_useful() {
        Some(cat)
    } else {
        None
    }
}

/// Recursively walk a Bolt response and extract all ride categories, de-duped
/// by id, preserving first-seen order. Works on `getRideOptions` and
/// `getPickupData` payloads alike.
pub fn parse_categories(value: &Value) -> Vec<RideCategory> {
    let mut out: Vec<RideCategory> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    fn walk(v: &Value, out: &mut Vec<RideCategory>, seen: &mut std::collections::HashSet<String>) {
        match v {
            Value::Array(arr) => {
                for item in arr {
                    walk(item, out, seen);
                }
            }
            Value::Object(_) => {
                if let Some(cat) = build_category(v) {
                    let key = format!("{}::{}", cat.id, cat.name);
                    if seen.insert(key) {
                        // Merge a later, richer entry (e.g. pickup_data price) into
                        // an earlier name-only one would require keying by id alone;
                        // we keep it simple and keep the first useful hit.
                        out.push(cat);
                    }
                }
                if let Some(map) = v.as_object() {
                    for child in map.values() {
                        walk(child, out, seen);
                    }
                }
            }
            _ => {}
        }
    }

    walk(value, &mut out, &mut seen);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_categories_with_price_and_icon() {
        let resp = json!({
            "code": 0,
            "message": "OK",
            "data": {
                "categories": [
                    {
                        "id": "economy",
                        "name": "Economy",
                        "icon": { "url": "https://static.bolt.eu/economy.png" },
                        "price": {
                            "price_str": "TZS 6,500",
                            "price_min": 6000,
                            "price_max": 7000,
                            "currency": "TZS"
                        },
                        "eta": 180
                    },
                    {
                        "category_id": "xl",
                        "title": "Bolt XL",
                        "image_url": "https://static.bolt.eu/xl.png",
                        "amount": 12000,
                        "currency_code": "TZS"
                    }
                ]
            }
        });
        let cats = parse_categories(&resp);
        assert_eq!(cats.len(), 2);

        let eco = &cats[0];
        assert_eq!(eco.id, "economy");
        assert_eq!(eco.name, "Economy");
        assert_eq!(eco.price_text.as_deref(), Some("TZS 6,500"));
        assert_eq!(eco.price_min, Some(6000.0));
        assert_eq!(eco.icon_url.as_deref(), Some("https://static.bolt.eu/economy.png"));
        assert_eq!(eco.eta_text.as_deref(), Some("3 min"));

        let xl = &cats[1];
        assert_eq!(xl.id, "xl");
        assert_eq!(xl.name, "Bolt XL");
        assert_eq!(xl.icon_url.as_deref(), Some("https://static.bolt.eu/xl.png"));
        assert_eq!(xl.price_text.as_deref(), Some("TZS 12,000"));
    }

    #[test]
    fn ignores_non_category_objects() {
        let resp = json!({ "data": { "viewport": { "lat": -6.8, "lng": 39.2 } } });
        assert!(parse_categories(&resp).is_empty());
    }
}
