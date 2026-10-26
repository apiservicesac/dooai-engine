use std::sync::Arc;

use engine_classify::{Lexicon, LexiconClassifier};
use engine_embed::E5Embedder;
use engine_ocr::PaddleOcrEngine;
use engine_server::{Application, ServerConfig};
use tracing_subscriber::EnvFilter;

struct LogFilter;

impl LogFilter {
    /// ONNX Runtime logs every device and session option at info level.
    const DEFAULT: &'static str = "info,ort=warn";
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(LogFilter::DEFAULT)))
        .init();

    let config = ServerConfig::from_environment()?;
    tracing::info!(models = %config.models_directory.display(), "loading OCR models");
    let models_directory = config.models_directory.clone();
    let lexicon_file = config.lexicon_file.clone();
    let ocr_engine = PaddleOcrEngine::load(&models_directory)?;

    let mut application = Application::new(config, Arc::new(ocr_engine));
    if let Some(path) = lexicon_file {
        let lexicon = Lexicon::from_json(&std::fs::read_to_string(path)?)?;
        application = application.with_classifier(Arc::new(LexiconClassifier::new(lexicon)));
    }
    if E5Embedder::is_installed(&models_directory) {
        tracing::info!("loading the embedding model");
        application = application.with_embedder(Arc::new(E5Embedder::load(&models_directory)?));
    } else {
        tracing::warn!("the embedding model is not installed: /v1/embed and /v1/similar are disabled");
    }
    application.serve().await?;
    Ok(())
}
