use crate::{EngineError, OcrDocument};

/// Reads the text of an encoded image (PNG, JPEG, WebP...).
/// Inference is blocking and CPU heavy, so callers run it on a blocking thread.
pub trait OcrEngine: Send + Sync {
    fn read(&self, encoded_image: &[u8]) -> Result<OcrDocument, EngineError>;
}
