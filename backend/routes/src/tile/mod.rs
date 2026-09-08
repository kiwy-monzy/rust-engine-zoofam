use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use maps::models::ZoomRange;
use maps::{generate_etag, TileGenerator};

use crate::state::AppState;

pub fn tile_routes() -> Router<AppState> {
    Router::new().route("/tiles/:layer_key/:z/:x/*y", get(tile_handler))
}

/// Serve vector tiles — `.json` suffix in the path → GeoJSON, otherwise → MVT protobuf.
#[utoipa::path(
    get,
    path = "/api/v1/tiles/{layer_key}/{z}/{x}/{y}",
    tag = "tiles",
    params(
        ("layer_key" = String, Path, description = "Layer key"),
        ("z" = u32, Path, description = "Zoom level"),
        ("x" = u32, Path, description = "Tile X coordinate"),
        ("y" = String, Path, description = "Tile Y coordinate (append .json for GeoJSON)")
    ),
    responses(
        (status = 200, description = "Tile data (MVT protobuf or GeoJSON)"),
        (status = 400, description = "Invalid tile coordinates"),
        (status = 404, description = "Layer not found")
    )
)]
async fn tile_handler(
    State(state): State<AppState>,
    Path((layer_key, z, x, y_path)): Path<(String, u32, u32, String)>,
) -> Result<Response, crate::middleware::ApiError> {
    // Parse "0.json" → (0, true) or "16" → (16, false)
    let wants_json = y_path.ends_with(".json");
    let y_str = if wants_json {
        y_path.trim_end_matches(".json")
    } else {
        &y_path
    };
    let y: u32 = y_str.parse().map_err(|_| {
        controller::Error::Invalid(format!("Invalid tile coordinate y: {}", y_path))
    })?;

    let layer = state.controllers().get_layer_by_key(&layer_key)?;

    if !layer.is_zoom_valid(z) {
        return Err(controller::Error::Invalid(format!(
            "Zoom {} out of range [{}, {}]",
            z,
            layer.min_zoom(),
            layer.max_zoom()
        ))
        .into());
    }

    let features = if wants_json {
        // For GeoJSON export, return all features (used by preview fitBounds)
        state.controllers().list_features_by_layer(layer.id)?
    } else {
        // For PBF tiles, filter by tile bbox for performance
        let gen = TileGenerator::new(layer_key.clone(), z, x, y);
        let (west, south, east, north) = gen.tile_bbox();
        state
            .controllers()
            .list_features_by_layer_in_bbox(layer.id, west, south, east, north)?
    };

    if wants_json {
        let generator = TileGenerator::new(layer_key, z, x, y);
        let geojson = generator.features_to_geojson(&features)?;
        Ok(Json(geojson).into_response())
    } else {
        // Try cached tile first
        if let Ok(cached_tile) = state
            .controllers()
            .get_tile(&layer_key, z as i32, x as i32, y as i32)
        {
            if !cached_tile.data.is_empty() {
                let etag = cached_tile
                    .etag
                    .unwrap_or_else(|| generate_etag(&layer_key, z, x, y, 1));
                return Ok(tile_response(cached_tile.data, &etag));
            }
        }

        let generator = TileGenerator::new(layer_key.clone(), z, x, y);
        let tile_data = generator.generate_tile(&features)?;
        let etag = generate_etag(&layer_key, z, x, y, 1);

        Ok(tile_response(tile_data.to_vec(), &etag))
    }
}

fn tile_response(data: Vec<u8>, etag: &str) -> Response {
    use axum::http::header;

    (
        [
            (
                header::CONTENT_TYPE,
                "application/x-protobuf;type=mapbox-vector",
            ),
            (header::CACHE_CONTROL, "public, max-age=3600"),
            (header::ETAG, etag),
        ],
        data,
    )
        .into_response()
}
