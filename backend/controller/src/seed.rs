//! Map data seeding: uploads GeoJSON to storage, creates sources/layers, and imports features.
//!
//! This module provides an idempotent seed function that:
//! 1. Reads GeoJSON files from the api-gateway assets
//! 2. Uploads them to storage under the admin user
//! 3. Creates map_sources, map_layers, map_styles
//! 4. Imports the GeoJSON features into map_features

use std::path::PathBuf;

use crate::maps::{CreateLayerRequest, CreateSourceRequest, CreateStyleRequest};
use crate::{Controllers, Result};
use storage::StorageService;

/// Definition of a map layer to seed.
struct LayerDef {
    id: &'static str,
    name: &'static str,
    file: &'static str,
    source_name: &'static str,
    source_type: &'static str,
    layer_type: &'static str,
    z_index: u8,
    style: Option<StyleDef>,
}

struct StyleDef {
    name: &'static str,
    style_type: &'static str,
    stroke: &'static str,
    fill: Option<&'static str>,
    width: f32,
    radius: Option<f32>,
    label_field: Option<&'static str>,
}

/// All layers to seed, in order.
const LAYER_DEFS: &[LayerDef] = &[
    LayerDef {
        id: "regions",
        name: "Regions (mkoa)",
        file: "regions.geojson",
        source_name: "Tanzania Admin Boundaries",
        source_type: "geojson",
        layer_type: "polygon",
        z_index: 0,
        style: Some(StyleDef {
            name: "Regions Fill",
            style_type: "fill",
            stroke: "#4a5568",
            fill: Some("#1f2733"),
            width: 1.0,
            radius: None,
            label_field: Some("NAME_1"),
        }),
    },
    LayerDef {
        id: "water",
        name: "Lakes and reservoirs",
        file: "water.geojson",
        source_name: "Tanzania Water Bodies",
        source_type: "geojson",
        layer_type: "polygon",
        z_index: 1,
        style: Some(StyleDef {
            name: "Water Fill",
            style_type: "fill",
            stroke: "#2e6fa8",
            fill: Some("#2e6fa8"),
            width: 0.8,
            radius: None,
            label_field: Some("name"),
        }),
    },
    LayerDef {
        id: "rivers",
        name: "Rivers",
        file: "rivers.geojson",
        source_name: "Tanzania Hydrography",
        source_type: "geojson",
        layer_type: "line",
        z_index: 5,
        style: Some(StyleDef {
            name: "River Line",
            style_type: "line",
            stroke: "#3e8fc8",
            fill: None,
            width: 0.7,
            radius: None,
            label_field: Some("name"),
        }),
    },
    LayerDef {
        id: "regional_roads",
        name: "Regional roads (R)",
        file: "regional_roads_2022.geojson",
        source_name: "Tanzania Road Network",
        source_type: "geojson",
        layer_type: "line",
        z_index: 6,
        style: Some(StyleDef {
            name: "Regional Road",
            style_type: "line",
            stroke: "#c88a2a",
            fill: None,
            width: 1.0,
            radius: None,
            label_field: Some("RoadLabel"),
        }),
    },
    LayerDef {
        id: "trunk_roads",
        name: "Trunk roads (T)",
        file: "trunk_roads_2022.geojson",
        source_name: "Tanzania Road Network",
        source_type: "geojson",
        layer_type: "line",
        z_index: 7,
        style: Some(StyleDef {
            name: "Trunk Road",
            style_type: "line",
            stroke: "#c0392b",
            fill: None,
            width: 1.8,
            radius: None,
            label_field: Some("RoadLabel"),
        }),
    },
    LayerDef {
        id: "railways",
        name: "Railways",
        file: "railways.geojson",
        source_name: "Tanzania Railways",
        source_type: "geojson",
        layer_type: "line",
        z_index: 10,
        style: Some(StyleDef {
            name: "Railway Line",
            style_type: "line",
            stroke: "#e8e8e8",
            fill: None,
            width: 1.5,
            radius: None,
            label_field: None,
        }),
    },
    LayerDef {
        id: "ferry_routes",
        name: "Ferry routes",
        file: "ferry_routes.geojson",
        source_name: "Tanzania Maritime",
        source_type: "geojson",
        layer_type: "line",
        z_index: 11,
        style: Some(StyleDef {
            name: "Ferry Line",
            style_type: "line",
            stroke: "#4fc3f7",
            fill: None,
            width: 1.4,
            radius: None,
            label_field: Some("name"),
        }),
    },
    LayerDef {
        id: "railway_stations",
        name: "Railway stations",
        file: "railway_stations.geojson",
        source_name: "Tanzania Railways",
        source_type: "geojson",
        layer_type: "point",
        z_index: 20,
        style: Some(StyleDef {
            name: "Station Point",
            style_type: "point",
            stroke: "#e0a33c",
            fill: Some("#2b2114"),
            width: 1.5,
            radius: Some(4.0),
            label_field: Some("name"),
        }),
    },
    LayerDef {
        id: "seaports",
        name: "Seaports",
        file: "seaports.geojson",
        source_name: "Tanzania Maritime",
        source_type: "geojson",
        layer_type: "point",
        z_index: 21,
        style: Some(StyleDef {
            name: "Port Point",
            style_type: "point",
            stroke: "#4fc3f7",
            fill: Some("#12262e"),
            width: 1.5,
            radius: Some(4.5),
            label_field: Some("name"),
        }),
    },
    LayerDef {
        id: "airports",
        name: "Airports",
        file: "airports.geojson",
        source_name: "Tanzania Aviation",
        source_type: "geojson",
        layer_type: "point",
        z_index: 22,
        style: Some(StyleDef {
            name: "Airport Point",
            style_type: "point",
            stroke: "#46c47f",
            fill: Some("#13291f"),
            width: 1.5,
            radius: Some(4.5),
            label_field: Some("name"),
        }),
    },
];

/// Seed map data idempotently.
///
/// - Skips layers that already exist (by layer_key).
/// - Uploads GeoJSON to storage under the admin user.
/// - Creates sources, layers, styles, and imports features from disk.
pub fn seed_map_data(controllers: &Controllers) -> Result<SeedResult> {
    let storage = StorageService::from_env();
    let mut result = SeedResult::default();

    // Look up the admin user (created by ensure_admin before this runs).
    let mut conn = db::conn(&controllers.pool)?;
    let admin_user_id = match crate::users::find_by_email(&mut conn, "admin@example.com") {
        Ok(u) => u.id,
        Err(_) => {
            result
                .errors
                .push("admin user not found for storage uploads".into());
            return Ok(result);
        }
    };

    // Group layers by source_name to share source IDs.
    let mut source_ids: std::collections::HashMap<&str, i32> = std::collections::HashMap::new();

    // Find the GeoJSON assets directory.
    let assets_dir = find_assets_dir();
    if assets_dir.is_none() {
        result
            .errors
            .push("could not find api-gateway assets directory".into());
        return Ok(result);
    }
    let assets_dir = assets_dir.unwrap();

    for def in LAYER_DEFS {
        // Check if layer already exists.
        if let Ok(existing) = controllers.get_layer_by_key(def.id) {
            tracing::debug!(
                "layer {} already exists (id={}), skipping",
                def.id,
                existing.id
            );
            result.skipped += 1;
            continue;
        }

        // Read the GeoJSON file.
        let geojson_path = assets_dir.join(def.file);
        let geojson_text = match std::fs::read_to_string(&geojson_path) {
            Ok(t) => t,
            Err(e) => {
                result
                    .errors
                    .push(format!("failed to read {}: {e}", def.file));
                continue;
            }
        };

        // Get or create source (with storage URL on first creation).
        let source_id = if let Some(&sid) = source_ids.get(def.source_name) {
            sid
        } else {
            // Upload to storage under admin user.
            let mut conn = db::conn(&controllers.pool)
                .map_err(|e| crate::Error::Invalid(format!("db pool: {e}")))?;
            let storage_url = match storage.upload_file(
                &mut conn,
                &admin_user_id,
                "maps",
                def.file,
                "application/geo+json",
                geojson_text.as_bytes(),
            ) {
                Ok(file) => {
                    tracing::info!(
                        "uploaded to storage: {} ({} bytes)",
                        def.file,
                        file.size_bytes
                    );
                    Some(format!("/api/v1/storage/{admin_user_id}/maps/{}", def.file))
                }
                Err(e) => {
                    tracing::warn!("storage upload failed for {}: {e}", def.file);
                    None
                }
            };

            let source = controllers.create_source(CreateSourceRequest {
                name: def.source_name.into(),
                source_type: def.source_type.into(),
                url: storage_url,
                version: None,
                description: None,
            })?;
            tracing::info!("created source: {} (id={})", def.source_name, source.id);
            source_ids.insert(def.source_name, source.id);
            result.sources += 1;
            source.id
        };

        // Create the layer.
        let layer = controllers.create_layer(CreateLayerRequest {
            source_id,
            name: def.name.into(),
            layer_key: def.id.into(),
            layer_type: def.layer_type.into(),
            description: Some(format!("Seeded from {}", def.file)),
            min_zoom: 0,
            max_zoom: 22,
            z_index: def.z_index as i32,
            visible: true,
            style_id: None,
        })?;
        tracing::info!("created layer: {} (id={})", def.name, layer.id);
        result.layers += 1;

        // Create style if defined.
        if let Some(s) = &def.style {
            let definition = serde_json::json!({
                "stroke": s.stroke,
                "fill": s.fill,
                "width": s.width,
                "radius": s.radius,
                "label_field": s.label_field,
            });
            let style = controllers.create_style(CreateStyleRequest {
                layer_id: Some(layer.id),
                name: s.name.into(),
                style_type: s.style_type.into(),
                definition: definition.to_string(),
            })?;
            tracing::info!("created style: {} (id={})", s.name, style.id);
            result.styles += 1;
        }

        // Import features.
        match controllers.import_geojson(layer.id, def.id, &geojson_text) {
            Ok(import_result) => {
                result.features += import_result.imported;
                if !import_result.errors.is_empty() {
                    for e in &import_result.errors {
                        result.errors.push(format!("{}: {}", def.id, e));
                    }
                }
                tracing::info!(
                    "imported {} features for {}",
                    import_result.imported,
                    def.id
                );
            }
            Err(e) => {
                result.errors.push(format!("{} import failed: {e}", def.id));
            }
        }
    }

    Ok(result)
}

#[derive(Debug, Default)]
pub struct SeedResult {
    pub sources: usize,
    pub layers: usize,
    pub styles: usize,
    pub features: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

/// Find the api-gateway assets directory by looking for known paths.
fn find_assets_dir() -> Option<PathBuf> {
    let candidates = [
        // From app/ workspace
        PathBuf::from("../crates/api-gateway/assets/geo"),
        // From crates/ workspace
        PathBuf::from("../api-gateway/assets/geo"),
        // Absolute fallback
        PathBuf::from("crates/api-gateway/assets/geo"),
    ];
    for p in &candidates {
        if p.join("regions.geojson").is_file() {
            return Some(p.clone());
        }
    }
    None
}
