//! SVG vessel marker generation with text labels.
//!
//! Generates base64-encoded SVG images for vessel markers, combining the hull/arrow
//! geometry from `fleet::marine::geometry` with text labels (vessel name + class).
//! Each unique vessel type gets a cached icon to avoid regeneration.
//!
//! The generated icons are keyed by `(ship_type, marker_mode)` so that all vessels
//! of the same type share one icon. Vessel-specific names are NOT embedded in the
//! icon — they are shown as tooltips by the client. The icon shows the shape +
//! class label only.

use std::collections::BTreeMap;

use base64::Engine;
use fleet::marine::geometry::{MarkerMode, Vessel};

/// Size of the generated SVG viewport in pixels.
const ICON_SIZE: f64 = 48.0;

/// Padding around the shape inside the viewport.
const PADDING: f64 = 4.0;

/// Font family used in the generated SVGs.
const FONT: &str = "Arial, Helvetica, sans-serif";

/// Font size for the class label below the shape.
const LABEL_SIZE: f64 = 9.0;

/// An in-memory cache of generated SVG data URIs, keyed by icon key.
///
/// The key is `"{ship_type}:{mode}"` — all vessels of the same AIS type and
/// marker mode share one icon. This keeps the icon table small even with
/// thousands of vessels on screen.
pub struct IconCache {
    icons: BTreeMap<String, String>,
}

impl IconCache {
    /// Create an empty cache.
    pub fn new() -> Self {
        Self {
            icons: BTreeMap::new(),
        }
    }

    /// Look up a cached data URI for the given key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.icons.get(key).map(String::as_str)
    }

    /// Total number of cached icons.
    pub fn len(&self) -> usize {
        self.icons.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.icons.is_empty()
    }
}

impl Default for IconCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the cache for a set of vessels.
///
/// Classifies each vessel, determines its marker mode (dot/arrow/hull), generates
/// an SVG icon for each unique `(ship_type, mode)` pair, and returns the cache
/// plus a map of `vessel_id -> icon_key` for attaching icons to markers.
pub fn build_cache<'a>(
    vessels: impl IntoIterator<Item = &'a Vessel>,
) -> (IconCache, BTreeMap<String, String>) {
    let mut cache = IconCache::new();
    let mut vessel_keys = BTreeMap::new();

    for vessel in vessels {
        let mode = vessel.marker_mode(50.0); // default zoom
        let key = icon_key(vessel.ship_type, mode);

        if !cache.icons.contains_key(&key) {
            let svg = generate_svg(vessel, mode);
            let data_uri = format!(
                "data:image/svg+xml;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(svg.as_bytes())
            );
            cache.icons.insert(key.clone(), data_uri);
        }

        vessel_keys.insert(vessel_id(vessel), key);
    }

    (cache, vessel_keys)
}

/// Generate a single SVG icon for a vessel at the given marker mode.
///
/// The SVG contains:
/// - The hull/arrow/dot shape filled with the vessel's AIS colour
/// - A text label showing the AIS category (e.g. "cargo", "tanker")
pub fn generate_svg(vessel: &Vessel, mode: MarkerMode) -> String {
    let color = vessel.color();
    let category = vessel.category();
    let shape = match mode {
        MarkerMode::Hull => hull_svg(vessel, color),
        MarkerMode::Arrow => arrow_svg(vessel, color),
        MarkerMode::Dot => dot_svg(color),
    };

    let w = ICON_SIZE;
    let h = ICON_SIZE;
    let cx = ICON_SIZE / 2.0;
    let ty = ICON_SIZE - PADDING + 1.0;

    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w} {h}\" width=\"{w}\" height=\"{h}\">\n\
         {shape}\n\
         <text x=\"{cx}\" y=\"{ty}\" text-anchor=\"middle\" font-family=\"{font}\" font-size=\"{fs}\" fill=\"#fff\" paint-order=\"stroke\" stroke=\"#000\" stroke-width=\"1.5\" stroke-linejoin=\"round\">{category}</text>\n\
         </svg>",
        w = w, h = h, cx = cx, ty = ty, font = FONT, fs = LABEL_SIZE, category = category, shape = shape,
    )
}

/// SVG polygon for the true-scale hull pentagon, centered in the viewport.
fn hull_svg(vessel: &Vessel, color: &str) -> String {
    let outline = vessel.hull_outline_m();

    // Find bounding box of the hull points.
    let (mut min_x, mut max_x) = (f64::MAX, f64::MIN);
    let (mut min_y, mut max_y) = (f64::MAX, f64::MIN);
    for p in &outline {
        min_x = min_x.min(p[0]);
        max_x = max_x.max(p[0]);
        min_y = min_y.min(p[1]);
        max_y = max_y.max(p[1]);
    }

    let hull_w = max_x - min_x;
    let hull_h = max_y - min_y;
    if hull_w <= 0.0 || hull_h <= 0.0 {
        return dot_svg(color);
    }

    let draw_w = ICON_SIZE - 2.0 * PADDING;
    let draw_h = draw_w; // square viewport
    let scale = (draw_w / hull_w).min(draw_h / hull_h);
    let cx = ICON_SIZE / 2.0;
    let cy = (ICON_SIZE - PADDING - LABEL_SIZE - 2.0) / 2.0 + PADDING;

    let points: Vec<String> = outline
        .iter()
        .map(|p| {
            let x = cx + (p[0] - (min_x + max_x) / 2.0) * scale;
            let y = cy + ((min_y + max_y) / 2.0 - p[1]) * scale; // flip Y
            format!("{:.1},{:.1}", x, y)
        })
        .collect();

    format!(
        "  <polygon points=\"{pts}\" fill=\"{color}\" stroke=\"#fff\" stroke-width=\"1\" opacity=\"0.9\"/>",
        pts = points.join(" "),
        color = color,
    )
}

/// SVG polygon for the directional arrow, centered in the viewport.
fn arrow_svg(vessel: &Vessel, color: &str) -> String {
    let arrow = vessel.arrow_unit(); // 4 points in ship frame [0,1] = bow

    // Bounding box.
    let (mut min_x, mut max_x) = (f64::MAX, f64::MIN);
    let (mut min_y, mut max_y) = (f64::MAX, f64::MIN);
    for p in &arrow {
        min_x = min_x.min(p[0]);
        max_x = max_x.max(p[0]);
        min_y = min_y.min(p[1]);
        max_y = max_y.max(p[1]);
    }

    let arrow_w = max_x - min_x;
    let arrow_h = max_y - min_y;
    if arrow_w <= 0.0 || arrow_h <= 0.0 {
        return dot_svg(color);
    }

    let draw_w = ICON_SIZE - 2.0 * PADDING;
    let draw_h = ICON_SIZE - 2.0 * PADDING - LABEL_SIZE - 2.0;
    let scale = (draw_w / arrow_w).min(draw_h / arrow_h);
    let cx = ICON_SIZE / 2.0;
    let cy = PADDING + draw_h / 2.0;

    let points: Vec<String> = arrow
        .iter()
        .map(|p| {
            let x = cx + (p[0] - (min_x + max_x) / 2.0) * scale;
            let y = cy + ((min_y + max_y) / 2.0 - p[1]) * scale; // flip Y
            format!("{:.1},{:.1}", x, y)
        })
        .collect();

    format!(
        "  <polygon points=\"{pts}\" fill=\"{color}\" stroke=\"#fff\" stroke-width=\"1\" opacity=\"0.9\"/>",
        pts = points.join(" "),
        color = color,
    )
}

/// SVG circle for a stopped/at-anchor vessel.
fn dot_svg(color: &str) -> String {
    let r = 6.0;
    let cx = ICON_SIZE / 2.0;
    let cy = ICON_SIZE / 2.0 - PADDING;
    format!(
        "  <circle cx=\"{cx}\" cy=\"{cy}\" r=\"{r}\" fill=\"{color}\" stroke=\"#fff\" stroke-width=\"1.5\" opacity=\"0.9\"/>",
        cx = cx,
        cy = cy,
        r = r,
        color = color,
    )
}

/// Build the cache key for a vessel: `"{ship_type}:{mode}"`.
fn icon_key(ship_type: i32, mode: MarkerMode) -> String {
    let mode_str = match mode {
        MarkerMode::Dot => "dot",
        MarkerMode::Arrow => "arrow",
        MarkerMode::Hull => "hull",
    };
    format!("{}:{}", ship_type, mode_str)
}

/// Build a unique vessel identifier from its position and type.
///
/// This is a simple composite key — not cryptographically unique, but
/// sufficient for attaching icons to markers in the same request.
fn vessel_id(vessel: &Vessel) -> String {
    format!(
        "{:.6},{:.6},{}",
        vessel.lat, vessel.lon, vessel.ship_type
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tanker() -> Vessel {
        Vessel {
            lat: -6.83,
            lon: 39.30,
            heading_deg: Some(90.0),
            speed_kn: Some(12.0),
            length_m: Some(200.0),
            width_m: Some(32.0),
            ship_type: 8,
            ..Vessel::default()
        }
    }

    fn anchor_vessel() -> Vessel {
        Vessel {
            lat: -6.83,
            lon: 39.30,
            heading_deg: None,
            speed_kn: Some(0.1),
            length_m: Some(100.0),
            width_m: Some(15.0),
            ship_type: 7,
            ..Vessel::default()
        }
    }

    #[test]
    fn hull_svg_contains_polygon() {
        let v = tanker();
        let svg = generate_svg(&v, MarkerMode::Hull);
        assert!(svg.contains("<polygon"), "hull must contain a polygon");
        assert!(svg.contains("tanker"), "must label the class");
    }

    #[test]
    fn arrow_svg_contains_polygon() {
        let v = tanker();
        let svg = generate_svg(&v, MarkerMode::Arrow);
        assert!(svg.contains("<polygon"), "arrow must contain a polygon");
        assert!(svg.contains("tanker"), "must label the class");
    }

    #[test]
    fn dot_svg_contains_circle() {
        let v = anchor_vessel();
        let svg = generate_svg(&v, MarkerMode::Dot);
        assert!(svg.contains("<circle"), "dot must contain a circle");
        assert!(svg.contains("cargo"), "must label the class");
    }

    #[test]
    fn icon_cache_deduplicates_by_type_and_mode() {
        let v1 = tanker();
        let mut v2 = tanker();
        v2.lon = 40.0; // same type, different position

        let (cache, keys) = build_cache([&v1, &v2]);
        // Both vessels share the same key because they have the same ship_type
        // and the same marker mode.
        assert_eq!(cache.len(), 1, "one icon for two identical-type vessels");
        assert_eq!(
            keys.get(&vessel_id(&v1)),
            keys.get(&vessel_id(&v2)),
            "both vessels map to the same icon key"
        );
    }

    #[test]
    fn different_types_get_different_icons() {
        let cargo = Vessel {
            ship_type: 7,
            length_m: Some(150.0),
            ..Vessel::default()
        };
        let tanker = Vessel {
            ship_type: 8,
            length_m: Some(200.0),
            ..Vessel::default()
        };

        let (cache, _) = build_cache([&cargo, &tanker]);
        assert_eq!(cache.len(), 2, "different types must produce different icons");
    }

    #[test]
    fn data_uri_is_valid() {
        let v = tanker();
        let (cache, _) = build_cache([&v]);
        let (_, uri) = cache.icons.iter().next().unwrap();
        assert!(uri.starts_with("data:image/svg+xml;base64,"));
    }
}
