use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::application_state::ApplicationState;
use crate::settings::ApiPaths;

pub struct HealthRoute;

#[derive(Serialize)]
struct HealthStatus {
    status: &'static str,
    /// What this installation can do: `ocr`, and `embeddings` when the model is installed.
    capabilities: Vec<&'static str>,
}

impl HealthRoute {
    pub fn router() -> Router<ApplicationState> {
        Router::new().route(ApiPaths::HEALTH, get(Self::check))
    }

    async fn check(State(state): State<ApplicationState>) -> Json<HealthStatus> {
        Json(HealthStatus {
            status: "ok",
            capabilities: state.capabilities(),
        })
    }
}
