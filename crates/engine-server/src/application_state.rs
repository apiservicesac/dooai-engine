use std::sync::Arc;

use engine_core::OcrEngine;

/// What every request handler shares.
#[derive(Clone)]
pub struct ApplicationState {
    pub ocr_engine: Arc<dyn OcrEngine>,
}
