use std::sync::Arc;

use crate::{EngineError, OcrDocument, OcrEngine, OcrOptions};

/// Reads with a fast engine first and escalates to a slower, stronger one when the fast one is unsure.
pub struct FallbackOcrEngine {
    primary: Arc<dyn OcrEngine>,
    fallback: Arc<dyn OcrEngine>,
    minimum_confidence: f32,
}

impl FallbackOcrEngine {
    pub fn new(primary: Arc<dyn OcrEngine>, fallback: Arc<dyn OcrEngine>, minimum_confidence: f32) -> Self {
        Self {
            primary,
            fallback,
            minimum_confidence,
        }
    }

    fn is_confident(&self, document: &OcrDocument) -> bool {
        document
            .confidence
            .is_some_and(|confidence| confidence >= self.minimum_confidence)
    }
}

impl OcrEngine for FallbackOcrEngine {
    fn read(&self, encoded_image: &[u8], options: &OcrOptions) -> Result<OcrDocument, EngineError> {
        let first_attempt = self.primary.read(encoded_image, options)?;
        if self.is_confident(&first_attempt) {
            return Ok(first_attempt);
        }
        match self.fallback.read(encoded_image, options) {
            Ok(second_attempt) => Ok(second_attempt),
            Err(error) => {
                tracing::warn!(%error, "fallback engine failed; answering with the first reading");
                Ok(first_attempt)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubEngine {
        name: &'static str,
        confidence: Option<f32>,
        fails: bool,
    }

    impl OcrEngine for StubEngine {
        fn read(&self, _: &[u8], _: &OcrOptions) -> Result<OcrDocument, EngineError> {
            if self.fails {
                return Err(EngineError::Inference("down".to_string()));
            }
            let mut document = OcrDocument::from_text(self.name.to_string(), self.name);
            document.confidence = self.confidence;
            Ok(document)
        }
    }

    fn engine(primary_confidence: Option<f32>, fallback_fails: bool) -> FallbackOcrEngine {
        FallbackOcrEngine::new(
            Arc::new(StubEngine {
                name: "primary",
                confidence: primary_confidence,
                fails: false,
            }),
            Arc::new(StubEngine {
                name: "fallback",
                confidence: None,
                fails: fallback_fails,
            }),
            0.8,
        )
    }

    fn read_with(engine: &FallbackOcrEngine) -> String {
        engine.read(b"", &OcrOptions::default()).unwrap().engine
    }

    #[test]
    fn keeps_a_confident_first_reading() {
        assert_eq!(read_with(&engine(Some(0.95), false)), "primary");
    }

    #[test]
    fn escalates_when_the_first_reading_is_unsure_or_empty() {
        assert_eq!(read_with(&engine(Some(0.5), false)), "fallback");
        assert_eq!(read_with(&engine(None, false)), "fallback");
    }

    #[test]
    fn answers_with_the_first_reading_when_the_fallback_is_down() {
        assert_eq!(read_with(&engine(Some(0.5), true)), "primary");
    }
}
