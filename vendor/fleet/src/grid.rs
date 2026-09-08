//! H3 hexagonal grid helpers, over the vendored pure-Rust `h3o`.
//!
//! H3 is a geodesic hex grid addressed in lat/lng — **not** a Mercator grid. To
//! draw a cell on the Web-Mercator map the client just takes the cell's
//! lat/lng boundary (below) and lets Galileo project it, exactly as it does for
//! any other WGS84 polygon. We never enumerate the whole planet: fleet events
//! are binned into the cells they fall in, and only those cells are drawn.

use std::collections::HashMap;

use h3o::{CellIndex, LatLng, Resolution};

/// Clamp a requested resolution (0–15) to a valid `h3o::Resolution`.
pub fn resolution(res: u8) -> Resolution {
    Resolution::try_from(res.min(15)).unwrap_or(Resolution::Six)
}

/// The H3 cell (as its canonical hex id) that contains a point, or `None` if
/// the coordinates are out of range.
pub fn cell_for(lat: f64, lng: f64, res: u8) -> Option<String> {
    let ll = LatLng::new(lat, lng).ok()?;
    Some(ll.to_cell(resolution(res)).to_string())
}

/// A cell's boundary as `[lng, lat]` pairs (GeoJSON ring order), closed.
pub fn cell_boundary(cell: &str) -> Option<Vec<[f64; 2]>> {
    let idx: CellIndex = cell.parse().ok()?;
    let mut ring: Vec<[f64; 2]> = idx.boundary().iter().map(|v| [v.lng(), v.lat()]).collect();
    if let Some(first) = ring.first().copied() {
        ring.push(first); // close the ring
    }
    Some(ring)
}

/// Bin points into cells, counting how many fall in each.
pub fn bin_counts(points: &[(f64, f64)], res: u8) -> HashMap<String, usize> {
    let r = resolution(res);
    let mut counts: HashMap<String, usize> = HashMap::new();
    for &(lat, lng) in points {
        if let Ok(ll) = LatLng::new(lat, lng) {
            *counts.entry(ll.to_cell(r).to_string()).or_insert(0) += 1;
        }
    }
    counts
}

/// A GeoJSON `FeatureCollection` of the binned cells, each polygon carrying its
/// `h3` id and event `count`. This is what the map client draws as the fleet
/// density grid over the DEM.
pub fn cells_geojson(counts: &HashMap<String, usize>) -> serde_json::Value {
    let features: Vec<serde_json::Value> = counts
        .iter()
        .filter_map(|(cell, &count)| {
            let ring = cell_boundary(cell)?;
            Some(serde_json::json!({
                "type": "Feature",
                "properties": { "h3": cell, "count": count },
                "geometry": { "type": "Polygon", "coordinates": [ring] }
            }))
        })
        .collect();

    serde_json::json!({ "type": "FeatureCollection", "features": features })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_point_bins_to_a_cell_and_back_to_a_ring() {
        // Dar es Salaam.
        let cell = cell_for(-6.83, 39.30, 6).expect("valid point");
        let ring = cell_boundary(&cell).expect("valid cell");
        // A hexagon boundary is 6 vertices + the closing point (7 in some
        // pentagon/edge cases, but never fewer than 4).
        assert!(ring.len() >= 4);
        assert_eq!(ring.first(), ring.last(), "ring is closed");
    }

    #[test]
    fn binning_counts_points_per_cell() {
        let pts = [(-6.83, 39.30), (-6.83, 39.30), (-6.90, 39.10)];
        let counts = bin_counts(&pts, 5);
        let total: usize = counts.values().sum();
        assert_eq!(total, 3);
        let gj = cells_geojson(&counts);
        assert_eq!(gj["type"], "FeatureCollection");
    }
}

/// Bin vehicles into H3 cells.
///
/// The counting form above answers "how busy is this hex"; this one keeps the
/// vehicles themselves, which is what a client needs to draw markers per cell
/// without re-binning them itself.
#[cfg(feature = "net")]
pub fn bin_vehicles(vehicles: &[crate::model::Vehicle], res: u8) -> crate::wire::Grid {
    use std::collections::BTreeMap;

    let r = resolution(res);
    let mut cells: BTreeMap<String, Vec<crate::wire::Vehicle>> = BTreeMap::new();
    let mut newest = 0_i64;

    for v in vehicles {
        // A vehicle at Null Island or with a truncated position would otherwise
        // pile into one cell in the Gulf of Guinea and read as a fleet.
        if !v.has_position() {
            continue;
        }
        let Ok(ll) = LatLng::new(v.lat, v.lng) else {
            continue;
        };
        newest = newest.max(v.fetched_at);
        cells
            .entry(ll.to_cell(r).to_string())
            .or_default()
            .push(crate::wire::Vehicle::from(v));
    }

    crate::wire::Grid {
        cells: cells
            .into_iter()
            .map(|(h3, vehicles)| crate::wire::Cell {
                h3,
                resolution: u8::from(r) as u32,
                vehicles,
            })
            .collect(),
        resolution: u8::from(r) as u32,
        fetched_at: newest,
    }
}

#[cfg(all(test, feature = "net"))]
mod grid_tests {
    use super::*;
    use crate::model::{Kind, Vehicle};

    fn at(lat: f64, lng: f64) -> Vehicle {
        Vehicle {
            id: "1".into(),
            source_id: "test".into(),
            kind: Kind::Vessel,
            name: None,
            sub_category: None,
            lat,
            lng,
            heading: None,
            vehicle_type: None,
            icon_url: None,
            raw: serde_json::Value::Null,
            fetched_at: 1_756_000_000_000,
        }
    }

    #[test]
    fn vehicles_in_the_same_hex_share_a_cell() {
        let grid = bin_vehicles(&[at(-6.83, 39.30), at(-6.831, 39.301)], 6);
        assert_eq!(grid.cells.len(), 1, "neighbours a few metres apart");
        assert_eq!(grid.cells[0].vehicles.len(), 2);
        assert_eq!(grid.resolution, 6);
    }

    /// The bug this guards: an unplaced vehicle binned at (0,0) draws a phantom
    /// cluster in the Atlantic.
    #[test]
    fn unplaced_vehicles_are_not_binned() {
        let grid = bin_vehicles(&[at(0.0, 0.0), at(-6.83, 39.30)], 6);
        assert_eq!(grid.cells.len(), 1);
        assert_eq!(grid.cells[0].vehicles.len(), 1);
    }
}
