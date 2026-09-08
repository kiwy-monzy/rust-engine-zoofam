use crate::{Controllers, Result};
use base64::{engine::general_purpose, Engine as _};
use diesel::prelude::*;
use maps::models::*;
use maps::schema::map_features;
use serde_json::Value;

pub struct ImportResult {
    pub imported: usize,
    pub errors: Vec<String>,
}

impl Controllers {
    pub fn import_geojson(
        &self,
        layer_id: i32,
        feature_type: &str,
        geojson_text: &str,
    ) -> Result<ImportResult> {
        import_geojson(self, layer_id, feature_type, geojson_text)
    }
}

/// Compute bounding box from a GeoJSON geometry value.
/// Returns (min_lon, min_lat, max_lon, max_lat).
fn compute_bbox(geometry: &Value) -> Option<(f64, f64, f64, f64)> {
    let mut min_lon = f64::INFINITY;
    let mut min_lat = f64::INFINITY;
    let mut max_lon = f64::NEG_INFINITY;
    let mut max_lat = f64::NEG_INFINITY;

    fn process_coords(
        coords: &Value,
        min_lon: &mut f64,
        min_lat: &mut f64,
        max_lon: &mut f64,
        max_lat: &mut f64,
    ) {
        match coords {
            Value::Array(arr) => {
                if arr.len() >= 2 {
                    if let (Some(lon), Some(lat)) = (arr[0].as_f64(), arr[1].as_f64()) {
                        *min_lon = min_lon.min(lon);
                        *min_lat = min_lat.min(lat);
                        *max_lon = max_lon.max(lon);
                        *max_lat = max_lat.max(lat);
                    }
                }
                for item in arr {
                    process_coords(item, min_lon, min_lat, max_lon, max_lat);
                }
            }
            _ => {}
        }
    }

    let coords = geometry.get("coordinates")?;
    process_coords(
        coords,
        &mut min_lon,
        &mut min_lat,
        &mut max_lon,
        &mut max_lat,
    );

    if min_lon.is_infinite() {
        return None;
    }
    Some((min_lon, min_lat, max_lon, max_lat))
}

pub fn import_geojson(
    controllers: &Controllers,
    layer_id: i32,
    feature_type: &str,
    geojson_text: &str,
) -> Result<ImportResult> {
    let geojson: Value = serde_json::from_str(geojson_text)
        .map_err(|e| crate::Error::Invalid(format!("invalid GeoJSON: {}", e)))?;

    let features = geojson
        .get("features")
        .and_then(|f| f.as_array())
        .ok_or_else(|| crate::Error::Invalid("GeoJSON has no features array".into()))?;

    let mut result = ImportResult {
        imported: 0,
        errors: Vec::new(),
    };

    let mut conn =
        db::conn(&controllers.pool).map_err(|e| crate::Error::Invalid(format!("db pool: {e}")))?;

    const BATCH: usize = 500;
    let mut batch: Vec<NewMapFeature> = Vec::with_capacity(BATCH);

    for (idx, feature) in features.iter().enumerate() {
        let geometry = match feature.get("geometry") {
            Some(g) => g,
            None => {
                result
                    .errors
                    .push(format!("feature {idx}: missing geometry"));
                continue;
            }
        };

        let geometry_type = match geometry.get("type").and_then(|t| t.as_str()) {
            Some(t) => t.to_string(),
            None => {
                result
                    .errors
                    .push(format!("feature {idx}: missing geometry type"));
                continue;
            }
        };

        let geometry_str = match serde_json::to_string(geometry) {
            Ok(s) => s,
            Err(e) => {
                result
                    .errors
                    .push(format!("feature {idx}: geometry ser failed: {e}"));
                continue;
            }
        };
        let geometry_b64 = general_purpose::STANDARD.encode(geometry_str.as_bytes());
        let bbox = compute_bbox(geometry);

        let properties = feature
            .get("properties")
            .cloned()
            .unwrap_or(Value::Object(serde_json::Map::new()));
        let properties_str = serde_json::to_string(&properties)
            .map_err(|e| crate::Error::Invalid(format!("props ser failed: {e}")))?;

        let feature_key = format!("{}-{}", feature_type, idx);

        batch.push(NewMapFeature {
            layer_id,
            feature_key,
            feature_type: feature_type.to_string(),
            geometry_type,
            geometry: geometry_b64.into_bytes(),
            properties: Some(properties_str),
            bbox_min_lon: bbox.map(|b| b.0),
            bbox_min_lat: bbox.map(|b| b.1),
            bbox_max_lon: bbox.map(|b| b.2),
            bbox_max_lat: bbox.map(|b| b.3),
        });

        if batch.len() >= BATCH {
            let count = batch.len();
            diesel::insert_into(map_features::table)
                .values(&batch)
                .execute(&mut conn)
                .map_err(|e| crate::Error::Invalid(format!("insert: {e}")))?;
            result.imported += count;
            batch.clear();
        }
    }

    if !batch.is_empty() {
        let count = batch.len();
        diesel::insert_into(map_features::table)
            .values(&batch)
            .execute(&mut conn)
            .map_err(|e| crate::Error::Invalid(format!("insert: {e}")))?;
        result.imported += count;
        batch.clear();
    }

    Ok(result)
}
