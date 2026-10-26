use std::path::Path;

use engine_core::{EngineError, OcrDocument, OcrEngine, OcrLine, OcrOptions};
use image::RgbImage;

use crate::detector_settings::DetectorSettings;
use crate::document_orientation::DocumentOrientation;
use crate::line_cropper::LineCropper;
use crate::line_orientation::LineOrientation;
use crate::model_layout::ModelLayout;
use crate::recognizer_set::RecognizerSet;
use crate::rotation::Rotation;
use crate::settings::Decoding;
use crate::text_detector::TextDetector;

const ENGINE_NAME: &str = "paddle-ocr";

/// The PP-OCR pipeline: turn the page upright, find the text lines, straighten each one, read it.
pub struct PaddleOcrEngine {
    document_orientation: DocumentOrientation,
    detector: TextDetector,
    line_orientation: LineOrientation,
    recognizers: RecognizerSet,
}

impl PaddleOcrEngine {
    pub fn load(models_directory: &Path) -> Result<Self, EngineError> {
        let layout = ModelLayout::new(models_directory);
        let environment = ort::init()
            .build()
            .map_err(|error| EngineError::ModelLoad(error.to_string()))?;

        let detector_files = layout.detector()?;
        let settings = DetectorSettings::from_config_file(&detector_files.config)?;
        Ok(Self {
            document_orientation: DocumentOrientation::load(&environment, &layout.document_orientation()?.model)?,
            detector: TextDetector::load(&environment, &detector_files.model, settings)?,
            line_orientation: LineOrientation::load(&environment, &layout.line_orientation()?.model)?,
            recognizers: RecognizerSet::load(&environment, &layout)?,
        })
    }

    /// Turns a cropped line around when it is upside down.
    fn upright(&self, line: RgbImage) -> Result<RgbImage, EngineError> {
        if self.line_orientation.is_upside_down(&line)? {
            return Ok(Rotation::HALF_TURN.apply(line));
        }
        Ok(line)
    }
}

impl OcrEngine for PaddleOcrEngine {
    fn read(&self, encoded_image: &[u8], options: &OcrOptions) -> Result<OcrDocument, EngineError> {
        let decoded = image::load_from_memory(encoded_image)
            .map_err(|error| EngineError::InvalidImage(error.to_string()))?
            .to_rgb8();

        let recognizer = self.recognizers.choose(options.language.as_deref())?;
        let rotation = self.document_orientation.upright_rotation(&decoded)?;
        let page = rotation.apply(decoded);

        let mut recognized_lines = Vec::new();
        for quad in self.detector.detect(&page)? {
            let Some(crop) = LineCropper::crop(&page, &quad) else {
                continue;
            };
            let recognized = recognizer.recognize(&self.upright(crop)?)?;
            if !recognized.text.is_empty() && recognized.confidence >= Decoding::MIN_LINE_CONFIDENCE {
                recognized_lines.push(OcrLine::new(recognized.text, recognized.confidence, quad));
            }
        }
        Ok(OcrDocument::from_lines(
            recognized_lines,
            format!("{ENGINE_NAME}/{}", recognizer.name),
            rotation.clockwise_degrees(),
        ))
    }
}
