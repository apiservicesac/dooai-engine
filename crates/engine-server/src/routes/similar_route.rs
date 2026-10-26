use axum::extract::{DefaultBodyLimit, State};
use axum::routing::post;
use axum::{Json, Router};
use engine_core::EngineError;
use engine_embed::{RankingOptions, SimilarMatch, SimilarityCandidate, TextRanker};
use serde::{Deserialize, Serialize};

use crate::api_error::ApiError;
use crate::application_state::ApplicationState;
use crate::settings::{ApiPaths, RequestLimits};

/// `POST /v1/similar`: which of the `candidates` mean the most like the `query`. Used to find duplicate contacts,
/// similar products and the invoice or contact a bank line refers to.
pub struct SimilarRoute;

#[derive(Deserialize)]
struct SimilarRequest {
    query: String,
    candidates: Vec<SimilarityCandidate>,
    top_k: Option<usize>,
    min_score: Option<f32>,
}

#[derive(Serialize)]
struct SimilarResponse {
    model: String,
    matches: Vec<SimilarMatch>,
}

impl SimilarRoute {
    pub fn router() -> Router<ApplicationState> {
        Router::new().route(
            ApiPaths::SIMILAR,
            post(Self::rank).layer(DefaultBodyLimit::max(RequestLimits::MAX_JSON_BYTES)),
        )
    }

    async fn rank(
        State(state): State<ApplicationState>,
        Json(request): Json<SimilarRequest>,
    ) -> Result<Json<SimilarResponse>, ApiError> {
        let embedder = state
            .embedder
            .clone()
            .ok_or(ApiError::Unavailable("the embedding model is not installed"))?;
        Self::check(&request)?;
        let options = RankingOptions {
            top_k: request
                .top_k
                .unwrap_or(RequestLimits::DEFAULT_TOP_K)
                .min(RequestLimits::MAX_TOP_K),
            min_score: request.min_score.unwrap_or(RequestLimits::DEFAULT_MIN_SCORE),
        };
        let response = tokio::task::spawn_blocking(move || {
            let model = embedder.model_name().to_string();
            let matches = TextRanker::new(embedder).rank(&request.query, &request.candidates, options)?;
            Ok::<_, EngineError>(SimilarResponse { model, matches })
        })
        .await
        .map_err(|error| ApiError::Worker(error.to_string()))??;
        Ok(Json(response))
    }

    fn check(request: &SimilarRequest) -> Result<(), EngineError> {
        if request.candidates.len() > RequestLimits::MAX_CANDIDATES {
            return Err(EngineError::InvalidRequest(format!(
                "send at most {} candidates",
                RequestLimits::MAX_CANDIDATES
            )));
        }
        let longest = request
            .candidates
            .iter()
            .map(|candidate| candidate.text.chars().count())
            .chain([request.query.chars().count()])
            .max()
            .unwrap_or(0);
        if longest > RequestLimits::MAX_TEXT_CHARACTERS {
            return Err(EngineError::InvalidRequest(format!(
                "a text has more than {} characters",
                RequestLimits::MAX_TEXT_CHARACTERS
            )));
        }
        Ok(())
    }
}
