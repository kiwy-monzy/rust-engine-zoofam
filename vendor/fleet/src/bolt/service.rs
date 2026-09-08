//! Bolt sign-in and vehicle fetch, normalised to the shared model.
//!
//! Stateless: the OTP session produced by [`start`] and [`confirm`] is handed
//! back for the caller to store. This module never writes it anywhere.

use std::collections::BTreeMap;

use crate::error::Result;
use crate::model::{Kind, Vehicle};

use super::types::{Device, Session};
use super::vehicle::{Vehicle as BoltVehicle, get_vehicle_type_opt};
use super::map_view::BoltMapView;

/// A viewport roughly 5 km across, centred on the point — enough to sweep a
/// city district in one call.
pub fn view(lat: f64, lng: f64) -> BoltMapView {
    let d = 0.025; // ~2.7 km at the equator; Bolt widens it further if needed.
    BoltMapView {
        center_lat: lat,
        center_lng: lng,
        sw_lat: lat - d,
        sw_lng: lng - d,
        ne_lat: lat + d,
        ne_lng: lng + d,
    }
}

/// Map a Bolt vehicle onto the shared model.
///
/// The icon URL is deliberately *not* filled in from the built-in table.
/// `icon_id` is zone-specific: the compiled-in names cover the handful of ids
/// seen in one city, and anywhere else every vehicle came back "Unknown" with
/// no artwork. The URLs travel with each poll instead — see [`fetch`] — and the
/// id survives as a last-resort label so an unnamed type is still identifiable.
pub fn to_vehicle(v: &BoltVehicle) -> Vehicle {
    let named = get_vehicle_type_opt(&v.icon_id).map(str::to_string);
    let label = named
        .or_else(|| (!v.category.is_empty()).then(|| v.category.clone()))
        .unwrap_or_else(|| format!("type {}", v.icon_id));
    Vehicle {
        id: v.id.clone(),
        source_id: "bolt".to_string(),
        kind: Kind::Taxi,
        name: Some(label.clone()),
        sub_category: Some(v.category.clone()),
        lat: v.lat,
        lng: v.lng,
        heading: Some(v.bearing),
        vehicle_type: Some(label),
        icon_url: None,
        raw: serde_json::to_value(v).unwrap_or_else(|_| serde_json::json!({})),
        fetched_at: crate::now_ms(),
    }
}

/// Begin phone-OTP sign-in. Returns the pending session for the caller to keep.
pub async fn start(phone: &str, device: &Device, channel: &str) -> Result<Session> {
    let client = super::client::create_client()?;
    super::client::start_verification(&client, phone, device, channel)
        .await
        .map_err(Into::into)
}

/// Confirm the OTP against the pending session, returning the authenticated one.
///
/// `device` must be the same one [`start`] was given: Bolt ties the pending
/// session to it, and a mismatch is rejected as a different handset.
pub async fn confirm(pending: &Session, otp: &str, device: &Device) -> Result<Session> {
    let client = super::client::create_client()?;
    super::client::confirm_verification(&client, pending, otp, device)
        .await
        .map_err(Into::into)
}

/// Nearby vehicles, together with the `icon id → URL` map the same poll
/// advertised.
///
/// The two come back together because they only make sense together: the ids
/// are meaningful within one response and one zone.
pub async fn fetch(
    session: &Session,
    lat: f64,
    lng: f64,
) -> Result<(Vec<BoltVehicle>, BTreeMap<String, String>)> {
    super::client::fetch_vehicles_and_icons(session, &view(lat, lng))
        .await
        .map_err(Into::into)
}

/// Fill in each vehicle's artwork from the poll's own dictionary.
///
/// Separate from [`to_vehicle`] so the mapping stays a pure function of one
/// record, and because a caller that resolves icons to inline images wants to
/// do it once per poll rather than once per vehicle.
pub fn attach_icons(vehicles: &mut [Vehicle], raw: &[BoltVehicle], icons: &BTreeMap<String, String>) {
    for (v, r) in vehicles.iter_mut().zip(raw) {
        v.icon_url = icons.get(&r.icon_id).cloned();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(icon_id: &str, category: &str) -> BoltVehicle {
        BoltVehicle {
            id: "v1".into(),
            lat: -6.83,
            lng: 39.30,
            bearing: 90.0,
            category: category.into(),
            icon_id: icon_id.into(),
        }
    }

    /// The zone bug: an id with no compiled-in name must still produce a label
    /// and must still be able to receive artwork.
    #[test]
    fn an_unknown_icon_id_still_labels_and_still_takes_a_url() {
        let r = raw("9999", "");
        let mut v = vec![to_vehicle(&r)];
        assert_eq!(v[0].name.as_deref(), Some("type 9999"));
        assert_eq!(v[0].icon_url, None, "no artwork until the poll supplies it");

        let icons = BTreeMap::from([("9999".to_string(), "https://cdn/x.png".to_string())]);
        attach_icons(&mut v, &[r], &icons);
        assert_eq!(v[0].icon_url.as_deref(), Some("https://cdn/x.png"));
    }

    #[test]
    fn the_category_labels_a_vehicle_when_the_id_is_unnamed() {
        let v = to_vehicle(&raw("9999", "Bolt XL"));
        assert_eq!(v.name.as_deref(), Some("Bolt XL"));
        assert_eq!(v.sub_category.as_deref(), Some("Bolt XL"));
    }

    #[test]
    fn the_viewport_brackets_the_point() {
        let v = view(-6.83, 39.30);
        assert!(v.sw_lat < v.center_lat && v.center_lat < v.ne_lat);
        assert!(v.sw_lng < v.center_lng && v.center_lng < v.ne_lng);
    }
}
