use std::sync::Arc;

use axum::Router;
use engine_core::OcrEngine;
use tokio::net::TcpListener;

use crate::application_state::ApplicationState;
use crate::routes::{HealthRoute, OcrRoute};
use crate::server_config::ServerConfig;
use crate::shutdown_signal::ShutdownSignal;

pub struct Application {
    config: ServerConfig,
    state: ApplicationState,
}

impl Application {
    pub fn new(config: ServerConfig, ocr_engine: Arc<dyn OcrEngine>) -> Self {
        Self {
            config,
            state: ApplicationState { ocr_engine },
        }
    }

    pub fn router(&self) -> Router {
        Router::new()
            .merge(HealthRoute::router())
            .merge(OcrRoute::router(self.config.max_upload_bytes))
            .with_state(self.state.clone())
    }

    pub async fn serve(self) -> std::io::Result<()> {
        let listener = TcpListener::bind(self.config.address).await?;
        tracing::info!(address = %self.config.address, "engine server listening");
        axum::serve(listener, self.router())
            .with_graceful_shutdown(ShutdownSignal::wait())
            .await
    }
}
