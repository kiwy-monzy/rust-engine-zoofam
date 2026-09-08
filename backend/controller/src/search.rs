use std::path::PathBuf;
use std::sync::Arc;

use search::{IndexDoc, SearchIndex};

use crate::Controllers;

/// Initialize the search index and populate it from storage + maps.
pub fn init_search(
    controllers: &Controllers,
    index_dir: &PathBuf,
) -> Result<Arc<SearchIndex>, String> {
    let index = SearchIndex::open(index_dir).map_err(|e| format!("search index: {e}"))?;

    // Index storage files.
    index_storage_files(controllers, &index)?;

    // Index map layers.
    index_map_layers(controllers, &index)?;

    tracing::info!("search index ready at {}", index_dir.display());
    Ok(Arc::new(index))
}

fn index_storage_files(controllers: &Controllers, index: &SearchIndex) -> Result<(), String> {
    let storage = storage::StorageService::from_env();

    // Get all users' files (we list for admin user since that's where seed data is).
    let mut conn = db::conn(&controllers.pool).map_err(|e| format!("db: {e}"))?;
    let admin = crate::users::find_by_email(&mut conn, "admin@example.com")
        .map_err(|e| format!("find admin: {e}"))?;
    let files = storage
        .list_files(&mut conn, &admin.id, None)
        .map_err(|e| format!("list files: {e}"))?;

    let mut docs = Vec::new();
    for f in &files {
        let url = format!("/api/v1/storage/{}/{}", f.user_id, f.collection);

        // For GeoJSON files, parse and extract property keys for richer search.
        let description = if f.mime_type == "application/geo+json" {
            format!("{} {} file", f.collection, extension_label(&f.filename))
        } else {
            format!("{} {}", f.collection, f.mime_type)
        };

        docs.push(IndexDoc {
            id: format!("file:{}", f.id),
            kind: "file".into(),
            name: f.filename.clone(),
            collection: Some(f.collection.clone()),
            description,
            url: Some(format!("{}/{}", url, url_encode(&f.filename))),
            meta: Some(serde_json::json!({
                "size_bytes": f.size_bytes,
                "mime_type": f.mime_type,
                "user_id": f.user_id,
            })),
        });
    }

    if !docs.is_empty() {
        index
            .delete_by_kind("file")
            .map_err(|e| format!("delete files: {e}"))?;
        index
            .index_batch(&docs)
            .map_err(|e| format!("index files: {e}"))?;
        tracing::info!("indexed {} storage files", docs.len());
    }
    Ok(())
}

fn index_map_layers(controllers: &Controllers, index: &SearchIndex) -> Result<(), String> {
    let sources = controllers
        .list_sources()
        .map_err(|e| format!("list sources: {e}"))?;
    let layers = controllers
        .list_layers()
        .map_err(|e| format!("list layers: {e}"))?;
    let styles = controllers
        .list_styles()
        .map_err(|e| format!("list styles: {e}"))?;

    // Build source name map.
    let source_map: std::collections::HashMap<i32, &str> =
        sources.iter().map(|s| (s.id, s.name.as_str())).collect();

    // Build styles-per-layer map.
    let mut layer_styles: std::collections::HashMap<i32, Vec<&str>> =
        std::collections::HashMap::new();
    for s in &styles {
        if let Some(lid) = s.layer_id {
            layer_styles.entry(lid).or_default().push(&s.name);
        }
    }

    let mut docs = Vec::new();

    // Index sources.
    for s in &sources {
        docs.push(IndexDoc {
            id: format!("source:{}", s.id),
            kind: "source".into(),
            name: s.name.clone(),
            collection: None,
            description: format!(
                "{} {}",
                s.source_type,
                s.description.as_deref().unwrap_or("")
            ),
            url: s.url.clone(),
            meta: Some(serde_json::json!({
                "source_type": s.source_type,
                "version": s.version,
            })),
        });
    }

    // Index layers.
    for l in &layers {
        let source_name = source_map.get(&l.source_id).unwrap_or(&"");
        let style_names = layer_styles.get(&l.id).cloned().unwrap_or_default();

        docs.push(IndexDoc {
            id: format!("layer:{}", l.id),
            kind: "layer".into(),
            name: l.name.clone(),
            collection: None,
            description: format!(
                "{} layer \"{}\" zoom {}-{} zindex {}",
                l.layer_type, l.layer_key, l.min_zoom, l.max_zoom, l.z_index,
            ),
            url: None,
            meta: Some(serde_json::json!({
                "layer_key": l.layer_key,
                "layer_type": l.layer_type,
                "source_name": source_name,
                "source_id": l.source_id,
                "min_zoom": l.min_zoom,
                "max_zoom": l.max_zoom,
                "z_index": l.z_index,
                "visible": l.visible,
                "styles": style_names,
            })),
        });
    }

    if !docs.is_empty() {
        index
            .delete_by_kind("source")
            .map_err(|e| format!("delete sources: {e}"))?;
        index
            .delete_by_kind("layer")
            .map_err(|e| format!("delete layers: {e}"))?;
        index
            .index_batch(&docs)
            .map_err(|e| format!("index layers: {e}"))?;
        tracing::info!(
            "indexed {} sources + {} layers",
            sources.len(),
            layers.len()
        );
    }
    Ok(())
}

fn extension_label(filename: &str) -> &str {
    if filename.ends_with(".geojson") {
        "GeoJSON"
    } else if filename.ends_with(".json") {
        "JSON"
    } else if filename.ends_with(".csv") {
        "CSV"
    } else if filename.ends_with(".shp") {
        "Shapefile"
    } else {
        "data"
    }
}

fn url_encode(s: &str) -> String {
    s.replace(' ', "%20")
}
