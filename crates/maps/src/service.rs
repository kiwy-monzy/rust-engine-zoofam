use crate::error::{MapError, MapResult};
use crate::models::*;
use crate::schema::*;
use crate::tile::TileCache;
use chrono::Utc;
use db::DbConn;
use diesel::prelude::*;
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

/// Professional service layer for map operations
pub struct MapService {
    conn: DbConn,
    tile_cache: Arc<Mutex<TileCache>>,
}

impl MapService {
    pub fn new(conn: DbConn) -> Self {
        Self {
            conn,
            tile_cache: Arc::new(Mutex::new(TileCache::new())),
        }
    }

    // ========================================================================
    // SOURCE OPERATIONS
    // ========================================================================

    pub fn create_source(&mut self, source: NewMapSource) -> MapResult<MapSource> {
        use map_sources::dsl;

        diesel::insert_into(dsl::map_sources)
            .values(&source)
            .execute(&mut self.conn)?;

        let created: MapSource = dsl::map_sources
            .order(dsl::id.desc())
            .first(&mut self.conn)?;

        info!("Created map source: {}", created.name);
        Ok(created)
    }

    pub fn get_source(&mut self, id: i32) -> MapResult<MapSource> {
        use map_sources::dsl;

        dsl::map_sources
            .find(id)
            .first(&mut self.conn)
            .map_err(|_| MapError::SourceNotFound(id.to_string()))
    }

    pub fn get_source_by_name(&mut self, name: &str) -> MapResult<MapSource> {
        use map_sources::dsl;

        dsl::map_sources
            .filter(dsl::name.eq(name))
            .first(&mut self.conn)
            .map_err(|_| MapError::SourceNotFound(name.to_string()))
    }

    pub fn list_sources(&mut self) -> MapResult<Vec<MapSource>> {
        use map_sources::dsl;

        let sources = dsl::map_sources
            .order(dsl::name.asc())
            .load(&mut self.conn)?;

        Ok(sources)
    }

    pub fn update_source(&mut self, id: i32, update: UpdateMapSource) -> MapResult<MapSource> {
        use map_sources::dsl;

        diesel::update(dsl::map_sources.find(id))
            .set(&update)
            .execute(&mut self.conn)?;

        self.get_source(id)
    }

    pub fn delete_source(&mut self, id: i32) -> MapResult<()> {
        use map_layers::dsl as ldsl;
        use map_sources::dsl;

        // Get the layer keys for this source's layers before deleting (to clean up tiles)
        let layer_keys: Vec<String> = ldsl::map_layers
            .filter(ldsl::source_id.eq(id))
            .select(ldsl::layer_key)
            .load(&mut self.conn)?;

        diesel::delete(dsl::map_sources.find(id)).execute(&mut self.conn)?;

        // Clean up cached tiles for each deleted layer
        for key in &layer_keys {
            self.tile_cache.lock().unwrap().invalidate(key);
        }

        info!(
            "Deleted map source {} ({} layers cleaned)",
            id,
            layer_keys.len()
        );
        Ok(())
    }

    // ========================================================================
    // LAYER OPERATIONS
    // ========================================================================

    pub fn create_layer(&mut self, layer: NewMapLayer) -> MapResult<MapLayer> {
        use map_layers::dsl;

        // Validate source exists
        self.get_source(layer.source_id)?;

        diesel::insert_into(dsl::map_layers)
            .values(&layer)
            .execute(&mut self.conn)?;

        let created = dsl::map_layers
            .order(dsl::id.desc())
            .first(&mut self.conn)?;

        info!("Created map layer: {}", layer.name);
        Ok(created)
    }

    pub fn get_layer(&mut self, id: i32) -> MapResult<MapLayer> {
        use map_layers::dsl;

        dsl::map_layers
            .find(id)
            .first(&mut self.conn)
            .map_err(|_| MapError::LayerNotFound(id.to_string()))
    }

    pub fn get_layer_by_key(&mut self, layer_key: &str) -> MapResult<MapLayer> {
        use map_layers::dsl;

        dsl::map_layers
            .filter(dsl::layer_key.eq(layer_key))
            .first(&mut self.conn)
            .map_err(|_| MapError::LayerNotFound(layer_key.to_string()))
    }

    pub fn list_layers(&mut self) -> MapResult<Vec<MapLayer>> {
        use map_layers::dsl;

        let layers = dsl::map_layers
            .order(dsl::z_index.asc())
            .then_order_by(dsl::name.asc())
            .load(&mut self.conn)?;

        Ok(layers)
    }

    pub fn list_visible_layers(&mut self) -> MapResult<Vec<MapLayer>> {
        use map_layers::dsl;

        let layers = dsl::map_layers
            .filter(dsl::visible.eq(true))
            .order(dsl::z_index.asc())
            .then_order_by(dsl::name.asc())
            .load(&mut self.conn)?;

        Ok(layers)
    }

    pub fn list_layers_by_source(&mut self, source_id: i32) -> MapResult<Vec<MapLayer>> {
        use map_layers::dsl;

        let layers = dsl::map_layers
            .filter(dsl::source_id.eq(source_id))
            .order(dsl::z_index.asc())
            .load(&mut self.conn)?;

        Ok(layers)
    }

    pub fn update_layer(&mut self, id: i32, update: UpdateMapLayer) -> MapResult<MapLayer> {
        use map_layers::dsl;

        diesel::update(dsl::map_layers.find(id))
            .set(&update)
            .execute(&mut self.conn)?;

        self.get_layer(id)
    }

    pub fn delete_layer(&mut self, id: i32) -> MapResult<()> {
        use map_layers::dsl;

        // Get layer_key before deleting (to clean up tiles)
        let layer_key: Option<String> = dsl::map_layers
            .filter(dsl::id.eq(id))
            .select(dsl::layer_key)
            .first(&mut self.conn)
            .ok();

        diesel::delete(dsl::map_layers.find(id)).execute(&mut self.conn)?;

        // Clean up cached tiles
        if let Some(ref key) = layer_key {
            self.tile_cache.lock().unwrap().invalidate(key);
        }

        info!("Deleted map layer {} (key={:?})", id, layer_key);
        Ok(())
    }

    pub fn toggle_layer_visibility(&mut self, id: i32) -> MapResult<MapLayer> {
        let layer = self.get_layer(id)?;
        let new_visibility = !layer.visible;

        let update = UpdateMapLayer {
            visible: Some(new_visibility),
            updated_at: Utc::now().naive_utc(),
            ..Default::default()
        };

        self.update_layer(id, update)
    }

    // ========================================================================
    // FEATURE OPERATIONS
    // ========================================================================

    pub fn create_feature(&mut self, feature: NewMapFeature) -> MapResult<MapFeature> {
        use map_features::dsl;

        // Validate layer exists
        self.get_layer(feature.layer_id)?;

        diesel::insert_into(dsl::map_features)
            .values(&feature)
            .execute(&mut self.conn)?;

        let created = dsl::map_features
            .order(dsl::id.desc())
            .first(&mut self.conn)?;

        debug!("Created map feature: {}", feature.feature_key);
        Ok(created)
    }

    pub fn get_feature(&mut self, id: i32) -> MapResult<MapFeature> {
        use map_features::dsl;

        dsl::map_features
            .find(id)
            .first(&mut self.conn)
            .map_err(|_| MapError::FeatureNotFound(id.to_string()))
    }

    pub fn list_features_by_layer(&mut self, layer_id: i32) -> MapResult<Vec<MapFeature>> {
        use map_features::dsl;

        let features = dsl::map_features
            .filter(dsl::layer_id.eq(layer_id))
            .order(dsl::feature_key.asc())
            .load(&mut self.conn)?;

        Ok(features)
    }

    pub fn list_features(&mut self) -> MapResult<Vec<MapFeature>> {
        use map_features::dsl;

        let features = dsl::map_features
            .order(dsl::layer_id.asc())
            .then_order_by(dsl::feature_key.asc())
            .load(&mut self.conn)?;

        Ok(features)
    }

    pub fn count_features(&mut self) -> MapResult<i64> {
        use map_features::dsl;
        Ok(dsl::map_features.count().get_result(&mut self.conn)?)
    }

    /// Return features for a layer whose stored bbox intersects the given tile bbox.
    /// Uses SQL WHERE to avoid loading all features.
    pub fn list_features_by_layer_in_bbox(
        &mut self,
        layer_id: i32,
        tile_west: f64,
        tile_south: f64,
        tile_east: f64,
        tile_north: f64,
    ) -> MapResult<Vec<MapFeature>> {
        use map_features::dsl;

        // First try: load only features with bbox that overlaps the tile
        // bbox intersect: f.min_lon <= tile_east AND f.max_lon >= tile_west
        //               AND f.min_lat <= tile_north AND f.max_lat >= tile_south
        let filtered = dsl::map_features
            .filter(dsl::layer_id.eq(layer_id))
            .filter(dsl::bbox_min_lon.le(tile_east))
            .filter(dsl::bbox_max_lon.ge(tile_west))
            .filter(dsl::bbox_min_lat.le(tile_north))
            .filter(dsl::bbox_max_lat.ge(tile_south))
            .order(dsl::feature_key.asc())
            .load::<MapFeature>(&mut self.conn)?;

        if !filtered.is_empty() {
            return Ok(filtered);
        }

        // Fallback: no bbox-filtered results — might be features without bbox columns
        // or all features have NULL bboxes. Load all and return.
        let all = dsl::map_features
            .filter(dsl::layer_id.eq(layer_id))
            .order(dsl::feature_key.asc())
            .load::<MapFeature>(&mut self.conn)?;

        let has_any_bbox = all.iter().any(|f| f.bbox_min_lon.is_some());
        if has_any_bbox {
            // All features have bboxes but none matched — return empty
            Ok(vec![])
        } else {
            // No features have bboxes at all — return all (backwards compat)
            Ok(all)
        }
    }

    pub fn update_feature(&mut self, id: i32, update: UpdateMapFeature) -> MapResult<MapFeature> {
        use map_features::dsl;

        diesel::update(dsl::map_features.find(id))
            .set(&update)
            .execute(&mut self.conn)?;

        self.get_feature(id)
    }

    pub fn delete_feature(&mut self, id: i32) -> MapResult<()> {
        use map_features::dsl;

        diesel::delete(dsl::map_features.find(id)).execute(&mut self.conn)?;

        debug!("Deleted map feature: {}", id);
        Ok(())
    }

    pub fn count_features_by_layer(&mut self, layer_id: i32) -> MapResult<i64> {
        use map_features::dsl;

        let count = dsl::map_features
            .filter(dsl::layer_id.eq(layer_id))
            .count()
            .get_result(&mut self.conn)?;

        Ok(count)
    }

    // ========================================================================
    // STYLE OPERATIONS
    // ========================================================================

    pub fn create_style(&mut self, style: NewMapStyle) -> MapResult<MapStyle> {
        use map_styles::dsl;

        // If layer_id is present, validate layer exists
        if let Some(layer_id) = style.layer_id {
            self.get_layer(layer_id)?;
        }

        diesel::insert_into(dsl::map_styles)
            .values(&style)
            .execute(&mut self.conn)?;

        let created = dsl::map_styles
            .order(dsl::id.desc())
            .first(&mut self.conn)?;

        info!("Created map style: {}", style.name);
        Ok(created)
    }

    pub fn get_style(&mut self, id: i32) -> MapResult<MapStyle> {
        use map_styles::dsl;

        dsl::map_styles
            .find(id)
            .first(&mut self.conn)
            .map_err(|_| MapError::StyleNotFound(id.to_string()))
    }

    pub fn list_styles(&mut self) -> MapResult<Vec<MapStyle>> {
        use map_styles::dsl;

        let styles = dsl::map_styles
            .order(dsl::name.asc())
            .load(&mut self.conn)?;

        Ok(styles)
    }

    pub fn list_styles_by_layer(&mut self, layer_id: i32) -> MapResult<Vec<MapStyle>> {
        use map_styles::dsl;

        let styles = dsl::map_styles
            .filter(dsl::layer_id.eq(layer_id))
            .load(&mut self.conn)?;

        Ok(styles)
    }

    pub fn update_style(&mut self, id: i32, update: UpdateMapStyle) -> MapResult<MapStyle> {
        use map_styles::dsl;

        diesel::update(dsl::map_styles.find(id))
            .set(&update)
            .execute(&mut self.conn)?;

        self.get_style(id)
    }

    pub fn delete_style(&mut self, id: i32) -> MapResult<()> {
        use map_layers::dsl as ldsl;
        use map_styles::dsl;

        // Null out style_id on any layer pointing to this style
        diesel::update(ldsl::map_layers.filter(ldsl::style_id.eq(id)))
            .set(ldsl::style_id.eq(None::<i32>))
            .execute(&mut self.conn)?;

        diesel::delete(dsl::map_styles.find(id)).execute(&mut self.conn)?;

        info!("Deleted map style {}", id);
        Ok(())
    }

    // ========================================================================
    // TILE OPERATIONS
    // ========================================================================

    pub fn list_tiles(&mut self, layer_key: Option<&str>) -> MapResult<Vec<MapTile>> {
        use map_tiles::dsl;

        let mut q = dsl::map_tiles.into_boxed();
        if let Some(key) = layer_key {
            q = q.filter(dsl::layer_key.eq(key));
        }
        Ok(q.order(dsl::z.asc())
            .then_order_by(dsl::x.asc())
            .then_order_by(dsl::y.asc())
            .load(&mut self.conn)?)
    }

    pub fn count_tiles(&mut self) -> MapResult<i64> {
        use map_tiles::dsl;
        Ok(dsl::map_tiles.count().get_result(&mut self.conn)?)
    }

    pub fn create_tile(&mut self, tile: NewMapTile) -> MapResult<MapTile> {
        use map_tiles::dsl;

        diesel::insert_into(dsl::map_tiles)
            .values(&tile)
            .execute(&mut self.conn)?;

        let created = dsl::map_tiles.order(dsl::id.desc()).first(&mut self.conn)?;

        // Store in in-memory cache
        let cache_key = format!("{}:{}:{}:{}", tile.layer_key, tile.z, tile.x, tile.y);
        let etag = tile.etag.clone().unwrap_or_else(|| {
            crate::tile::generate_etag(
                &tile.layer_key,
                tile.z as u32,
                tile.x as u32,
                tile.y as u32,
                1,
            )
        });
        self.tile_cache
            .lock()
            .unwrap()
            .put(cache_key, bytes::Bytes::from(tile.data.clone()), etag);

        debug!("Created map tile: {}/{}/{}.pbf", tile.z, tile.x, tile.y);
        Ok(created)
    }

    pub fn get_tile(&mut self, layer_key: &str, z: i32, x: i32, y: i32) -> MapResult<MapTile> {
        use map_tiles::dsl;

        // Check in-memory cache first
        let cache_key = format!("{}:{}:{}:{}", layer_key, z, x, y);
        if let Some(cached_data) = self.tile_cache.lock().unwrap().get(&cache_key) {
            return Ok(MapTile {
                id: 0,
                layer_key: layer_key.to_string(),
                z,
                x,
                y,
                data: cached_data.to_vec(),
                etag: None,
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
            });
        }

        // Fall back to database
        dsl::map_tiles
            .filter(dsl::layer_key.eq(layer_key))
            .filter(dsl::z.eq(z))
            .filter(dsl::x.eq(x))
            .filter(dsl::y.eq(y))
            .first(&mut self.conn)
            .map_err(|_| MapError::TileGeneration(format!("Tile not found: {}/{}/{}.pbf", z, x, y)))
    }

    pub fn update_tile(
        &mut self,
        layer_key: &str,
        z: i32,
        x: i32,
        y: i32,
        update: UpdateMapTile,
    ) -> MapResult<MapTile> {
        use map_tiles::dsl;

        diesel::update(
            dsl::map_tiles
                .filter(dsl::layer_key.eq(layer_key))
                .filter(dsl::z.eq(z))
                .filter(dsl::x.eq(x))
                .filter(dsl::y.eq(y)),
        )
        .set(&update)
        .execute(&mut self.conn)?;

        self.get_tile(layer_key, z, x, y)
    }

    pub fn delete_tile(&mut self, layer_key: &str, z: i32, x: i32, y: i32) -> MapResult<()> {
        use map_tiles::dsl;

        diesel::delete(
            dsl::map_tiles
                .filter(dsl::layer_key.eq(layer_key))
                .filter(dsl::z.eq(z))
                .filter(dsl::x.eq(x))
                .filter(dsl::y.eq(y)),
        )
        .execute(&mut self.conn)?;

        debug!("Deleted map tile: {}/{}/{}.pbf", z, x, y);
        Ok(())
    }

    pub fn clear_tiles_for_layer(&mut self, layer_key: &str) -> MapResult<usize> {
        use map_tiles::dsl;

        let count = diesel::delete(dsl::map_tiles.filter(dsl::layer_key.eq(layer_key)))
            .execute(&mut self.conn)?;

        // Clear from in-memory cache
        self.tile_cache.lock().unwrap().invalidate(layer_key);

        info!("Cleared {} tiles for layer: {}", count, layer_key);
        Ok(count)
    }

    // ========================================================================
    // FEATURE → TILE INDEX OPERATIONS
    // ========================================================================

    pub fn index_tile_feature(
        &mut self,
        tile_feature: NewMapTileFeature,
    ) -> MapResult<MapTileFeature> {
        use map_tile_features::dsl;

        diesel::insert_into(dsl::map_tile_features)
            .values(&tile_feature)
            .execute(&mut self.conn)?;

        let created = dsl::map_tile_features
            .order(dsl::id.desc())
            .first(&mut self.conn)?;

        Ok(created)
    }

    pub fn list_tile_features(&mut self) -> MapResult<Vec<MapTileFeature>> {
        use map_tile_features::dsl;

        Ok(dsl::map_tile_features
            .order(dsl::z.asc())
            .then_order_by(dsl::x.asc())
            .then_order_by(dsl::y.asc())
            .load(&mut self.conn)?)
    }

    pub fn list_tile_features_by_feature(
        &mut self,
        feature_id: i32,
    ) -> MapResult<Vec<MapTileFeature>> {
        use map_tile_features::dsl;

        Ok(dsl::map_tile_features
            .filter(dsl::feature_id.eq(feature_id))
            .load(&mut self.conn)?)
    }

    pub fn list_tile_features_for_tile(
        &mut self,
        z: i32,
        x: i32,
        y: i32,
    ) -> MapResult<Vec<MapTileFeature>> {
        use map_tile_features::dsl;

        Ok(dsl::map_tile_features
            .filter(dsl::z.eq(z))
            .filter(dsl::x.eq(x))
            .filter(dsl::y.eq(y))
            .load(&mut self.conn)?)
    }

    pub fn get_features_for_tile(&mut self, z: i32, x: i32, y: i32) -> MapResult<Vec<MapFeature>> {
        use map_features::dsl as mf;
        use map_tile_features::dsl as tf;

        let feature_ids = tf::map_tile_features
            .filter(tf::z.eq(z))
            .filter(tf::x.eq(x))
            .filter(tf::y.eq(y))
            .select(tf::feature_id)
            .load::<i32>(&mut self.conn)?;

        let features = mf::map_features
            .filter(mf::id.eq_any(feature_ids))
            .load(&mut self.conn)?;

        Ok(features)
    }

    pub fn delete_tile_feature(&mut self, id: i32) -> MapResult<()> {
        use map_tile_features::dsl;

        diesel::delete(dsl::map_tile_features.find(id)).execute(&mut self.conn)?;

        Ok(())
    }

    pub fn clear_tile_features(&mut self) -> MapResult<usize> {
        use map_tile_features::dsl;

        let n = diesel::delete(dsl::map_tile_features).execute(&mut self.conn)?;
        Ok(n)
    }

    pub fn count_tile_features(&mut self) -> MapResult<i64> {
        use map_tile_features::dsl;
        Ok(dsl::map_tile_features.count().get_result(&mut self.conn)?)
    }

    // ========================================================================
    // COMPOSITE QUERIES
    // ========================================================================

    pub fn get_layer_with_feature_count(&mut self, id: i32) -> MapResult<LayerWithFeatures> {
        let layer = self.get_layer(id)?;
        let feature_count = self.count_features_by_layer(id)?;

        Ok(LayerWithFeatures {
            layer,
            feature_count,
        })
    }

    pub fn get_source_with_layer_count(&mut self, id: i32) -> MapResult<SourceWithLayers> {
        let source = self.get_source(id)?;
        let layers = self.list_layers_by_source(id)?;
        let layer_count = layers.len() as i64;

        Ok(SourceWithLayers {
            source,
            layer_count,
        })
    }
}

// Implement Default for update structs
impl Default for UpdateMapLayer {
    fn default() -> Self {
        Self {
            name: None,
            layer_key: None,
            layer_type: None,
            description: None,
            min_zoom: None,
            max_zoom: None,
            z_index: None,
            visible: None,
            style_id: None,
            updated_at: Utc::now().naive_utc(),
        }
    }
}

impl Default for UpdateMapSource {
    fn default() -> Self {
        Self {
            name: None,
            source_type: None,
            url: None,
            version: None,
            description: None,
            updated_at: Utc::now().naive_utc(),
        }
    }
}

impl Default for UpdateMapFeature {
    fn default() -> Self {
        Self {
            feature_key: None,
            feature_type: None,
            geometry_type: None,
            geometry: None,
            properties: None,
            updated_at: Utc::now().naive_utc(),
        }
    }
}

impl Default for UpdateMapStyle {
    fn default() -> Self {
        Self {
            layer_id: None,
            name: None,
            style_type: None,
            definition: None,
            updated_at: Utc::now().naive_utc(),
        }
    }
}

impl Default for UpdateMapTile {
    fn default() -> Self {
        Self {
            data: None,
            etag: None,
            updated_at: Utc::now().naive_utc(),
        }
    }
}
