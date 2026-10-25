use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("invalid image: {0}")]
    InvalidImage(String),
    #[error("invalid configuration: {0}")]
    Configuration(String),
    #[error("model could not be loaded: {0}")]
    ModelLoad(String),
    #[error("inference failed: {0}")]
    Inference(String),
}
