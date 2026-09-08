use thiserror::Error;

#[derive(Debug, Error)]
pub enum MapError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Layer not found: {0}")]
    LayerNotFound(String),

    #[error("Source not found: {0}")]
    SourceNotFound(String),

    #[error("Feature not found: {0}")]
    FeatureNotFound(String),

    #[error("Style not found: {0}")]
    StyleNotFound(String),

    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),

    #[error("Invalid tile coordinates: z={z}, x={x}, y={y}")]
    InvalidTileCoordinates { z: u32, x: u32, y: u32 },

    #[error("Tile generation failed: {0}")]
    TileGeneration(String),

    #[error("Zoom level out of range: {z} (min: {min}, max: {max})")]
    ZoomOutOfRange { z: u32, min: u32, max: u32 },

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Unauthorized access to layer: {0}")]
    Unauthorized(String),

    #[error("Configuration error: {0}")]
    Configuration(String),
}

impl From<diesel::result::Error> for MapError {
    fn from(err: diesel::result::Error) -> Self {
        MapError::Database(err.to_string())
    }
}

impl From<serde_json::Error> for MapError {
    fn from(err: serde_json::Error) -> Self {
        MapError::Serialization(err.to_string())
    }
}

pub type MapResult<T> = Result<T, MapError>;
