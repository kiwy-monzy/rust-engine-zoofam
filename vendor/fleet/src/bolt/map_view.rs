//! Map viewport passed from the UI so Bolt poll/login use the same coordinates as the user sees.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoltMapView {
    pub center_lat: f64,
    pub center_lng: f64,
    pub sw_lat: f64,
    pub sw_lng: f64,
    pub ne_lat: f64,
    pub ne_lng: f64,
}

impl BoltMapView {
    /// Normalized corners (south-west vs north-east).
    pub fn normalized(self) -> Self {
        let sw_lat = self.sw_lat.min(self.ne_lat);
        let ne_lat = self.sw_lat.max(self.ne_lat);
        let sw_lng = self.sw_lng.min(self.ne_lng);
        let ne_lng = self.sw_lng.max(self.ne_lng);
        Self {
            center_lat: self.center_lat,
            center_lng: self.center_lng,
            sw_lat,
            sw_lng,
            ne_lat,
            ne_lng,
        }
    }

    /// Until the map reports a real view, avoid huge bogus queries.
    pub fn is_placeholder(self) -> bool {
        self.center_lat.abs() < 1e-6 && self.center_lng.abs() < 1e-6
    }
}

impl Default for BoltMapView {
    fn default() -> Self {
        Self {
            center_lat: 0.0,
            center_lng: 0.0,
            sw_lat: -0.02,
            sw_lng: -0.02,
            ne_lat: 0.02,
            ne_lng: 0.02,
        }
    }
}
