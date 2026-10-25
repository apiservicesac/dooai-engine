use std::path::Path;

use engine_core::{BoundingBox, EngineError, OcrDocument, OcrEngine, OcrLine};
use image::RgbImage;
use image::imageops;

use crate::character_set::CharacterSet;
use crate::detector_settings::DetectorSettings;
use crate::model_files::ModelFiles;
use crate::settings::Decoding;
use crate::text_detector::TextDetector;
use crate::text_recognizer::TextRecognizer;

/// PP-OCR pipeline: detect the text lines, then recognize each one.
pub struct PaddleOcrEngine {
    detector: TextDetector,
    recognizer: TextRecognizer,
}

impl PaddleOcrEngine {
    pub fn load(models_directory: &Path) -> Result<Self, EngineError> {
        let files = ModelFiles::in_directory(models_directory)?;
        let settings = DetectorSettings::from_config_file(&files.detector_config)?;
        let character_set = CharacterSet::from_config_file(&files.recognizer_config)?;
        let environment = ort::init()
            .build()
            .map_err(|error| EngineError::ModelLoad(error.to_string()))?;
        Ok(Self {
            detector: TextDetector::load(&environment, &files.detector_model, settings)?,
            recognizer: TextRecognizer::load(&environment, &files.recognizer_model, character_set)?,
        })
    }

    fn crop_line(image: &RgbImage, bounding_box: &BoundingBox) -> Option<RgbImage> {
        let left = bounding_box.left.floor() as u32;
        let top = bounding_box.top.floor() as u32;
        let width = (bounding_box.right.ceil() as u32).saturating_sub(left);
        let height = (bounding_box.bottom.ceil() as u32).saturating_sub(top);
        if width == 0 || height == 0 {
            return None;
        }
        Some(imageops::crop_imm(image, left, top, width, height).to_image())
    }
}

impl OcrEngine for PaddleOcrEngine {
    fn read(&self, encoded_image: &[u8]) -> Result<OcrDocument, EngineError> {
        let image = image::load_from_memory(encoded_image)
            .map_err(|error| EngineError::InvalidImage(error.to_string()))?
            .to_rgb8();

        let mut lines = Vec::new();
        for bounding_box in self.detector.detect(&image)? {
            let Some(line_image) = Self::crop_line(&image, &bounding_box) else {
                continue;
            };
            let recognized = self.recognizer.recognize(&line_image)?;
            if recognized.text.is_empty() || recognized.confidence < Decoding::MIN_LINE_CONFIDENCE {
                continue;
            }
            lines.push(OcrLine {
                text: recognized.text,
                confidence: recognized.confidence,
                bounding_box,
            });
        }
        Ok(OcrDocument::from_lines(lines))
    }
}
