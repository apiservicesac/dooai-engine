use axum::extract::{DefaultBodyLimit, State};
use axum::routing::post;
use axum::{Json, Router};
use engine_core::EngineError;
use serde::{Deserialize, Serialize};

use crate::api_error::ApiError;
use crate::application_state::ApplicationState;
use crate::settings::{ApiPaths, RequestLimits};

/// `POST /v1/embed`: `{"texts": ["..."]}` answers one unit-length vector per text, in order.
pub struct EmbedRoute;

#[derive(Deserialize)]
struct EmbedRequest {
    texts: Vec<String>,
}

#[derive(Serialize)]
struct EmbedResponse {
    model: String,
    dimensions: usize,
    embeddings: Vec<Vec<f32>>,
}

impl EmbedRoute {
    pub fn router() -> Router<ApplicationState> {
        Router::new().route(
            ApiPaths::EMBED,
            post(Self::embed).layer(DefaultBodyLimit::max(RequestLimits::MAX_JSON_BYTES)),
        )
    }

    async fn embed(
        State(state): State<ApplicationState>,
        Json(request): Json<EmbedRequest>,
    ) -> Result<Json<EmbedResponse>, ApiError> {
        let embedder = state
            .embedder
            .clone()
            .ok_or(ApiError::Unavailable("the embedding model is not installed"))?;
        Self::check(&request.texts)?;
        let response = tokio::task::spawn_blocking(move || {
            Ok::<_, EngineError>(EmbedResponse {
                model: embedder.model_name().to_string(),
                dimensions: embedder.dimensions(),
                embeddings: embedder.embed(&request.texts)?,
            })
        })
        .await
        .map_err(|error| ApiError::Worker(error.to_string()))??;
        Ok(Json(response))
    }

    fn check(texts: &[String]) -> Result<(), EngineError> {
        if texts.is_empty() || texts.len() > RequestLimits::MAX_TEXTS {
            return Err(EngineError::InvalidRequest(format!(
                "send between 1 and {} texts",
                RequestLimits::MAX_TEXTS
            )));
        }
        if texts
            .iter()
            .any(|text| text.chars().count() > RequestLimits::MAX_TEXT_CHARACTERS)
        {
            return Err(EngineError::InvalidRequest(format!(
                "a text has more than {} characters",
                RequestLimits::MAX_TEXT_CHARACTERS
            )));
        }
        Ok(())
    }
}
