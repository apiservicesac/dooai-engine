use std::sync::Arc;

use axum::Router;
use engine_classify::{Lexicon, LexiconClassifier};
use engine_core::{OcrEngine, TextEmbedder};
use engine_document::DocumentReader;
use tokio::net::TcpListener;

use crate::application_state::ApplicationState;
use crate::routes::{ClassifyRoute, EmbedRoute, HealthRoute, OcrRoute, SimilarRoute};
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
            state: ApplicationState {
                document_reader: Arc::new(DocumentReader::new(ocr_engine)),
                embedder: None,
                classifier: Arc::new(LexiconClassifier::new(Lexicon::built_in())),
            },
        }
    }

    /// Replaces the built-in kinds of document with others.
    pub fn with_classifier(mut self, classifier: Arc<LexiconClassifier>) -> Self {
        self.state.classifier = classifier;
        self
    }

    /// Turns on `/v1/embed` and `/v1/similar`.
    pub fn with_embedder(mut self, embedder: Arc<dyn TextEmbedder>) -> Self {
        self.state.embedder = Some(embedder);
        self
    }

    pub fn router(&self) -> Router {
        Router::new()
            .merge(HealthRoute::router())
            .merge(OcrRoute::router(self.config.max_upload_bytes))
            .merge(EmbedRoute::router())
            .merge(SimilarRoute::router())
            .merge(ClassifyRoute::router())
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
