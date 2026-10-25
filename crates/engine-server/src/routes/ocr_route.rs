use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::routing::post;
use axum::{Json, Router};
use engine_core::OcrDocument;

use crate::api_error::ApiError;
use crate::application_state::ApplicationState;
use crate::settings::ApiPaths;

/// `POST /v1/ocr`: the request body is the encoded image (PNG, JPEG, WebP...).
pub struct OcrRoute;

impl OcrRoute {
    pub fn router(max_upload_bytes: usize) -> Router<ApplicationState> {
        Router::new().route(
            ApiPaths::OCR,
            post(Self::read).layer(DefaultBodyLimit::max(max_upload_bytes)),
        )
    }

    async fn read(State(state): State<ApplicationState>, image: Bytes) -> Result<Json<OcrDocument>, ApiError> {
        let engine = Arc::clone(&state.ocr_engine);
        let document = tokio::task::spawn_blocking(move || engine.read(&image))
            .await
            .map_err(|error| ApiError::Worker(error.to_string()))??;
        Ok(Json(document))
    }
}
