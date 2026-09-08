//! Fleet live-tile generator — serves MVT/GeoJSON straight from the in-memory
//! poller cache. No DB writes happen on the tile path.

use crate::live::LiveFleet;
use crate::BoltVehicle;
use crate::{grid, load_live};
use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;
use bytes::Bytes;
use maps::TileGenerator;
use serde_json::{json, Value};

/// Bounding box in WGS-84 lon/lat (west, south, east, north) used by tile filters.
const MAX_Z: u32 = 22;
const MIN_Z: u32 = 0;

fn tile_bbox(z: u32, x: u32, y: u32) -> (f64, f64, f64, f64) {
    let n = 2f64.powi(z as i32);
    let west = (x as f64 / n) * 360.0 - 180.0;
    let east = ((x + 1) as f64 / n) * 360.0 - 180.0;
    let north_rad = std::f64::consts::PI * (1.0 - 2.0 * y as f64 / n);
    let south_rad = std::f64::consts::PI * (1.0 - 2.0 * (y + 1) as f64 / n);
    let north = north_rad.sinh().atan().to_degrees();
    let south = south_rad.sinh().atan().to_degrees();
    (west, south, east, north)
}

fn within_tile(v: &BoltVehicle, w: f64, s: f64, e: f64, n: f64) -> bool {
    v.lat >= s && v.lat <= n && v.lng >= w && v.lng <= e
}

fn vehicles_for_tile(snap: &LiveFleet, source: &str, z: u32, x: u32, y: u32) -> Vec<BoltVehicle> {
    if z < MIN_Z || z > MAX_Z {
        return Vec::new();
    }
    let (w, s, e, n) = tile_bbox(z, x, y);
    snap.source(source)
        .iter()
        .filter(|v| within_tile(v, w, s, e, n))
        .cloned()
        .collect()
}

/// Return GeoJSON FeatureCollection for a tile, or `None` if the live cache is empty.
pub fn geojson_for_tile(source: &str, z: u32, x: u32, y: u32) -> Option<Value> {
    let snap = load_live()?;
    let features = vehicles_for_tile(&snap, source, z, x, y);
    let fc = Value::Array(
        features
            .iter()
            .map(|v| {
                let name = v.name.clone().unwrap_or_else(|| v.id.clone());
                let kind = match source {
                    "bolt" => "bolt",
                    "marine" => "marine",
                    "flights" => "flight",
                    _ => "vehicle",
                };
                json!({
                    "type": "Feature",
                    "id": v.id,
                    "geometry": { "type": "Point", "coordinates": [v.lng, v.lat] },
                    "properties": {
                        "id": v.id,
                        "label": name,
                        "name": name,
                        "category": v.category,
                        "sub_category": v.sub_category,
                        "vehicle_type": v.vehicle_type,
                        "icon_url": v.icon_url,
                        "heading": v.heading,
                        "kind": kind,
                        "source": source,
                    }
                })
            })
            .collect(),
    );
    Some(json!({ "type": "FeatureCollection", "features": fc }))
}

/// Build an MVT tile by packaging the in-memory vehicles as base64-encoded GeoJSON
/// `MapFeature` rows and handing them to `maps::TileGenerator`.
pub fn mvt_for_tile(source: &str, z: u32, x: u32, y: u32) -> Result<Bytes> {
    use maps::models::{MapFeature, NewMapFeature};

    let snap = load_live().ok_or_else(|| anyhow!("fleet live cache is empty"))?;
    let vehicles = vehicles_for_tile(&snap, source, z, x, y);
    if vehicles.is_empty() {
        return Ok(Bytes::new());
    }

    // Convert to transient MapFeature rows so we can reuse maps::TileGenerator.
    let now = chrono::Utc::now().to_rfc3339();
    let features: Vec<MapFeature> = vehicles
        .iter()
        .map(|v| {
            let name = v.name.clone().unwrap_or_else(|| v.id.clone());
            let kind = match source {
                "bolt" => "bolt",
                "marine" => "marine",
                "flights" => "flight",
                _ => "vehicle",
            };
            let mut props = serde_json::Map::new();
            props.insert("id".to_string(), json!(v.id));
            props.insert("name".to_string(), json!(name));
            props.insert("label".to_string(), json!(name));
            props.insert("category".to_string(), json!(v.category));
            props.insert("sub_category".to_string(), json!(v.sub_category));
            props.insert("vehicle_type".to_string(), json!(v.vehicle_type));
            props.insert("icon_url".to_string(), json!(v.icon_url));
            props.insert("heading".to_string(), json!(v.heading));
            props.insert("kind".to_string(), json!(kind));
            props.insert("source".to_string(), json!(source));
            let props_str =
                serde_json::to_string(&Value::Object(props)).unwrap_or_else(|_| "{}".to_string());
            let geometry = json!({ "type": "Point", "coordinates": [v.lng, v.lat] });
            let geom_b64 = STANDARD.encode(geometry.to_string().as_bytes());

            // Use the v.id as the synthetic feature_key (MapFeature requires unique ids).
            let new = NewMapFeature {
                layer_id: 0,
                feature_key: format!("{source}:{}", v.id),
                feature_type: kind.to_string(),
                geometry_type: "Point".to_string(),
                geometry: geom_b64.into_bytes(),
                properties: Some(props_str),
                bbox_min_lon: Some(v.lng),
                bbox_min_lat: Some(v.lat),
                bbox_max_lon: Some(v.lng),
                bbox_max_lat: Some(v.lat),
            };
            MapFeature {
                id: 0,
                layer_id: 0,
                feature_key: new.feature_key.clone(),
                feature_type: new.feature_type.clone(),
                geometry_type: new.geometry_type.clone(),
                geometry: new.geometry.clone(),
                properties: new.properties.clone(),
                bbox_min_lon: new.bbox_min_lon,
                bbox_min_lat: new.bbox_min_lat,
                bbox_max_lon: new.bbox_max_lon,
                bbox_max_lat: new.bbox_max_lat,
                created_at: chrono::NaiveDateTime::parse_from_str(&now, "%Y-%m-%dT%H:%M:%S%.f%:z")
                    .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
                updated_at: chrono::NaiveDateTime::parse_from_str(&now, "%Y-%m-%dT%H:%M:%S%.f%:z")
                    .unwrap_or_else(|_| chrono::Utc::now().naive_utc()),
            }
        })
        .collect();

    let gen = TileGenerator::new(source.to_string(), z, x, y);
    gen.generate_tile(&features).context("encoding MVT")
}

/// Aggregate marker set for a `fleet_state`-style summary. Reuses the in-memory
/// cache and the `grid` package for H3 binning.
pub fn live_summary() -> Option<Value> {
    let snap = load_live()?;
    let bolt = snap.bolt.len();
    let marine = snap.marine.len();
    let flights = snap.flights.len();
    let points: Vec<(f64, f64)> = snap
        .bolt
        .iter()
        .chain(snap.marine.iter())
        .chain(snap.flights.iter())
        .map(|v| (v.lat, v.lng))
        .collect();
    let counts = grid::bin_counts(&points, 6);
    let cells = grid::cells_geojson(&counts);
    Some(json!({
        "success": true,
        "updated_at": snap.updated_at,
        "sources": snap.sources,
        "counts": { "bolt": bolt, "marine": marine, "flights": flights, "total": points.len() },
        "cells": cells,
    }))
}
