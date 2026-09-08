use crate::{Controllers, Error, Result};
use maps::{models::*, MapError, MapService};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSourceRequest {
    pub name: String,
    pub source_type: String,
    pub url: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct UpdateSourceRequest {
    pub name: Option<String>,
    pub source_type: Option<String>,
    pub url: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateLayerRequest {
    pub source_id: i32,
    pub name: String,
    pub layer_key: String,
    pub layer_type: String,
    pub description: Option<String>,
    #[serde(default)]
    pub min_zoom: i32,
    #[serde(default = "default_max_zoom")]
    pub max_zoom: i32,
    #[serde(default)]
    pub z_index: i32,
    #[serde(default = "default_true")]
    pub visible: bool,
    pub style_id: Option<i32>,
}

fn default_max_zoom() -> i32 {
    22
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateLayerRequest {
    pub name: Option<String>,
    pub layer_key: Option<String>,
    pub layer_type: Option<String>,
    pub description: Option<String>,
    pub min_zoom: Option<i32>,
    pub max_zoom: Option<i32>,
    pub z_index: Option<i32>,
    pub visible: Option<bool>,
    pub style_id: Option<Option<i32>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateFeatureRequest {
    pub layer_id: i32,
    pub feature_key: String,
    pub feature_type: String,
    pub geometry_type: String,
    pub geometry: String, // Base64 encoded
    pub properties: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateFeatureRequest {
    pub feature_key: Option<String>,
    pub feature_type: Option<String>,
    pub geometry_type: Option<String>,
    pub geometry: Option<String>, // Base64 encoded
    pub properties: Option<Option<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateStyleRequest {
    pub layer_id: Option<i32>,
    pub name: String,
    pub style_type: String,
    pub definition: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateStyleRequest {
    pub layer_id: Option<Option<i32>>,
    pub name: Option<String>,
    pub style_type: Option<String>,
    pub definition: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTileRequest {
    pub layer_key: String,
    pub z: i32,
    pub x: i32,
    pub y: i32,
    pub data: String, // Base64 encoded
    pub etag: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTileRequest {
    pub data: Option<String>, // Base64 encoded
    pub etag: Option<Option<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTileFeatureRequest {
    pub feature_id: i32,
    pub z: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TileAdminSummary {
    pub tiles: usize,
    pub tile_features: i64,
}

// ============================================================================
// CONTROLLER IMPLEMENTATION
// ============================================================================

impl Controllers {
    fn get_map_service(&self) -> MapService {
        let conn = self.pool.get().expect("Failed to get DB connection");
        MapService::new(conn)
    }

    // ========================================================================
    // SOURCE CONTROLLERS
    // ========================================================================

    pub fn create_source(&self, req: CreateSourceRequest) -> Result<MapSource> {
        let mut service = self.get_map_service();
        let new_source = NewMapSource {
            name: req.name,
            source_type: req.source_type,
            url: req.url,
            version: req.version,
            description: req.description,
        };
        service.create_source(new_source).map_err(map_error)
    }

    pub fn get_source(&self, id: i32) -> Result<MapSource> {
        let mut service = self.get_map_service();
        service.get_source(id).map_err(map_error)
    }

    pub fn get_source_by_name(&self, name: &str) -> Result<MapSource> {
        let mut service = self.get_map_service();
        service.get_source_by_name(name).map_err(map_error)
    }

    pub fn list_sources(&self) -> Result<Vec<MapSource>> {
        let mut service = self.get_map_service();
        service.list_sources().map_err(map_error)
    }

    pub fn update_source(&self, id: i32, req: UpdateSourceRequest) -> Result<MapSource> {
        let mut service = self.get_map_service();
        let update = UpdateMapSource {
            name: req.name,
            source_type: req.source_type,
            url: req.url,
            version: req.version,
            description: req.description,
            updated_at: chrono::Utc::now().naive_utc(),
        };
        service.update_source(id, update).map_err(map_error)
    }

    pub fn delete_source(&self, id: i32) -> Result<()> {
        let mut service = self.get_map_service();
        service.delete_source(id).map_err(map_error)
    }

    // ========================================================================
    // LAYER CONTROLLERS
    // ========================================================================

    pub fn create_layer(&self, req: CreateLayerRequest) -> Result<MapLayer> {
        let mut service = self.get_map_service();
        let new_layer = NewMapLayer {
            source_id: req.source_id,
            name: req.name,
            layer_key: req.layer_key,
            layer_type: req.layer_type,
            description: req.description,
            min_zoom: req.min_zoom,
            max_zoom: req.max_zoom,
            z_index: req.z_index,
            visible: req.visible,
            style_id: req.style_id,
        };
        service.create_layer(new_layer).map_err(map_error)
    }

    pub fn get_layer(&self, id: i32) -> Result<MapLayer> {
        let mut service = self.get_map_service();
        service.get_layer(id).map_err(map_error)
    }

    pub fn get_layer_by_key(&self, layer_key: &str) -> Result<MapLayer> {
        let mut service = self.get_map_service();
        service.get_layer_by_key(layer_key).map_err(map_error)
    }

    pub fn list_layers(&self) -> Result<Vec<MapLayer>> {
        let mut service = self.get_map_service();
        service.list_layers().map_err(map_error)
    }

    pub fn list_visible_layers(&self) -> Result<Vec<MapLayer>> {
        let mut service = self.get_map_service();
        service.list_visible_layers().map_err(map_error)
    }

    pub fn list_layers_by_source(&self, source_id: i32) -> Result<Vec<MapLayer>> {
        let mut service = self.get_map_service();
        service.list_layers_by_source(source_id).map_err(map_error)
    }

    pub fn update_layer(&self, id: i32, req: UpdateLayerRequest) -> Result<MapLayer> {
        let mut service = self.get_map_service();
        let update = UpdateMapLayer {
            name: req.name,
            layer_key: req.layer_key,
            layer_type: req.layer_type,
            description: req.description,
            min_zoom: req.min_zoom,
            max_zoom: req.max_zoom,
            z_index: req.z_index,
            visible: req.visible,
            style_id: req.style_id,
            updated_at: chrono::Utc::now().naive_utc(),
        };
        service.update_layer(id, update).map_err(map_error)
    }

    pub fn delete_layer(&self, id: i32) -> Result<()> {
        let mut service = self.get_map_service();
        service.delete_layer(id).map_err(map_error)
    }

    pub fn toggle_layer_visibility(&self, id: i32) -> Result<MapLayer> {
        let mut service = self.get_map_service();
        service.toggle_layer_visibility(id).map_err(map_error)
    }

    // ========================================================================
    // FEATURE CONTROLLERS
    // ========================================================================

    pub fn create_feature(&self, req: CreateFeatureRequest) -> Result<MapFeature> {
        let mut service = self.get_map_service();
        let geometry = decode_base64(&req.geometry)?;
        let new_feature = NewMapFeature {
            layer_id: req.layer_id,
            feature_key: req.feature_key,
            feature_type: req.feature_type,
            geometry_type: req.geometry_type,
            geometry,
            properties: req.properties,
            bbox_min_lon: None,
            bbox_min_lat: None,
            bbox_max_lon: None,
            bbox_max_lat: None,
        };
        service.create_feature(new_feature).map_err(map_error)
    }

    pub fn get_feature(&self, id: i32) -> Result<MapFeature> {
        let mut service = self.get_map_service();
        service.get_feature(id).map_err(map_error)
    }

    pub fn list_features(&self) -> Result<Vec<MapFeature>> {
        let mut service = self.get_map_service();
        service.list_features().map_err(map_error)
    }

    pub fn count_features(&self) -> Result<i64> {
        let mut service = self.get_map_service();
        service.count_features().map_err(map_error)
    }

    pub fn list_features_by_layer(&self, layer_id: i32) -> Result<Vec<MapFeature>> {
        let mut service = self.get_map_service();
        service.list_features_by_layer(layer_id).map_err(map_error)
    }

    pub fn list_features_by_layer_in_bbox(
        &self,
        layer_id: i32,
        west: f64,
        south: f64,
        east: f64,
        north: f64,
    ) -> Result<Vec<MapFeature>> {
        let mut service = self.get_map_service();
        service
            .list_features_by_layer_in_bbox(layer_id, west, south, east, north)
            .map_err(map_error)
    }

    pub fn update_feature(&self, id: i32, req: UpdateFeatureRequest) -> Result<MapFeature> {
        let mut service = self.get_map_service();
        let geometry = req
            .geometry
            .as_ref()
            .map(|g| decode_base64(g))
            .transpose()?;
        let update = UpdateMapFeature {
            feature_key: req.feature_key,
            feature_type: req.feature_type,
            geometry_type: req.geometry_type,
            geometry,
            properties: req.properties,
            updated_at: chrono::Utc::now().naive_utc(),
        };
        service.update_feature(id, update).map_err(map_error)
    }

    pub fn delete_feature(&self, id: i32) -> Result<()> {
        let mut service = self.get_map_service();
        service.delete_feature(id).map_err(map_error)
    }

    pub fn count_features_by_layer(&self, layer_id: i32) -> Result<i64> {
        let mut service = self.get_map_service();
        service.count_features_by_layer(layer_id).map_err(map_error)
    }

    // ========================================================================
    // STYLE CONTROLLERS
    // ========================================================================

    pub fn create_style(&self, req: CreateStyleRequest) -> Result<MapStyle> {
        let mut service = self.get_map_service();
        let new_style = NewMapStyle {
            layer_id: req.layer_id,
            name: req.name,
            style_type: req.style_type,
            definition: req.definition,
        };
        service.create_style(new_style).map_err(map_error)
    }

    pub fn get_style(&self, id: i32) -> Result<MapStyle> {
        let mut service = self.get_map_service();
        service.get_style(id).map_err(map_error)
    }

    pub fn list_styles(&self) -> Result<Vec<MapStyle>> {
        let mut service = self.get_map_service();
        service.list_styles().map_err(map_error)
    }

    pub fn list_styles_by_layer(&self, layer_id: i32) -> Result<Vec<MapStyle>> {
        let mut service = self.get_map_service();
        service.list_styles_by_layer(layer_id).map_err(map_error)
    }

    pub fn update_style(&self, id: i32, req: UpdateStyleRequest) -> Result<MapStyle> {
        let mut service = self.get_map_service();
        let update = UpdateMapStyle {
            layer_id: req.layer_id,
            name: req.name,
            style_type: req.style_type,
            definition: req.definition,
            updated_at: chrono::Utc::now().naive_utc(),
        };
        service.update_style(id, update).map_err(map_error)
    }

    pub fn delete_style(&self, id: i32) -> Result<()> {
        let mut service = self.get_map_service();
        service.delete_style(id).map_err(map_error)
    }

    // ========================================================================
    // TILE CONTROLLERS
    // ========================================================================

    pub fn create_tile(&self, req: CreateTileRequest) -> Result<MapTile> {
        let mut service = self.get_map_service();
        let data = decode_base64(&req.data)?;
        let new_tile = NewMapTile {
            layer_key: req.layer_key,
            z: req.z,
            x: req.x,
            y: req.y,
            data,
            etag: req.etag,
        };
        service.create_tile(new_tile).map_err(map_error)
    }

    pub fn list_tiles(&self, layer_key: Option<&str>) -> Result<Vec<MapTile>> {
        let mut service = self.get_map_service();
        service.list_tiles(layer_key).map_err(map_error)
    }

    pub fn count_tiles(&self) -> Result<i64> {
        let mut service = self.get_map_service();
        service.count_tiles().map_err(map_error)
    }

    pub fn get_tile(&self, layer_key: &str, z: i32, x: i32, y: i32) -> Result<MapTile> {
        let mut service = self.get_map_service();
        service.get_tile(layer_key, z, x, y).map_err(map_error)
    }

    pub fn update_tile(
        &self,
        layer_key: &str,
        z: i32,
        x: i32,
        y: i32,
        req: UpdateTileRequest,
    ) -> Result<MapTile> {
        let mut service = self.get_map_service();
        let data = req.data.as_ref().map(|d| decode_base64(d)).transpose()?;
        let update = UpdateMapTile {
            data,
            etag: req.etag,
            updated_at: chrono::Utc::now().naive_utc(),
        };
        service
            .update_tile(layer_key, z, x, y, update)
            .map_err(map_error)
    }

    pub fn delete_tile(&self, layer_key: &str, z: i32, x: i32, y: i32) -> Result<()> {
        let mut service = self.get_map_service();
        service.delete_tile(layer_key, z, x, y).map_err(map_error)
    }

    pub fn clear_tiles_for_layer(&self, layer_key: &str) -> Result<usize> {
        let mut service = self.get_map_service();
        service.clear_tiles_for_layer(layer_key).map_err(map_error)
    }

    // ========================================================================
    // TILE-FEATURE INDEX CONTROLLERS
    // ========================================================================

    pub fn list_tile_features(&self) -> Result<Vec<MapTileFeature>> {
        let mut service = self.get_map_service();
        service.list_tile_features().map_err(map_error)
    }

    pub fn list_tile_features_for_tile(
        &self,
        z: i32,
        x: i32,
        y: i32,
    ) -> Result<Vec<MapTileFeature>> {
        let mut service = self.get_map_service();
        service
            .list_tile_features_for_tile(z, x, y)
            .map_err(map_error)
    }

    pub fn list_tile_features_by_feature(&self, feature_id: i32) -> Result<Vec<MapTileFeature>> {
        let mut service = self.get_map_service();
        service
            .list_tile_features_by_feature(feature_id)
            .map_err(map_error)
    }

    pub fn create_tile_feature(&self, req: CreateTileFeatureRequest) -> Result<MapTileFeature> {
        let mut service = self.get_map_service();
        service
            .index_tile_feature(NewMapTileFeature {
                feature_id: req.feature_id,
                z: req.z,
                x: req.x,
                y: req.y,
            })
            .map_err(map_error)
    }

    pub fn delete_tile_feature(&self, id: i32) -> Result<()> {
        let mut service = self.get_map_service();
        service.delete_tile_feature(id).map_err(map_error)
    }

    pub fn clear_tile_features(&self) -> Result<usize> {
        let mut service = self.get_map_service();
        service.clear_tile_features().map_err(map_error)
    }

    pub fn count_tile_features(&self) -> Result<i64> {
        let mut service = self.get_map_service();
        service.count_tile_features().map_err(map_error)
    }

    // ========================================================================
    // COMPOSITE QUERIES
    // ========================================================================

    pub fn get_layer_with_feature_count(&self, id: i32) -> Result<LayerWithFeatures> {
        let mut service = self.get_map_service();
        service.get_layer_with_feature_count(id).map_err(map_error)
    }

    pub fn get_source_with_layer_count(&self, id: i32) -> Result<SourceWithLayers> {
        let mut service = self.get_map_service();
        service.get_source_with_layer_count(id).map_err(map_error)
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn map_error(err: MapError) -> Error {
    match err {
        MapError::Database(msg) => Error::Invalid(msg),
        MapError::LayerNotFound(_id) => Error::NotFound("layer"),
        MapError::SourceNotFound(_id) => Error::NotFound("source"),
        MapError::FeatureNotFound(_id) => Error::NotFound("feature"),
        MapError::StyleNotFound(_id) => Error::NotFound("style"),
        MapError::InvalidGeometry(msg) => Error::Invalid(msg),
        MapError::InvalidTileCoordinates { z, x, y } => {
            Error::Invalid(format!("Invalid tile: {}/{}/{}", z, x, y))
        }
        MapError::TileGeneration(msg) => Error::Invalid(msg),
        MapError::ZoomOutOfRange { z, min, max } => {
            Error::Invalid(format!("Zoom {z} out of range [{min}, {max}]"))
        }
        MapError::Serialization(msg) => Error::Invalid(msg),
        MapError::Deserialization(msg) => Error::Invalid(msg),
        MapError::Unauthorized(msg) => Error::Forbidden(msg),
        MapError::Configuration(msg) => Error::Invalid(msg),
    }
}

fn decode_base64(data: &str) -> Result<Vec<u8>> {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD
        .decode(data)
        .map_err(|e| Error::Invalid(format!("Invalid base64: {}", e)))
}

/// Compute bounding box from a GeoJSON geometry value.
/// Returns (min_lon, min_lat, max_lon, max_lat).
fn compute_bbox_from_geometry(geometry: &serde_json::Value) -> Option<(f64, f64, f64, f64)> {
    use serde_json::Value;

    let mut min_lon = f64::INFINITY;
    let mut min_lat = f64::INFINITY;
    let mut max_lon = f64::NEG_INFINITY;
    let mut max_lat = f64::NEG_INFINITY;

    fn walk(
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
                    walk(item, min_lon, min_lat, max_lon, max_lat);
                }
            }
            _ => {}
        }
    }

    let coords = geometry.get("coordinates")?;
    walk(
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

impl Controllers {
    /// Backfill bbox for all features that don't have one yet.
    /// Returns the number of features updated.
    pub fn backfill_feature_bboxes(&self) -> Result<usize> {
        use base64::{engine::general_purpose, Engine as _};
        use diesel::prelude::*;
        use maps::schema::map_features::dsl;

        let mut conn = db::conn(&self.pool).map_err(|e| Error::Invalid(format!("db pool: {e}")))?;

        // Fetch all features missing bbox_min_lon
        let features: Vec<(i32, Vec<u8>)> = dsl::map_features
            .filter(dsl::bbox_min_lon.is_null())
            .select((dsl::id, dsl::geometry))
            .load(&mut conn)
            .map_err(|e| Error::Invalid(format!("query: {e}")))?;

        if features.is_empty() {
            return Ok(0);
        }

        let mut updated = 0;
        for (id, geometry_b64) in features {
            let geo_bytes = match general_purpose::STANDARD.decode(&geometry_b64) {
                Ok(b) => b,
                Err(_) => continue,
            };
            let geo: serde_json::Value = match serde_json::from_slice(&geo_bytes) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let bbox = compute_bbox_from_geometry(&geo);

            diesel::update(dsl::map_features.find(id))
                .set((
                    dsl::bbox_min_lon.eq(bbox.as_ref().map(|b| b.0)),
                    dsl::bbox_min_lat.eq(bbox.as_ref().map(|b| b.1)),
                    dsl::bbox_max_lon.eq(bbox.as_ref().map(|b| b.2)),
                    dsl::bbox_max_lat.eq(bbox.as_ref().map(|b| b.3)),
                ))
                .execute(&mut conn)
                .ok();
            updated += 1;
        }

        Ok(updated)
    }
}
