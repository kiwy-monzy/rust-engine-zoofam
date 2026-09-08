#![warn(missing_docs)]
//! Vessel marker geometry and styling for AIS / marine-radar style map displays.
//!
//! This crate is **renderer-agnostic**: it turns a [`Vessel`]'s AIS attributes
//! (position, heading, dimensions, type) into the shapes and colours that
//! navigation displays use to draw ships, and leaves the actual drawing to you
//! (galileo, egui, canvas, SVG, …).
//!
//! # The three marker shapes
//! Navigation displays don't draw a ship as a plain dot — they draw a
//! *directional* symbol, and switch representation with zoom and motion:
//!
//! | Situation | Shape | Method |
//! |-----------|-------|--------|
//! | Under way, small on screen | directional arrow (fixed pixel size) | [`Vessel::arrow_unit`] |
//! | Under way, large on screen | true-scale hull pentagon | [`Vessel::hull_lonlat`] |
//! | Stopped / at anchor | circle | (draw a dot) |
//!
//! Pick the right one with [`Vessel::marker_mode`].
//!
//! # Coordinate conventions
//! * Ship frame: `x` = starboard (right), `y` = forward (bow), metres.
//! * Heading / course: degrees clockwise from north (0 = N, 90 = E).
//! * Geographic output is `[lon, lat]` (GeoJSON order).
//!
//! # Example
//! ```
//! use fleet::marine::{Vessel, MarkerMode};
//!
//! let ship = Vessel {
//!     lat: -6.83, lon: 39.30,
//!     heading_deg: Some(45.0),
//!     course_deg: Some(50.0),
//!     speed_kn: Some(12.0),
//!     length_m: Some(180.0),
//!     width_m: Some(28.0),
//!     ship_type: 8, // tanker
//!     ..Vessel::default()
//! };
//!
//! assert_eq!(ship.color(), "#e74c3c");      // tanker red
//! assert!(ship.is_moving());
//! // At 50 m/px the 180 m hull is ~3.6 px -> draw the arrow, not the hull.
//! assert_eq!(ship.marker_mode(50.0), MarkerMode::Arrow);
//! // Zoomed in to 2 m/px the hull is 90 px -> draw the true-scale hull.
//! assert_eq!(ship.marker_mode(2.0), MarkerMode::Hull);
//!
//! let hull = ship.hull_lonlat(); // 5 [lon, lat] points, rotated to heading
//! assert_eq!(hull.len(), 5);
//! ```

/// Speed (knots) below which a vessel is treated as stopped / at anchor.
pub const ANCHOR_SPEED_KN: f64 = 0.5;

/// Minimum on-screen hull length (pixels) before switching from the arrow
/// marker to the true-scale hull pentagon.
pub const HULL_MIN_PX: f64 = 12.0;

/// Metres per degree of latitude (WGS84 mean).
const M_PER_DEG: f64 = 111_320.0;

/// A vessel's live AIS state — the input to all marker calculations.
///
/// Build one directly, or from a MarineTraffic tile row via
/// [`crate::marine::client`].
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Vessel {
    /// Latitude of the reported position (degrees).
    pub lat: f64,
    /// Longitude of the reported position (degrees).
    pub lon: f64,
    /// True heading, degrees clockwise from north. `None` if not transmitted.
    pub heading_deg: Option<f64>,
    /// Course over ground, degrees clockwise from north. Used to orient the
    /// marker when `heading_deg` is missing.
    pub course_deg: Option<f64>,
    /// Speed over ground (knots).
    pub speed_kn: Option<f64>,
    /// Overall length (metres), AIS `A + B`.
    pub length_m: Option<f64>,
    /// Overall beam / width (metres), AIS `C + D`.
    pub width_m: Option<f64>,
    /// AIS ship-type code (single MarineTraffic digit or full 2-digit code).
    pub ship_type: i32,
    /// Distance from the bow to the AIS antenna reference point (metres, AIS `A`).
    /// When set with [`Vessel::w_left`] the hull is positioned precisely about
    /// the reported point; otherwise the hull is centred.
    pub l_fore: Option<f64>,
    /// Distance from the port side to the AIS antenna reference point (metres,
    /// AIS `C`).
    pub w_left: Option<f64>,
}

impl Default for Vessel {
    fn default() -> Self {
        Self {
            lat: 0.0,
            lon: 0.0,
            heading_deg: None,
            course_deg: None,
            speed_kn: None,
            length_m: None,
            width_m: None,
            ship_type: 0,
            l_fore: None,
            w_left: None,
        }
    }
}

/// Which symbol a display should draw for a vessel at the current zoom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerMode {
    /// Stopped / at anchor — draw a circle.
    Dot,
    /// Under way but small on screen — draw a fixed-size directional arrow.
    Arrow,
    /// Under way and large enough — draw the true-scale hull pentagon.
    Hull,
}

impl Vessel {
    /// `true` if the vessel is moving faster than [`ANCHOR_SPEED_KN`].
    pub fn is_moving(&self) -> bool {
        self.speed_kn.is_some_and(|s| s >= ANCHOR_SPEED_KN)
    }

    /// Orientation to draw the marker at (degrees clockwise from north):
    /// heading if available, otherwise course over ground.
    pub fn orientation_deg(&self) -> Option<f64> {
        self.heading_deg.or(self.course_deg)
    }

    /// Choose the marker representation for the given map scale.
    ///
    /// `meters_per_pixel` is the map resolution (e.g. galileo `view().resolution()`).
    pub fn marker_mode(&self, meters_per_pixel: f64) -> MarkerMode {
        if !self.is_moving() {
            return MarkerMode::Dot;
        }
        let length = self.length_m.unwrap_or(0.0);
        if meters_per_pixel > 0.0 && length / meters_per_pixel >= HULL_MIN_PX {
            MarkerMode::Hull
        } else {
            MarkerMode::Arrow
        }
    }

    /// AIS category (`"cargo"`, `"tanker"`, `"passenger"`, …).
    pub fn category(&self) -> &'static str {
        ais_category(self.ship_type.max(0) as u32)
    }

    /// Display colour (hex) for this vessel's type.
    pub fn color(&self) -> &'static str {
        ais_color(self.ship_type.max(0) as u32)
    }

    /// Size class and a suggested arrow scale factor, from length.
    pub fn size_class(&self) -> (&'static str, f64) {
        ship_size_class(self.length_m)
    }

    /// True-scale hull outline as 5 points in the **ship frame** (metres),
    /// `[x = starboard, y = forward]`, ordered bow → starboard-stern →
    /// port-stern → bow. The reference point is the AIS antenna when
    /// [`Vessel::l_fore`]/[`Vessel::w_left`] are set, else the hull centre.
    ///
    /// Defaults of 30 m × 8 m are used when dimensions are missing.
    pub fn hull_outline_m(&self) -> [[f64; 2]; 5] {
        let length = self.length_m.filter(|l| *l > 0.0).unwrap_or(30.0);
        let width = self.width_m.filter(|w| *w > 0.0).unwrap_or(8.0);

        // Edge offsets relative to the reference point.
        let (bow_y, stern_y, right_x, left_x) = match (self.l_fore, self.w_left) {
            (Some(a), Some(c)) => (a, -(length - a), width - c, -c),
            _ => (length / 2.0, -length / 2.0, width / 2.0, -width / 2.0),
        };
        let center_x = (right_x + left_x) / 2.0;
        let taper = (length * 0.25).min(length); // pointed bow section

        [
            [center_x, bow_y],           // bow tip (on centreline)
            [right_x, bow_y - taper],    // starboard shoulder
            [right_x, stern_y],          // starboard stern
            [left_x, stern_y],           // port stern
            [left_x, bow_y - taper],     // port shoulder
        ]
    }

    /// True-scale hull as geographic `[lon, lat]` points, rotated to the
    /// vessel's orientation and placed at its reported position. Ready to feed
    /// to a polygon renderer. Empty if there is no orientation to draw at.
    pub fn hull_lonlat(&self) -> Vec<[f64; 2]> {
        let Some(theta) = self.orientation_deg() else {
            return Vec::new();
        };
        self.hull_outline_m()
            .iter()
            .map(|p| ship_to_lonlat(*p, theta, self.lat, self.lon))
            .collect()
    }

    /// A unit directional arrow in the ship frame (`y` forward), for the small
    /// fixed-pixel-size marker. Scale by the desired pixel size and rotate by
    /// [`Vessel::orientation_deg`] before drawing.
    pub fn arrow_unit(&self) -> [[f64; 2]; 4] {
        [
            [0.0, 1.0],    // bow
            [0.6, -0.8],   // starboard tail
            [0.0, -0.4],   // notch
            [-0.6, -0.8],  // port tail
        ]
    }

    /// Directional arrow as geographic `[lon, lat]` points, `size_m` giving the
    /// bow distance from the centre (overall length ≈ `1.8 * size_m`), rotated
    /// to the vessel's orientation (north if unknown) and placed at its
    /// position. Pass `size_m = pixels * meters_per_pixel` for a constant
    /// on-screen size.
    pub fn arrow_lonlat(&self, size_m: f64) -> Vec<[f64; 2]> {
        let theta = self.orientation_deg().unwrap_or(0.0);
        self.arrow_unit()
            .iter()
            .map(|p| ship_to_lonlat([p[0] * size_m, p[1] * size_m], theta, self.lat, self.lon))
            .collect()
    }

    /// A small non-directional square (for stopped / no-heading vessels),
    /// `size_m` across, as geographic `[lon, lat]` points.
    pub fn square_lonlat(&self, size_m: f64) -> Vec<[f64; 2]> {
        let h = size_m / 2.0;
        [[-h, -h], [h, -h], [h, h], [-h, h]]
            .iter()
            .map(|p| ship_to_lonlat(*p, 0.0, self.lat, self.lon))
            .collect()
    }
}

/// Rotate a ship-frame point `[x = starboard, y = forward]` (metres) by heading
/// `theta` (degrees CW from north) and offset it from `(lat, lon)`, returning
/// `[lon, lat]`.
fn ship_to_lonlat(p: [f64; 2], theta_deg: f64, lat: f64, lon: f64) -> [f64; 2] {
    let (x, y) = (p[0], p[1]);
    let t = theta_deg.to_radians();
    let (sin, cos) = (t.sin(), t.cos());
    // Bow direction = (east=sin, north=cos); starboard = bow rotated +90° CW.
    let east = x * cos + y * sin;
    let north = -x * sin + y * cos;
    let dlat = north / M_PER_DEG;
    let dlon = east / (M_PER_DEG * lat.to_radians().cos().max(1e-6));
    [lon + dlon, lat + dlat]
}

// ------------------------------------------------------------------------------------------------
// AIS classification (self-contained; mirrors MarineTraffic SHIPTYPE handling).
// ------------------------------------------------------------------------------------------------

/// Ship category from an AIS / MarineTraffic ship-type code.
///
/// MarineTraffic's tile API returns a single digit (the first digit of the AIS
/// code); those are matched first, then the full 2-digit AIS ranges.
/// Reference: ITU-R M.1371-5 Table 53 + MarineTraffic SHIPTYPE.
pub fn ais_category(shiptype: u32) -> &'static str {
    match shiptype {
        1 => return "navaid",
        3 => return "tug",
        4 => return "highspeed",
        5 => return "tug",
        6 => return "passenger",
        7 => return "cargo",
        8 => return "tanker",
        9 => return "other",
        _ => {}
    }
    match shiptype {
        20..=29 => "wig",
        30 => "fishing",
        31..=32 => "tug",
        33 => "dredger",
        34 => "diving",
        35 => "military",
        36 => "sailing",
        37 => "pleasure",
        40..=49 => "highspeed",
        50 => "pilot",
        51 => "sar",
        52 => "tug",
        53 => "port_tender",
        55 => "law_enforcement",
        60..=69 => "passenger",
        70..=79 => "cargo",
        80..=89 => "tanker",
        90..=99 => "other",
        _ => "unknown",
    }
}

/// Display colour (hex) for a vessel's type.
pub fn ais_color(shiptype: u32) -> &'static str {
    match ais_category(shiptype) {
        "cargo" => "#2ecc71",
        "tanker" => "#e74c3c",
        "passenger" => "#3498db",
        "highspeed" => "#f1c40f",
        "tug" => "#5dade2",
        "fishing" => "#e67e22",
        "pleasure" => "#8e44ad",
        "sailing" => "#9b59b6",
        "military" => "#1a3c6e",
        "sar" => "#e74c3c",
        "pilot" => "#16a085",
        "law_enforcement" => "#2c3e50",
        "dredger" => "#95a5a6",
        "wig" => "#f39c12",
        "other" => "#7f8c8d",
        _ => "#bdc3c7",
    }
}

/// Size class and suggested arrow scale factor from overall length.
pub fn ship_size_class(length_m: Option<f64>) -> (&'static str, f64) {
    match length_m {
        Some(l) if l >= 300.0 => ("mega", 1.8),
        Some(l) if l >= 200.0 => ("large", 1.4),
        Some(l) if l >= 100.0 => ("medium", 1.1),
        Some(l) if l >= 50.0 => ("small", 0.85),
        Some(l) if l > 0.0 => ("tiny", 0.6),
        _ => ("unknown", 0.7),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn tanker() -> Vessel {
        Vessel {
            lat: -6.83,
            lon: 39.30,
            heading_deg: Some(90.0), // due east
            speed_kn: Some(12.0),
            length_m: Some(200.0),
            width_m: Some(32.0),
            ship_type: 8,
            ..Vessel::default()
        }
    }

    #[test]
    fn mode_depends_on_speed_and_scale() {
        let mut v = tanker();
        assert_eq!(v.marker_mode(50.0), MarkerMode::Arrow);
        assert_eq!(v.marker_mode(2.0), MarkerMode::Hull);
        v.speed_kn = Some(0.1);
        assert_eq!(v.marker_mode(2.0), MarkerMode::Dot);
    }

    #[test]
    fn color_and_category() {
        assert_eq!(tanker().category(), "tanker");
        assert_eq!(tanker().color(), "#e74c3c");
    }

    #[test]
    fn hull_has_five_points_and_bow_points_east() {
        let v = tanker();
        let hull = v.hull_lonlat();
        assert_eq!(hull.len(), 5);
        // Heading due east -> bow tip should be east (larger lon) of the centre.
        let bow = hull[0];
        assert!(bow[0] > v.lon, "bow lon {} should be east of {}", bow[0], v.lon);
    }
}
