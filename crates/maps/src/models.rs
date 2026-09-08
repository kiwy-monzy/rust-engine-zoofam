use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

use crate::schema::{
    map_features, map_layers, map_sources, map_styles, map_tile_features, map_tiles,
};

// ============================================================================
// TRAITS
// ============================================================================

/// Core trait for all map entities
pub trait MapEntity {
    fn id(&self) -> i32;
    fn created_at(&self) -> NaiveDateTime;
    fn updated_at(&self) -> NaiveDateTime;
}

/// Trait for entities that can be published/visible
pub trait Publishable {
    fn is_visible(&self) -> bool;
    fn set_visible(&mut self, visible: bool);
}

/// Trait for entities with hierarchical zoom levels
pub trait ZoomRange {
    fn min_zoom(&self) -> u32;
    fn max_zoom(&self) -> u32;
    fn is_zoom_valid(&self, zoom: u32) -> bool {
        zoom >= self.min_zoom() && zoom <= self.max_zoom()
    }
}

/// Trait for entities that can be styled
pub trait Styleable {
    fn style_id(&self) -> Option<i32>;
    fn set_style_id(&mut self, style_id: Option<i32>);
}

/// Trait for spatial entities with geometry
pub trait Spatial {
    fn geometry_type(&self) -> &str;
    fn has_geometry(&self) -> bool;
}

/// Trait for tile cache entities
pub trait Cacheable {
    fn etag(&self) -> Option<&str>;
    fn set_etag(&mut self, etag: String);
    fn is_cached(&self) -> bool;
}

// ============================================================================
// SOURCE
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, ToSchema)]
#[diesel(table_name = map_sources)]
pub struct MapSource {
    pub id: i32,
    pub name: String,
    pub source_type: String,
    pub url: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl MapEntity for MapSource {
    fn id(&self) -> i32 {
        self.id
    }
    fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }
    fn updated_at(&self) -> NaiveDateTime {
        self.updated_at
    }
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = map_sources)]
pub struct NewMapSource {
    pub name: String,
    pub source_type: String,
    pub url: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = map_sources)]
pub struct UpdateMapSource {
    pub name: Option<String>,
    pub source_type: Option<String>,
    pub url: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub updated_at: NaiveDateTime,
}

// ============================================================================
// LAYER
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, ToSchema)]
#[diesel(table_name = map_layers)]
pub struct MapLayer {
    pub id: i32,
    pub source_id: i32,
    pub name: String,
    pub layer_key: String,
    pub layer_type: String,
    pub description: Option<String>,
    pub min_zoom: i32,
    pub max_zoom: i32,
    pub z_index: i32,
    pub visible: bool,
    pub style_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl MapEntity for MapLayer {
    fn id(&self) -> i32 {
        self.id
    }
    fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }
    fn updated_at(&self) -> NaiveDateTime {
        self.updated_at
    }
}

impl Publishable for MapLayer {
    fn is_visible(&self) -> bool {
        self.visible
    }
    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

impl ZoomRange for MapLayer {
    fn min_zoom(&self) -> u32 {
        self.min_zoom as u32
    }
    fn max_zoom(&self) -> u32 {
        self.max_zoom as u32
    }
}

impl Styleable for MapLayer {
    fn style_id(&self) -> Option<i32> {
        self.style_id
    }
    fn set_style_id(&mut self, style_id: Option<i32>) {
        self.style_id = style_id;
    }
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = map_layers)]
pub struct NewMapLayer {
    pub source_id: i32,
    pub name: String,
    pub layer_key: String,
    pub layer_type: String,
    pub description: Option<String>,
    pub min_zoom: i32,
    pub max_zoom: i32,
    pub z_index: i32,
    pub visible: bool,
    pub style_id: Option<i32>,
}

#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = map_layers)]
pub struct UpdateMapLayer {
    pub name: Option<String>,
    pub layer_key: Option<String>,
    pub layer_type: Option<String>,
    pub description: Option<String>,
    pub min_zoom: Option<i32>,
    pub max_zoom: Option<i32>,
    pub z_index: Option<i32>,
    pub visible: Option<bool>,
    pub style_id: Option<Option<i32>>,
    pub updated_at: NaiveDateTime,
}

// ============================================================================
// FEATURE
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, ToSchema)]
#[diesel(table_name = map_features)]
pub struct MapFeature {
    pub id: i32,
    pub layer_id: i32,
    pub feature_key: String,
    pub feature_type: String,
    pub geometry_type: String,
    pub geometry: Vec<u8>,
    pub properties: Option<String>,
    pub bbox_min_lon: Option<f64>,
    pub bbox_min_lat: Option<f64>,
    pub bbox_max_lon: Option<f64>,
    pub bbox_max_lat: Option<f64>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl MapEntity for MapFeature {
    fn id(&self) -> i32 {
        self.id
    }
    fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }
    fn updated_at(&self) -> NaiveDateTime {
        self.updated_at
    }
}

impl Spatial for MapFeature {
    fn geometry_type(&self) -> &str {
        &self.geometry_type
    }
    fn has_geometry(&self) -> bool {
        !self.geometry.is_empty()
    }
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = map_features)]
pub struct NewMapFeature {
    pub layer_id: i32,
    pub feature_key: String,
    pub feature_type: String,
    pub geometry_type: String,
    pub geometry: Vec<u8>,
    pub properties: Option<String>,
    pub bbox_min_lon: Option<f64>,
    pub bbox_min_lat: Option<f64>,
    pub bbox_max_lon: Option<f64>,
    pub bbox_max_lat: Option<f64>,
}

#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = map_features)]
pub struct UpdateMapFeature {
    pub feature_key: Option<String>,
    pub feature_type: Option<String>,
    pub geometry_type: Option<String>,
    pub geometry: Option<Vec<u8>>,
    pub properties: Option<Option<String>>,
    pub updated_at: NaiveDateTime,
}

// ============================================================================
// STYLE
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, ToSchema)]
#[diesel(table_name = map_styles)]
pub struct MapStyle {
    pub id: i32,
    pub layer_id: Option<i32>,
    pub name: String,
    pub style_type: String,
    pub definition: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl MapEntity for MapStyle {
    fn id(&self) -> i32 {
        self.id
    }
    fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }
    fn updated_at(&self) -> NaiveDateTime {
        self.updated_at
    }
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = map_styles)]
pub struct NewMapStyle {
    pub layer_id: Option<i32>,
    pub name: String,
    pub style_type: String,
    pub definition: String,
}

#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = map_styles)]
pub struct UpdateMapStyle {
    pub layer_id: Option<Option<i32>>,
    pub name: Option<String>,
    pub style_type: Option<String>,
    pub definition: Option<String>,
    pub updated_at: NaiveDateTime,
}

// ============================================================================
// TILE
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, ToSchema)]
#[diesel(table_name = map_tiles)]
pub struct MapTile {
    pub id: i32,
    pub layer_key: String,
    pub z: i32,
    pub x: i32,
    pub y: i32,
    pub data: Vec<u8>,
    pub etag: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl MapEntity for MapTile {
    fn id(&self) -> i32 {
        self.id
    }
    fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }
    fn updated_at(&self) -> NaiveDateTime {
        self.updated_at
    }
}

impl Cacheable for MapTile {
    fn etag(&self) -> Option<&str> {
        self.etag.as_deref()
    }
    fn set_etag(&mut self, etag: String) {
        self.etag = Some(etag);
    }
    fn is_cached(&self) -> bool {
        !self.data.is_empty()
    }
}

impl fmt::Display for MapTile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}.pbf", self.z, self.x, self.y)
    }
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = map_tiles)]
pub struct NewMapTile {
    pub layer_key: String,
    pub z: i32,
    pub x: i32,
    pub y: i32,
    pub data: Vec<u8>,
    pub etag: Option<String>,
}

#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = map_tiles)]
pub struct UpdateMapTile {
    pub data: Option<Vec<u8>>,
    pub etag: Option<Option<String>>,
    pub updated_at: NaiveDateTime,
}

// ============================================================================
// FEATURE → TILE INDEX (per-feature tile presence)
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, ToSchema)]
#[diesel(table_name = map_tile_features)]
pub struct MapTileFeature {
    pub id: i32,
    pub feature_id: i32,
    pub z: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = map_tile_features)]
pub struct NewMapTileFeature {
    pub feature_id: i32,
    pub z: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = map_tile_features)]
pub struct UpdateMapTileFeature {
    pub feature_id: Option<i32>,
    pub z: Option<i32>,
    pub x: Option<i32>,
    pub y: Option<i32>,
}

// ============================================================================
// COMPOSITE TYPES
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct LayerWithFeatures {
    #[serde(flatten)]
    pub layer: MapLayer,
    pub feature_count: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SourceWithLayers {
    #[serde(flatten)]
    pub source: MapSource,
    pub layer_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct TileRequest {
    pub layer_key: String,
    pub z: u32,
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Deserialize)]
pub struct TileBBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}
