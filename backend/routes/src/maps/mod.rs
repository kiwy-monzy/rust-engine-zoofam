use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use controller::maps::*;
use serde::Deserialize;

pub fn map_routes() -> Router<AppState> {
    Router::new()
        // Source routes - require maps:read for GET, maps:write for POST/PATCH/DELETE
        .route(
            "/map/sources",
            get(list_sources_handler).post(create_source_handler),
        )
        .route(
            "/map/sources/:id",
            get(get_source_handler)
                .patch(update_source_handler)
                .delete(delete_source_handler),
        )
        // Layer routes
        .route(
            "/map/layers",
            get(list_layers_handler).post(create_layer_handler),
        )
        .route(
            "/map/layers/:id",
            get(get_layer_handler)
                .patch(update_layer_handler)
                .delete(delete_layer_handler),
        )
        .route(
            "/map/layers/:id/toggle-visibility",
            post(toggle_layer_visibility_handler),
        )
        .route("/map/layers/by-key/:key", get(get_layer_by_key_handler))
        .route(
            "/map/layers/source/:source_id",
            get(list_layers_by_source_handler),
        )
        .route("/map/layers/visible", get(list_visible_layers_handler))
        // Feature routes
        .route(
            "/map/features",
            get(list_features_handler).post(create_feature_handler),
        )
        .route(
            "/map/features/:id",
            get(get_feature_handler)
                .patch(update_feature_handler)
                .delete(delete_feature_handler),
        )
        .route(
            "/map/features/layer/:layer_id",
            get(list_features_by_layer_handler),
        )
        .route(
            "/map/features/layer/:layer_id/geojson",
            get(export_geojson_handler),
        )
        .route(
            "/map/features/layer/:layer_id/count",
            get(count_features_by_layer_handler),
        )
        // Style routes
        .route(
            "/map/styles",
            get(list_styles_handler).post(create_style_handler),
        )
        .route(
            "/map/styles/:id",
            get(get_style_handler)
                .patch(update_style_handler)
                .delete(delete_style_handler),
        )
        .route(
            "/map/styles/layer/:layer_id",
            get(list_styles_by_layer_handler),
        )
        // Tile cache routes
        .route(
            "/map/tiles",
            get(list_tiles_handler).post(create_tile_handler),
        )
        .route(
            "/map/tiles/:layer_key/:z/:x/:y",
            get(get_tile_handler)
                .patch(update_tile_handler)
                .delete(delete_tile_handler),
        )
        .route(
            "/map/tiles/layer/:layer_key/clear",
            delete(clear_tiles_for_layer_handler),
        )
        // Per-feature tile index (formerly `feature_tiles` → `map_tile_features`)
        .route(
            "/map/tile-features",
            get(list_tile_features_handler).post(create_tile_feature_handler),
        )
        .route(
            "/map/tile-features/clear",
            delete(clear_tile_features_handler),
        )
        .route("/map/tile-features/count", get(count_tile_features_handler))
        .route(
            "/map/tile-features/tile/:z/:x/:y",
            get(list_tile_features_for_tile_handler),
        )
        .route(
            "/map/tile-features/feature/:feature_id",
            get(list_tile_features_by_feature_handler),
        )
        .route(
            "/map/tile-features/:id",
            delete(delete_tile_feature_handler),
        )
        // Composite queries
        .route(
            "/map/layers/:id/with-features",
            get(get_layer_with_feature_count_handler),
        )
        .route(
            "/map/sources/:id/with-layers",
            get(get_source_with_layer_count_handler),
        )
}

// ============================================================================
// SOURCE HANDLERS
// ============================================================================

async fn create_source_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateSourceRequest>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let source = state.controllers().create_source(req)?;
    Ok(Json(source))
}

async fn get_source_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let source = state.controllers().get_source(id)?;
    Ok(Json(source))
}

async fn list_sources_handler(
    State(state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let sources = state.controllers().list_sources()?;
    Ok(Json(serde_json::json!({ "sources": sources })))
}

async fn update_source_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateSourceRequest>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let source = state.controllers().update_source(id, req)?;
    Ok(Json(source))
}

async fn delete_source_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    state.controllers().delete_source(id)?;
    Ok(Json(
        serde_json::json!({"message": "Source deleted successfully"}),
    ))
}

// ============================================================================
// LAYER HANDLERS
// ============================================================================

async fn create_layer_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateLayerRequest>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let layer = state.controllers().create_layer(req)?;
    Ok(Json(layer))
}

async fn get_layer_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let layer = state.controllers().get_layer(id)?;
    Ok(Json(layer))
}

async fn get_layer_by_key_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let layer = state.controllers().get_layer_by_key(&key)?;
    Ok(Json(layer))
}

async fn list_layers_handler(
    State(state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let layers = state.controllers().list_layers()?;
    Ok(Json(serde_json::json!({ "layers": layers })))
}

async fn list_visible_layers_handler(
    State(state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let layers = state.controllers().list_visible_layers()?;
    Ok(Json(layers))
}

async fn list_layers_by_source_handler(
    State(state): State<AppState>,
    Path(source_id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let layers = state.controllers().list_layers_by_source(source_id)?;
    Ok(Json(serde_json::json!({ "layers": layers })))
}

async fn update_layer_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateLayerRequest>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let layer = state.controllers().update_layer(id, req)?;
    Ok(Json(layer))
}

async fn delete_layer_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    state.controllers().delete_layer(id)?;
    Ok(Json(
        serde_json::json!({"message": "Layer deleted successfully"}),
    ))
}

async fn toggle_layer_visibility_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let layer = state.controllers().toggle_layer_visibility(id)?;
    Ok(Json(layer))
}

// ============================================================================
// FEATURE HANDLERS
// ============================================================================

async fn create_feature_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateFeatureRequest>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let feature = state.controllers().create_feature(req)?;
    Ok(Json(feature))
}

async fn list_features_handler(
    State(state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let features = state.controllers().list_features()?;
    Ok(Json(serde_json::json!({ "features": features })))
}

async fn get_feature_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let feature = state.controllers().get_feature(id)?;
    Ok(Json(feature))
}

async fn list_features_by_layer_handler(
    State(state): State<AppState>,
    Path(layer_id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let features = state.controllers().list_features_by_layer(layer_id)?;
    Ok(Json(serde_json::json!({ "features": features })))
}

async fn export_geojson_handler(
    State(state): State<AppState>,
    Path(layer_id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine as _;

    let features = state.controllers().list_features_by_layer(layer_id)?;
    let mut geo_features: Vec<serde_json::Value> = Vec::new();
    for f in features {
        let geom = STANDARD
            .decode(&f.geometry)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
            .unwrap_or(serde_json::Value::Null);
        let props = f
            .properties
            .as_ref()
            .and_then(|p| serde_json::from_str::<serde_json::Value>(p).ok())
            .unwrap_or(serde_json::json!({}));
        geo_features.push(serde_json::json!({
            "type": "Feature",
            "id": f.id,
            "geometry": geom,
            "properties": props,
        }));
    }
    let fc = serde_json::json!({
        "type": "FeatureCollection",
        "features": geo_features,
    });
    Ok(Json(fc))
}

async fn update_feature_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateFeatureRequest>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let feature = state.controllers().update_feature(id, req)?;
    Ok(Json(feature))
}

async fn delete_feature_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    state.controllers().delete_feature(id)?;
    Ok(Json(
        serde_json::json!({"message": "Feature deleted successfully"}),
    ))
}

async fn count_features_by_layer_handler(
    State(state): State<AppState>,
    Path(layer_id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let count = state.controllers().count_features_by_layer(layer_id)?;
    Ok(Json(serde_json::json!({"count": count})))
}

// ============================================================================
// STYLE HANDLERS
// ============================================================================

async fn create_style_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateStyleRequest>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let style = state.controllers().create_style(req)?;
    Ok(Json(style))
}

async fn get_style_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let style = state.controllers().get_style(id)?;
    Ok(Json(style))
}

async fn list_styles_handler(
    State(state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let styles = state.controllers().list_styles()?;
    Ok(Json(serde_json::json!({ "styles": styles })))
}

async fn list_styles_by_layer_handler(
    State(state): State<AppState>,
    Path(layer_id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let styles = state.controllers().list_styles_by_layer(layer_id)?;
    Ok(Json(serde_json::json!({ "styles": styles })))
}

async fn update_style_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateStyleRequest>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let style = state.controllers().update_style(id, req)?;
    Ok(Json(style))
}

async fn delete_style_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    state.controllers().delete_style(id)?;
    Ok(Json(
        serde_json::json!({"message": "Style deleted successfully"}),
    ))
}

// ============================================================================
// TILE HANDLERS
// ============================================================================

#[derive(Debug, Default, Deserialize)]
struct TileListQuery {
    layer_key: Option<String>,
}

async fn list_tiles_handler(
    State(state): State<AppState>,
    Query(q): Query<TileListQuery>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let tiles = state.controllers().list_tiles(q.layer_key.as_deref())?;
    Ok(Json(serde_json::json!({ "tiles": tiles })))
}

async fn create_tile_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateTileRequest>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let tile = state.controllers().create_tile(req)?;
    Ok(Json(tile))
}

async fn get_tile_handler(
    State(state): State<AppState>,
    Path((layer_key, z, x, y)): Path<(String, i32, i32, i32)>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let tile = state.controllers().get_tile(&layer_key, z, x, y)?;
    Ok(Json(tile))
}

async fn update_tile_handler(
    State(state): State<AppState>,
    Path((layer_key, z, x, y)): Path<(String, i32, i32, i32)>,
    Json(req): Json<UpdateTileRequest>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let tile = state.controllers().update_tile(&layer_key, z, x, y, req)?;
    Ok(Json(tile))
}

async fn delete_tile_handler(
    State(state): State<AppState>,
    Path((layer_key, z, x, y)): Path<(String, i32, i32, i32)>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    state.controllers().delete_tile(&layer_key, z, x, y)?;
    Ok(Json(
        serde_json::json!({"message": "Tile deleted successfully"}),
    ))
}

async fn clear_tiles_for_layer_handler(
    State(state): State<AppState>,
    Path(layer_key): Path<String>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let count = state.controllers().clear_tiles_for_layer(&layer_key)?;
    Ok(Json(serde_json::json!({"cleared": count})))
}

// ============================================================================
// TILE-FEATURE INDEX HANDLERS
// ============================================================================

async fn list_tile_features_handler(
    State(state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let entries = state.controllers().list_tile_features()?;
    Ok(Json(serde_json::json!({ "tile_features": entries })))
}

async fn list_tile_features_for_tile_handler(
    State(state): State<AppState>,
    Path((z, x, y)): Path<(i32, i32, i32)>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let entries = state.controllers().list_tile_features_for_tile(z, x, y)?;
    Ok(Json(serde_json::json!({ "tile_features": entries })))
}

async fn list_tile_features_by_feature_handler(
    State(state): State<AppState>,
    Path(feature_id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let entries = state
        .controllers()
        .list_tile_features_by_feature(feature_id)?;
    Ok(Json(serde_json::json!({ "tile_features": entries })))
}

async fn create_tile_feature_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateTileFeatureRequest>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let entry = state.controllers().create_tile_feature(req)?;
    Ok(Json(entry))
}

async fn delete_tile_feature_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    state.controllers().delete_tile_feature(id)?;
    Ok(Json(
        serde_json::json!({"message": "Tile-feature index entry deleted"}),
    ))
}

async fn clear_tile_features_handler(
    State(state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let n = state.controllers().clear_tile_features()?;
    Ok(Json(serde_json::json!({"cleared": n})))
}

async fn count_tile_features_handler(
    State(state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let n = state.controllers().count_tile_features()?;
    Ok(Json(serde_json::json!({"count": n})))
}

// ============================================================================
// COMPOSITE QUERY HANDLERS
// ============================================================================

async fn get_layer_with_feature_count_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let layer = state.controllers().get_layer_with_feature_count(id)?;
    Ok(Json(layer))
}

async fn get_source_with_layer_count_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl axum::response::IntoResponse, crate::middleware::ApiError> {
    let source = state.controllers().get_source_with_layer_count(id)?;
    Ok(Json(source))
}
