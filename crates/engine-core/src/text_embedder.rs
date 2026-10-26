use crate::EngineError;

/// Turns texts into vectors where similar meanings sit close together, whatever the language.
/// Inference is blocking and CPU heavy, so callers run it on a blocking thread.
pub trait TextEmbedder: Send + Sync {
    fn model_name(&self) -> &str;

    fn dimensions(&self) -> usize;

    /// One unit-length vector per text, in the same order.
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EngineError>;
}
