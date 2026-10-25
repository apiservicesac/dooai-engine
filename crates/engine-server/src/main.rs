use std::sync::Arc;

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
    let ocr_engine = PaddleOcrEngine::load(&config.models_directory)?;

    Application::new(config, Arc::new(ocr_engine)).serve().await?;
    Ok(())
}
