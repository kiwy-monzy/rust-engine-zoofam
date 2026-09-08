use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};

use crate::middleware::ApiError;
use crate::state::AppState;

const MAX_UPLOAD_BYTES: usize = 25 * 1024 * 1024;

pub fn upload_routes() -> Router<AppState> {
    Router::new().route("/map/layers/:layer_id/import", post(import_geojson_handler))
}

#[utoipa::path(
    post,
    path = "/api/v1/map/layers/{layer_id}/import",
    tag = "uploads",
    params(
        ("layer_id" = i32, Path, description = "Layer ID")
    ),
    request_body(
        content = Vec<u8>,
        content_type = "multipart/form-data",
        description = "GeoJSON file upload as multipart/form-data with `feature_type` and `file` fields"
    ),
    responses(
        (status = 200, description = "Import complete"),
        (status = 400, description = "Invalid upload"),
        (status = 401, description = "Not authenticated"),
        (status = 413, description = "File too large")
    ),
    security(("bearer_auth" = []))
)]
async fn import_geojson_handler(
    State(state): State<AppState>,
    Path(layer_id): Path<i32>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let mut feature_type = String::from("feature");
    let mut file_text: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, format!("multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "feature_type" => {
                let text = field.text().await.map_err(|e| {
                    ApiError::new(StatusCode::BAD_REQUEST, format!("read error: {e}"))
                })?;
                if !text.trim().is_empty() {
                    feature_type = text.trim().to_string();
                }
            }
            "file" => {
                let data = field.bytes().await.map_err(|e| {
                    ApiError::new(StatusCode::BAD_REQUEST, format!("read error: {e}"))
                })?;
                if data.len() > MAX_UPLOAD_BYTES {
                    return Err(ApiError::new(
                        StatusCode::PAYLOAD_TOO_LARGE,
                        format!(
                            "file too large (max {} MB)",
                            MAX_UPLOAD_BYTES / (1024 * 1024)
                        ),
                    ));
                }
                file_text = Some(String::from_utf8(data.to_vec()).map_err(|e| {
                    ApiError::new(StatusCode::BAD_REQUEST, format!("not valid UTF-8: {e}"))
                })?);
            }
            _ => {}
        }
    }

    let geojson_text =
        file_text.ok_or_else(|| ApiError::new(StatusCode::BAD_REQUEST, "missing file field"))?;

    let result = state
        .controllers()
        .import_geojson(layer_id, &feature_type, &geojson_text)?;

    Ok(Json(serde_json::json!({
        "imported": result.imported,
        "errors": result.errors,
    })))
}
