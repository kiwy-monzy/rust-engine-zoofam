use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;

use crate::state::AppState;

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    20
}

#[utoipa::path(
    get,
    path = "/api/v1/search",
    tag = "search",
    params(
        ("q" = String, Query, description = "Search query"),
        ("limit" = Option<usize>, Query, description = "Max results")
    ),
    responses(
        (status = 200, description = "Search results"),
        (status = 503, description = "Search service unavailable")
    ),
    security(("bearer_auth" = []))
)]
pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let index = state.search.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "search not available".into(),
        )
    })?;

    let results = index
        .search(&params.q, params.limit)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({ "results": results })))
}

#[utoipa::path(
    post,
    path = "/api/v1/search/reindex",
    tag = "search",
    responses(
        (status = 200, description = "Reindexing complete"),
        (status = 503, description = "Search service unavailable")
    ),
    security(("bearer_auth" = []))
)]
pub async fn reindex(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let index = state.search.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "search not available".into(),
        )
    })?;

    let controllers = state.controllers();

    // Re-index storage files.
    let storage_svc = storage::StorageService::from_env();
    let admin = controller::users::find_by_email(&state.pool, "admin@example.com")
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let mut conn = db::conn(&state.pool)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let files = storage_svc
        .list_files(&mut conn, &admin.id, None)
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    index
        .delete_by_kind("file")
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut file_docs = Vec::new();
    for f in &files {
        let url = format!(
            "/api/v1/storage/{}/{}/{}",
            f.user_id,
            f.collection,
            f.filename.replace(' ', "%20")
        );
        let description = if f.mime_type == "application/geo+json" {
            format!("{} GeoJSON file", f.collection)
        } else {
            format!("{} {}", f.collection, f.mime_type)
        };
        file_docs.push(search::IndexDoc {
            id: format!("file:{}", f.id),
            kind: "file".into(),
            name: f.filename.clone(),
            collection: Some(f.collection.clone()),
            description,
            url: Some(url),
            meta: Some(serde_json::json!({
                "size_bytes": f.size_bytes,
                "mime_type": f.mime_type,
            })),
        });
    }
    if !file_docs.is_empty() {
        index
            .index_batch(&file_docs)
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    // Re-index layers.
    let sources = controllers
        .list_sources()
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let layers = controllers
        .list_layers()
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let styles = controllers
        .list_styles()
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let source_map: std::collections::HashMap<i32, &str> =
        sources.iter().map(|s| (s.id, s.name.as_str())).collect();
    let mut layer_styles: std::collections::HashMap<i32, Vec<&str>> =
        std::collections::HashMap::new();
    for s in &styles {
        if let Some(lid) = s.layer_id {
            layer_styles.entry(lid).or_default().push(&s.name);
        }
    }

    index
        .delete_by_kind("source")
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    index
        .delete_by_kind("layer")
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut map_docs = Vec::new();
    for s in &sources {
        map_docs.push(search::IndexDoc {
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
            })),
        });
    }
    for l in &layers {
        let source_name = source_map.get(&l.source_id).unwrap_or(&"");
        let style_names = layer_styles.get(&l.id).cloned().unwrap_or_default();
        map_docs.push(search::IndexDoc {
            id: format!("layer:{}", l.id),
            kind: "layer".into(),
            name: l.name.clone(),
            collection: None,
            description: format!(
                "{} layer \"{}\" zoom {}-{}",
                l.layer_type, l.layer_key, l.min_zoom, l.max_zoom,
            ),
            url: None,
            meta: Some(serde_json::json!({
                "layer_key": l.layer_key,
                "layer_type": l.layer_type,
                "source_name": source_name,
                "styles": style_names,
            })),
        });
    }
    if !map_docs.is_empty() {
        index
            .index_batch(&map_docs)
            .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(Json(serde_json::json!({
        "indexed": {
            "files": file_docs.len(),
            "sources": sources.len(),
            "layers": layers.len(),
        }
    })))
}
