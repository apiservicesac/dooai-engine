use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Query, State};
use axum::routing::post;
use axum::{Json, Router};
use engine_core::{OcrOptions, ReadDocument};
use serde::Deserialize;

use crate::api_error::ApiError;
use crate::application_state::ApplicationState;
use crate::settings::ApiPaths;

/// `POST /v1/ocr?language=ru`: the request body is an image (PNG, JPEG, WebP...) or a PDF.
/// `language` is optional; without it the engine picks the alphabet itself.
pub struct OcrRoute;

#[derive(Deserialize)]
struct OcrQuery {
    language: Option<String>,
}

impl OcrRoute {
    pub fn router(max_upload_bytes: usize) -> Router<ApplicationState> {
        Router::new().route(
            ApiPaths::OCR,
            post(Self::read).layer(DefaultBodyLimit::max(max_upload_bytes)),
        )
    }

    async fn read(
        State(state): State<ApplicationState>,
        Query(query): Query<OcrQuery>,
        image: Bytes,
    ) -> Result<Json<ReadDocument>, ApiError> {
        let reader = Arc::clone(&state.document_reader);
        let options = OcrOptions {
            language: query.language,
        };
        let document = tokio::task::spawn_blocking(move || reader.read(&image, &options))
            .await
            .map_err(|error| ApiError::Worker(error.to_string()))??;
        Ok(Json(document))
    }
}
