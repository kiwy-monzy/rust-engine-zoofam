//! AIS vessel classification and the colours a map draws it in.
//!
//! The numeric ship types vessels broadcast, and the names and colours a
//! display gives them. The classification lives here rather than in a client
//! because every client must reach the same answer about what a ship *is*.
//!
//! MarineTraffic's tile API returns a **single-digit** SHIPTYPE (the first
//! digit of the AIS code), so those are matched first; the full two-digit
//! ITU-R M.1371-5 Table 53 ranges are the fallback.

/// Vessel category from an AIS/MarineTraffic ship type code.
pub fn ais_category(shiptype: u32) -> &'static str {
    // Single-digit MarineTraffic SHIPTYPE — the common case from the tile API.
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
        38..=39 => "reserved",
        40..=49 => "highspeed",
        50 => "pilot",
        51 => "sar",
        52 => "tug",
        53 => "port_tender",
        54 => "anti_pollution",
        55 => "law_enforcement",
        56..=57 => "local",
        58 => "medical",
        59 => "noncombatant",
        60..=69 => "passenger",
        70..=79 => "cargo",
        80..=89 => "tanker",
        90..=99 => "other",
        _ => "unknown",
    }
}

/// Display colour for a vessel category.
pub fn category_color(category: &str) -> &'static str {
    match category {
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

/// Colour for a vessel straight from its type code.
pub fn ais_color(shiptype: u32) -> &'static str {
    category_color(ais_category(shiptype))
}

/// Flights are coloured by altitude band rather than by type: at a glance the
/// useful question about an aircraft is how high it is, not what model it is.
/// Ground and approach traffic reads warm, cruise reads cool.
pub fn altitude_color(feet: Option<f64>) -> &'static str {
    match feet {
        None => "#9aa5b4",
        Some(ft) if ft < 1_000.0 => "#e74c3c",
        Some(ft) if ft < 10_000.0 => "#e67e22",
        Some(ft) if ft < 20_000.0 => "#f1c40f",
        Some(ft) if ft < 30_000.0 => "#2ecc71",
        Some(ft) if ft < 40_000.0 => "#3498db",
        Some(_) => "#9b59b6",
    }
}

/// Marker diameter in screen pixels for a vessel of the given beam.
///
/// Beam, not length: it is the dimension a top-down marker actually shows, and
/// it separates a 12 m tug from a 60 m tanker without the marker becoming a
/// smear at country zoom. Clamped hard at both ends — an unknown beam must
/// still be clickable, and a supertanker must not swallow the port it is in.
pub fn beam_size(width_m: Option<f64>) -> f32 {
    const DEFAULT: f32 = 7.0;
    const MIN: f32 = 5.0;
    const MAX: f32 = 16.0;
    match width_m {
        Some(w) if w > 0.0 => ((w as f32) * 0.28 + 4.0).clamp(MIN, MAX),
        _ => DEFAULT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_digit_shiptypes_win_over_the_two_digit_ranges() {
        // 7 is MarineTraffic's "cargo"; the two-digit table would call it
        // "unknown", and 8 would fall in no range at all.
        assert_eq!(ais_category(7), "cargo");
        assert_eq!(ais_category(8), "tanker");
        assert_eq!(ais_category(6), "passenger");
    }

    #[test]
    fn the_full_ais_ranges_still_classify() {
        assert_eq!(ais_category(70), "cargo");
        assert_eq!(ais_category(80), "tanker");
        assert_eq!(ais_category(60), "passenger");
        assert_eq!(ais_category(36), "sailing");
        assert_eq!(ais_category(0), "unknown");
    }

    #[test]
    fn every_category_has_a_colour_and_they_are_not_all_the_same() {
        let cargo = ais_color(7);
        let tanker = ais_color(8);
        assert_ne!(cargo, tanker);
        assert!(cargo.starts_with('#') && cargo.len() == 7);
        // An unmapped category still gets something visible.
        assert!(category_color("nonsense").starts_with('#'));
    }

    #[test]
    fn altitude_bands_are_ordered_and_distinct() {
        let ground = altitude_color(Some(500.0));
        let cruise = altitude_color(Some(35_000.0));
        assert_ne!(ground, cruise);
        assert_eq!(altitude_color(None), "#9aa5b4");
        // The boundary belongs to the band above it.
        assert_eq!(altitude_color(Some(999.0)), ground);
        assert_ne!(altitude_color(Some(1_000.0)), ground);
    }

    #[test]
    fn beam_scales_but_stays_inside_its_bounds() {
        let tug = beam_size(Some(8.0));
        let tanker = beam_size(Some(60.0));
        assert!(tanker > tug, "a wider beam must draw larger");
        assert!((5.0..=16.0).contains(&beam_size(Some(500.0))), "clamped high");
        assert!((5.0..=16.0).contains(&beam_size(Some(0.5))), "clamped low");
        // Unknown or nonsense beams still produce a clickable marker.
        assert_eq!(beam_size(None), beam_size(Some(0.0)));
        assert!(beam_size(None) >= 5.0);
    }
}
