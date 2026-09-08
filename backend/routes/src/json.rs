use axum::async_trait;
use axum::extract::{FromRequest, Request};
use axum::http::{header::CONTENT_TYPE, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::error::Category;

use crate::middleware::ApiError;

/// A JSON body extractor that never echoes parser internals to the client.
/// Axum's built-in `Json<T>` rejection quotes serde messages verbatim
/// ("invalid type: integer `12345678`, expected a string at line 1 column 40"),
/// which hands an attacker a map of the wire format. Here a malformed body is
/// always one of two fixed sentences, with the detail left to the status code.
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let is_json = req
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|ct| ct.starts_with("application/json"));
        if !is_json {
            return Err(ApiError::new(
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "requests must use the application/json content type",
            ));
        }

        // Bytes consults DefaultBodyLimit, so the router-wide cap still applies.
        let bytes = axum::body::Bytes::from_request(req, state)
            .await
            .map_err(|_| {
                ApiError::new(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "the request body is too large",
                )
            })?;

        match serde_json::from_slice::<T>(&bytes) {
            Ok(value) => Ok(Self(value)),
            Err(e) => {
                tracing::debug!(error = %e, "request body rejected");
                Err(match e.classify() {
                    Category::Data => ApiError::new(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "the request body fields have the wrong types or are missing",
                    ),
                    _ => ApiError::new(
                        StatusCode::BAD_REQUEST,
                        "the request body is not valid JSON",
                    ),
                })
            }
        }
    }
}
