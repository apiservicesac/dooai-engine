use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::application_state::ApplicationState;
use crate::settings::ApiPaths;

pub struct HealthRoute;

#[derive(Serialize)]
struct HealthStatus {
    status: &'static str,
}

impl HealthRoute {
    pub fn router() -> Router<ApplicationState> {
        Router::new().route(ApiPaths::HEALTH, get(Self::check))
    }

    async fn check() -> Json<HealthStatus> {
        Json(HealthStatus { status: "ok" })
    }
}
