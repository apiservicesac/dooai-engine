use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Query, State};
use axum::routing::post;
use axum::{Json, Router};
use engine_classify::Classification;
use engine_core::{EngineError, OcrOptions};
use engine_extract::Extraction;
use serde::{Deserialize, Serialize};

use crate::api_error::ApiError;
use crate::application_state::ApplicationState;
use crate::settings::ApiPaths;

/// `POST /v1/extract?language=ru`: the body is an image or a PDF. The document is read, its kind is told, and its
/// data (issuer, customer, rows, taxes, totals) is extracted and checked: every sum must add up.
pub struct ExtractRoute;

#[derive(Deserialize)]
struct ExtractQuery {
    language: Option<String>,
}

#[derive(Serialize)]
struct ExtractResponse {
    classification: Classification,
    #[serde(flatten)]
    extraction: Extraction,
    /// The language model that read the text.
    model: String,
    /// The text the data was extracted from.
    text: String,
}

impl ExtractRoute {
    pub fn router(max_upload_bytes: usize) -> Router<ApplicationState> {
        Router::new().route(
            ApiPaths::EXTRACT,
            post(Self::extract).layer(DefaultBodyLimit::max(max_upload_bytes)),
        )
    }

    async fn extract(
        State(state): State<ApplicationState>,
        Query(query): Query<ExtractQuery>,
        file: Bytes,
    ) -> Result<Json<ExtractResponse>, ApiError> {
        let extractor = state.extractor.clone().ok_or(ApiError::Unavailable(
            "no language model is configured: set ENGINE_LLM_URL and ENGINE_LLM_MODEL",
        ))?;
        let reader = Arc::clone(&state.document_reader);
        let classifier = Arc::clone(&state.classifier);
        let options = OcrOptions {
            language: query.language,
        };

        let response = tokio::task::spawn_blocking(move || {
            let document = reader.read(&file, &options)?;
            if document.text.trim().is_empty() {
                return Err(EngineError::InvalidDocument(
                    "no text could be read from the file".to_string(),
                ));
            }
            Ok::<_, EngineError>(ExtractResponse {
                classification: classifier.classify(&document.text),
                extraction: extractor.extract(&document.text)?,
                model: extractor.model_name().to_string(),
                text: document.text,
            })
        })
        .await
        .map_err(|error| ApiError::Worker(error.to_string()))??;
        Ok(Json(response))
    }
}
