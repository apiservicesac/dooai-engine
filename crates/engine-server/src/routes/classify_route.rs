use axum::extract::{DefaultBodyLimit, State};
use axum::routing::post;
use axum::{Json, Router};
use engine_classify::Classification;
use engine_core::EngineError;
use serde::Deserialize;

use crate::api_error::ApiError;
use crate::application_state::ApplicationState;
use crate::settings::{ApiPaths, RequestLimits};

/// `POST /v1/classify`: `{"text": "..."}` says what kind of document the text is (invoice, credit note, delivery
/// note, receipt, contract...). Only the start of the text is read.
pub struct ClassifyRoute;

#[derive(Deserialize)]
struct ClassifyRequest {
    text: String,
}

impl ClassifyRoute {
    pub fn router() -> Router<ApplicationState> {
        Router::new().route(
            ApiPaths::CLASSIFY,
            post(Self::classify).layer(DefaultBodyLimit::max(RequestLimits::MAX_JSON_BYTES)),
        )
    }

    async fn classify(
        State(state): State<ApplicationState>,
        Json(request): Json<ClassifyRequest>,
    ) -> Result<Json<Classification>, ApiError> {
        if request.text.chars().count() > RequestLimits::MAX_CLASSIFY_CHARACTERS {
            return Err(EngineError::InvalidRequest(format!(
                "the text has more than {} characters",
                RequestLimits::MAX_CLASSIFY_CHARACTERS
            ))
            .into());
        }
        Ok(Json(state.classifier.classify(&request.text)))
    }
}
