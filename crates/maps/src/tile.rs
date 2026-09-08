use crate::error::{MapError, MapResult};
use crate::models::MapFeature;
use bytes::Bytes;
use mvt::{GeomEncoder, GeomType, Tile};
use pointy::Transform;
use serde_json::Value;
use std::collections::HashMap;

const EXTENT: f64 = 4096.0;

pub struct TileGenerator {
    layer_key: String,
    z: u32,
    x: u32,
    y: u32,
}

impl TileGenerator {
    pub fn new(layer_key: String, z: u32, x: u32, y: u32) -> Self {
        Self { layer_key, z, x, y }
    }

    pub fn generate_tile(&self, features: &[MapFeature]) -> MapResult<Bytes> {
        self.validate_coordinates()?;
        let geojson = self.features_to_geojson(features)?;
        let tile_data = self.geojson_to_mvt(&geojson)?;
        Ok(Bytes::from(tile_data))
    }

    pub fn features_to_geojson(&self, features: &[MapFeature]) -> MapResult<Value> {
        use base64::engine::general_purpose::STANDARD;
        use base64::Engine as _;

        let mut geo_features = Vec::new();

        for feature in features {
            let geometry: Value = STANDARD
                .decode(&feature.geometry)
                .ok()
                .and_then(|bytes| serde_json::from_slice(&bytes).ok())
                .ok_or_else(|| {
                    MapError::InvalidGeometry(format!(
                        "Failed to decode geometry for feature {}",
                        feature.feature_key
                    ))
                })?;

            let properties: Value = feature
                .properties
                .as_ref()
                .and_then(|p| serde_json::from_str(p).ok())
                .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

            let mut props = match properties {
                Value::Object(map) => map,
                _ => serde_json::Map::new(),
            };
            props.insert("id".to_string(), serde_json::json!(feature.id));
            props.insert(
                "feature_key".to_string(),
                serde_json::json!(feature.feature_key),
            );
            props.insert(
                "feature_type".to_string(),
                serde_json::json!(feature.feature_type),
            );
            props.insert("layer_id".to_string(), serde_json::json!(feature.layer_id));

            // Compute a `label` property for text display on the map
            if let Some(road_label) = props.get("RoadLabel").and_then(|v| v.as_str()) {
                let label = road_label
                    .split(" - ")
                    .next()
                    .unwrap_or(road_label)
                    .trim()
                    .to_string();
                props.insert("label".to_string(), serde_json::json!(label));
            } else if let Some(name) = props.get("name").or_else(|| props.get("Name")) {
                if let Some(s) = name.as_str() {
                    props.insert("label".to_string(), serde_json::json!(s));
                }
            } else if let Some(region) = props.get("Region").or_else(|| props.get("region")) {
                if let Some(s) = region.as_str() {
                    props.insert("label".to_string(), serde_json::json!(s));
                }
            }

            let geo_feature = serde_json::json!({
                "type": "Feature",
                "geometry": geometry,
                "properties": Value::Object(props)
            });

            geo_features.push(geo_feature);
        }

        Ok(serde_json::json!({
            "type": "FeatureCollection",
            "features": geo_features
        }))
    }

    fn geojson_to_mvt(&self, geojson: &Value) -> MapResult<Vec<u8>> {
        let (min_lon, min_lat, max_lon, max_lat) = self.tile_bbox();
        let span_x = max_lon - min_lon;
        let span_y = max_lat - min_lat;

        let transform = if span_x.abs() > f64::EPSILON && span_y.abs() > f64::EPSILON {
            Transform::with_translate(-min_lon, -min_lat)
                .scale(EXTENT / span_x, -EXTENT / span_y)
                .translate(0.0, EXTENT)
        } else {
            Transform::default()
        };

        let mut tile = Tile::new(EXTENT as u32);
        let mut layer = tile.create_layer(&self.layer_key);

        if let Some(features) = geojson.get("features").and_then(|f| f.as_array()) {
            for (idx, feat) in features.iter().enumerate() {
                let geometry = match feat.get("geometry") {
                    Some(g) => g,
                    None => continue,
                };
                let geom_type_str = match geometry.get("type").and_then(|t| t.as_str()) {
                    Some(t) => t,
                    None => continue,
                };
                let coords = match geometry.get("coordinates") {
                    Some(c) => c,
                    None => continue,
                };

                let mvt_geom_type = match geom_type_str {
                    "Point" | "MultiPoint" => GeomType::Point,
                    "LineString" | "MultiLineString" => GeomType::Linestring,
                    "Polygon" | "MultiPolygon" => GeomType::Polygon,
                    _ => continue,
                };

                let geom_result = encode_geometry(mvt_geom_type, coords, transform);
                let geom_data = match geom_result {
                    Ok(g) => g,
                    Err(_) => continue,
                };

                let mut mvt_feature = layer.into_feature(geom_data);

                let feature_id = feat
                    .get("id")
                    .or_else(|| feat.get("properties").and_then(|p| p.get("id")))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(idx as u64);
                mvt_feature.set_id(feature_id);

                if let Some(props) = feat.get("properties").and_then(|p| p.as_object()) {
                    for (key, value) in props {
                        if key == "id" || key == "layer_id" {
                            continue;
                        }
                        match value {
                            Value::String(s) => {
                                mvt_feature.add_tag_string(key, s);
                            }
                            Value::Number(n) => {
                                if let Some(i) = n.as_i64() {
                                    mvt_feature.add_tag_int(key, i);
                                } else if let Some(f) = n.as_f64() {
                                    mvt_feature.add_tag_double(key, f);
                                }
                            }
                            Value::Bool(b) => {
                                mvt_feature.add_tag_bool(key, *b);
                            }
                            _ => {}
                        }
                    }
                }

                layer = mvt_feature.into_layer();
            }
        }

        tile.add_layer(layer)
            .map_err(|e| MapError::TileGeneration(format!("Failed to add layer: {}", e)))?;
        tile.to_bytes()
            .map_err(|e| MapError::TileGeneration(format!("MVT encoding failed: {}", e)))
    }

    fn validate_coordinates(&self) -> MapResult<()> {
        let max_z = 22;
        if self.z > max_z {
            return Err(MapError::ZoomOutOfRange {
                z: self.z,
                min: 0,
                max: max_z,
            });
        }

        let max_coord = 2u32.pow(self.z);
        if self.x >= max_coord || self.y >= max_coord {
            return Err(MapError::InvalidTileCoordinates {
                z: self.z,
                x: self.x,
                y: self.y,
            });
        }

        Ok(())
    }

    pub fn tile_bbox(&self) -> (f64, f64, f64, f64) {
        let n = 2f64.powi(self.z as i32);
        let west = (self.x as f64 / n) * 360.0 - 180.0;
        let east = ((self.x + 1) as f64 / n) * 360.0 - 180.0;

        let north_rad = std::f64::consts::PI * (1.0 - 2.0 * self.y as f64 / n);
        let south_rad = std::f64::consts::PI * (1.0 - 2.0 * (self.y + 1) as f64 / n);

        let north = north_rad.sinh().atan().to_degrees();
        let south = south_rad.sinh().atan().to_degrees();

        (west, south, east, north)
    }
}

fn encode_geometry(
    geom_type: GeomType,
    coords: &Value,
    transform: Transform<f64>,
) -> MapResult<mvt::GeomData> {
    let mut encoder = GeomEncoder::new(geom_type).transform(transform);

    match geom_type {
        GeomType::Point => {
            encode_point(&mut encoder, coords)?;
        }
        GeomType::Linestring => {
            if let Some(arr) = coords.as_array() {
                for pt in arr {
                    encode_coord(&mut encoder, pt)?;
                }
            }
        }
        GeomType::Polygon => {
            if let Some(rings) = coords.as_array() {
                for (ri, ring) in rings.iter().enumerate() {
                    if ri > 0 {
                        encoder
                            .complete_geom()
                            .map_err(|e| MapError::TileGeneration(e.to_string()))?;
                    }
                    if let Some(points) = ring.as_array() {
                        for pt in points {
                            encode_coord(&mut encoder, pt)?;
                        }
                    }
                }
            }
        }
    }

    encoder
        .encode()
        .map_err(|e| MapError::TileGeneration(e.to_string()))
}

fn encode_point(encoder: &mut GeomEncoder<f64>, coords: &Value) -> MapResult<()> {
    match coords {
        Value::Array(arr) => {
            if arr.len() >= 2 {
                let lon = arr[0].as_f64().unwrap_or(0.0);
                let lat = arr[1].as_f64().unwrap_or(0.0);
                encoder
                    .add_point(lon, lat)
                    .map_err(|e| MapError::TileGeneration(e.to_string()))?;
            }
        }
        Value::Object(map) => {
            if let (Some(Value::Number(x)), Some(Value::Number(y))) = (map.get("x"), map.get("y")) {
                if let (Some(lon), Some(lat)) = (x.as_f64(), y.as_f64()) {
                    encoder
                        .add_point(lon, lat)
                        .map_err(|e| MapError::TileGeneration(e.to_string()))?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn encode_coord(encoder: &mut GeomEncoder<f64>, pt: &Value) -> MapResult<()> {
    if let Some(arr) = pt.as_array() {
        if arr.len() >= 2 {
            let lon = arr[0].as_f64().unwrap_or(0.0);
            let lat = arr[1].as_f64().unwrap_or(0.0);
            encoder
                .add_point(lon, lat)
                .map_err(|e| MapError::TileGeneration(e.to_string()))?;
        }
    }
    Ok(())
}

pub struct TileCache {
    cache: HashMap<String, CachedTile>,
}

#[derive(Clone)]
struct CachedTile {
    data: Bytes,
    etag: String,
    #[allow(dead_code)]
    created_at: chrono::DateTime<chrono::Utc>,
}

impl TileCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<Bytes> {
        self.cache.get(key).map(|tile| tile.data.clone())
    }

    pub fn get_with_etag(&self, key: &str) -> Option<(Bytes, String)> {
        self.cache
            .get(key)
            .map(|tile| (tile.data.clone(), tile.etag.clone()))
    }

    pub fn put(&mut self, key: String, data: Bytes, etag: String) {
        self.cache.insert(
            key,
            CachedTile {
                data,
                etag,
                created_at: chrono::Utc::now(),
            },
        );
    }

    pub fn invalidate(&mut self, layer_key: &str) {
        self.cache
            .retain(|key, _| !key.starts_with(&format!("{}:", layer_key)));
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for TileCache {
    fn default() -> Self {
        Self::new()
    }
}

pub fn generate_etag(layer_key: &str, z: u32, x: u32, y: u32, version: i64) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    format!("{}:{}:{}:{}:{}", layer_key, z, x, y, version).hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_validation() {
        let generator = TileGenerator::new("test".to_string(), 15, 10000, 10000);
        assert!(generator.validate_coordinates().is_ok());

        let invalid = TileGenerator::new("test".to_string(), 25, 0, 0);
        assert!(invalid.validate_coordinates().is_err());
    }

    #[test]
    fn test_tile_bbox() {
        let generator = TileGenerator::new("test".to_string(), 0, 0, 0);
        let bbox = generator.tile_bbox();
        assert!(bbox.0 < bbox.2);
        assert!(bbox.1 < bbox.3);
    }

    #[test]
    fn test_etag_generation() {
        let etag1 = generate_etag("layer1", 10, 100, 200, 1);
        let etag2 = generate_etag("layer1", 10, 100, 200, 1);
        let etag3 = generate_etag("layer1", 10, 100, 200, 2);

        assert_eq!(etag1, etag2);
        assert_ne!(etag1, etag3);
    }
}
