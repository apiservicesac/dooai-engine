use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("invalid image: {0}")]
    InvalidImage(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("invalid document: {0}")]
    InvalidDocument(String),
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("invalid configuration: {0}")]
    Configuration(String),
    #[error("{0}")]
    UpstreamUnavailable(String),
    #[error("model could not be loaded: {0}")]
    ModelLoad(String),
    #[error("inference failed: {0}")]
    Inference(String),
}
