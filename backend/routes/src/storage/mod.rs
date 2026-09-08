use auth::Claims;
use axum::extract::{Extension, Multipart, Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use db::DbPool;
use utoipa::ToSchema;

use crate::middleware::ApiError;
use crate::state::AppState;

const MAX_FILE_BYTES: usize = 25 * 1024 * 1024;

pub fn storage_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/storage/:collection",
            get(list_files_handler).post(upload_file_handler),
        )
        .route(
            "/storage/:collection/:filename",
            get(get_file_handler).delete(delete_file_handler),
        )
        .layer(axum::extract::DefaultBodyLimit::max(25 * 1024 * 1024))
}

pub fn storage_public_routes() -> Router<AppState> {
    Router::new().route(
        "/storage/:user_id/:collection/:filename",
        get(serve_file_handler),
    )
}

fn svc() -> storage::StorageService {
    storage::StorageService::from_env()
}

fn conn(pool: &DbPool) -> Result<db::DbConn, ApiError> {
    db::conn(pool).map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, format!("db: {e}")))
}

#[derive(ToSchema)]
pub struct UploadFileResponse {
    pub id: i32,
    pub collection: String,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub created_at: chrono::NaiveDateTime,
}

#[utoipa::path(
    post,
    path = "/api/v1/storage/{collection}",
    tag = "storage",
    params(
        ("collection" = String, Path, description = "File collection")
    ),
    request_body(
        content = Vec<u8>,
        content_type = "multipart/form-data",
        description = "File upload as multipart/form-data with a `file` field"
    ),
    responses(
        (status = 200, description = "File uploaded", body = UploadFileResponse),
        (status = 400, description = "Invalid upload"),
        (status = 401, description = "Not authenticated"),
        (status = 413, description = "File too large")
    ),
    security(("bearer_auth" = []))
)]
async fn upload_file_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(collection): Path<String>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let mut filename: Option<String> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut mime_type = String::from("application/octet-stream");

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, format!("multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let ct = field
                .content_type()
                .map(|s| s.to_string())
                .unwrap_or_else(|| mime_type.clone());
            mime_type = ct;
            filename = field
                .file_name()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            let data = field
                .bytes()
                .await
                .map_err(|e| ApiError::new(StatusCode::BAD_REQUEST, format!("read error: {e}")))?;
            if data.len() > MAX_FILE_BYTES {
                return Err(ApiError::new(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    format!("file too large (max {} MB)", MAX_FILE_BYTES / (1024 * 1024)),
                ));
            }
            file_data = Some(data.to_vec());
        }
    }

    let fname =
        filename.ok_or_else(|| ApiError::new(StatusCode::BAD_REQUEST, "missing file field"))?;
    let data =
        file_data.ok_or_else(|| ApiError::new(StatusCode::BAD_REQUEST, "missing file data"))?;

    let service = svc();
    let mut c = conn(&state.pool)?;
    let file = service
        .upload_file(&mut c, &claims.sub, &collection, &fname, &mime_type, &data)
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "id": file.id,
        "collection": file.collection,
        "filename": file.filename,
        "mime_type": file.mime_type,
        "size_bytes": file.size_bytes,
        "created_at": file.created_at,
    })))
}

#[utoipa::path(
    get,
    path = "/api/v1/storage/{collection}",
    tag = "storage",
    params(
        ("collection" = String, Path, description = "File collection")
    ),
    responses(
        (status = 200, description = "List of files"),
        (status = 401, description = "Not authenticated")
    ),
    security(("bearer_auth" = []))
)]
async fn list_files_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(collection): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let service = svc();
    let mut c = conn(&state.pool)?;
    let files = service
        .list_files(&mut c, &claims.sub, Some(&collection))
        .map_err(|e| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({ "files": files })))
}

#[utoipa::path(
    get,
    path = "/api/v1/storage/{collection}/{filename}",
    tag = "storage",
    params(
        ("collection" = String, Path, description = "File collection"),
        ("filename" = String, Path, description = "File name")
    ),
    responses(
        (status = 200, description = "File metadata"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "File not found")
    ),
    security(("bearer_auth" = []))
)]
async fn get_file_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((collection, filename)): Path<(String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    let service = svc();
    let mut c = conn(&state.pool)?;
    let file = service
        .get_file(&mut c, &claims.sub, &collection, &filename)
        .map_err(|e| match e {
            storage::Error::NotFound(_) => ApiError::new(StatusCode::NOT_FOUND, e.to_string()),
            other => ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, other.to_string()),
        })?;
    Ok(Json(file))
}

#[utoipa::path(
    delete,
    path = "/api/v1/storage/{collection}/{filename}",
    tag = "storage",
    params(
        ("collection" = String, Path, description = "File collection"),
        ("filename" = String, Path, description = "File name")
    ),
    responses(
        (status = 204, description = "File deleted"),
        (status = 401, description = "Not authenticated"),
        (status = 404, description = "File not found")
    ),
    security(("bearer_auth" = []))
)]
async fn delete_file_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((collection, filename)): Path<(String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    let service = svc();
    let mut c = conn(&state.pool)?;
    service
        .delete_file(&mut c, &claims.sub, &collection, &filename)
        .map_err(|e| match e {
            storage::Error::NotFound(_) => ApiError::new(StatusCode::NOT_FOUND, e.to_string()),
            other => ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, other.to_string()),
        })?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/storage/{user_id}/{collection}/{filename}",
    tag = "storage",
    params(
        ("user_id" = String, Path, description = "User ID"),
        ("collection" = String, Path, description = "File collection"),
        ("filename" = String, Path, description = "File name")
    ),
    responses(
        (status = 200, description = "File content"),
        (status = 404, description = "File not found")
    )
)]
async fn serve_file_handler(
    Path((user_id, collection, filename)): Path<(String, String, String)>,
) -> Result<Response, ApiError> {
    let service = svc();
    let path = service
        .file_path_on_disk(&user_id, &collection, &filename)
        .map_err(|e| match e {
            storage::Error::NotFound(_) => ApiError::new(StatusCode::NOT_FOUND, e.to_string()),
            other => ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, other.to_string()),
        })?;

    let mime = mime_from_ext(&filename).to_string();
    let bytes = std::fs::read(&path).map_err(|e| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("read error: {e}"),
        )
    })?;

    Ok((
        [
            (header::CONTENT_TYPE, mime),
            (header::CACHE_CONTROL, "public, max-age=3600".to_string()),
        ],
        bytes,
    )
        .into_response())
}

fn mime_from_ext(filename: &str) -> &'static str {
    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "geojson" => "application/geo+json",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "csv" => "text/csv",
        "txt" => "text/plain",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}
