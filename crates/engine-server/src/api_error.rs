use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use engine_core::EngineError;
use serde::Serialize;

/// What a failed request answers with: a status and `{"error": "..."}`.
/// Invalid input is explained to the caller; internal failures are only logged.
#[derive(Debug)]
pub enum ApiError {
    Engine(EngineError),
    Worker(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl ApiError {
    const INTERNAL_MESSAGE: &'static str = "internal error";

    fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            Self::Engine(error @ EngineError::InvalidImage(_)) => (StatusCode::UNPROCESSABLE_ENTITY, error.to_string()),
            Self::Engine(error) => {
                tracing::error!(%error, "request failed");
                (StatusCode::INTERNAL_SERVER_ERROR, Self::INTERNAL_MESSAGE.to_string())
            }
            Self::Worker(message) => {
                tracing::error!(%message, "worker thread failed");
                (StatusCode::INTERNAL_SERVER_ERROR, Self::INTERNAL_MESSAGE.to_string())
            }
        }
    }
}

impl From<EngineError> for ApiError {
    fn from(error: EngineError) -> Self {
        Self::Engine(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error) = self.status_and_message();
        (status, Json(ErrorBody { error })).into_response()
    }
}
