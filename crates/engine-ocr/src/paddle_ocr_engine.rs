use std::path::Path;

use engine_core::{EngineError, OcrDocument, OcrEngine, OcrLine, OcrOptions};
use image::RgbImage;

use crate::detector_settings::DetectorSettings;
use crate::document_orientation::DocumentOrientation;
use crate::line_cropper::LineCropper;
use crate::line_orientation::LineOrientation;
use crate::model_layout::ModelLayout;
use crate::recognizer_set::{NamedRecognizer, RecognizerSet};
use crate::rotation::Rotation;
use crate::settings::{Decoding, PageVote};
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

    /// Reads the page after turning it by `rotation`; the geometry is mapped back onto the original image.
    fn read_turned(
        &self,
        original: &RgbImage,
        rotation: Rotation,
        recognizer: &NamedRecognizer,
    ) -> Result<PageReading, EngineError> {
        let original_size = (original.width() as f32, original.height() as f32);
        let page = rotation.apply(original.clone());

        let mut recognized_lines = Vec::new();
        let (mut scanned_lines, mut upside_down_lines) = (0, 0);
        for quad in self.detector.detect(&page)? {
            let Some(crop) = LineCropper::crop(&page, &quad) else {
                continue;
            };
            scanned_lines += 1;
            let line = if self.line_orientation.is_upside_down(&crop)? {
                upside_down_lines += 1;
                Rotation::HALF_TURN.apply(crop)
            } else {
                crop
            };
            let recognized = recognizer.recognize(&line)?;
            if !recognized.text.is_empty() && recognized.confidence >= Decoding::MIN_LINE_CONFIDENCE {
                recognized_lines.push(OcrLine::new(recognized.text, recognized.confidence, quad));
            }
        }
        let document = OcrDocument::from_lines(
            recognized_lines,
            format!("{ENGINE_NAME}/{}", recognizer.name),
            rotation.clockwise_degrees(),
        );
        Ok(PageReading {
            document: document.restore_geometry(|point| rotation.restore(point, original_size)),
            scanned_lines,
            upside_down_lines,
        })
    }

    /// Reads the page turned by `rotation`; when most of its lines turn out upside down, the page is upside down,
    /// so it is read again turned a half turn more.
    fn read_with_vote(
        &self,
        original: &RgbImage,
        rotation: Rotation,
        recognizer: &NamedRecognizer,
    ) -> Result<OcrDocument, EngineError> {
        let reading = self.read_turned(original, rotation, recognizer)?;
        if reading.is_mostly_upside_down() {
            return Ok(self
                .read_turned(original, rotation.then(Rotation::HALF_TURN), recognizer)?
                .document);
        }
        Ok(reading.document)
    }
}

/// A page read at one orientation, with what its lines said about that orientation.
struct PageReading {
    document: OcrDocument,
    scanned_lines: usize,
    upside_down_lines: usize,
}

impl PageReading {
    fn is_mostly_upside_down(&self) -> bool {
        self.scanned_lines >= PageVote::MIN_LINES && self.upside_down_lines * 2 > self.scanned_lines
    }
}

impl OcrEngine for PaddleOcrEngine {
    fn read(&self, encoded_image: &[u8], options: &OcrOptions) -> Result<OcrDocument, EngineError> {
        let original = image::load_from_memory(encoded_image)
            .map_err(|error| EngineError::InvalidImage(error.to_string()))?
            .to_rgb8();

        let recognizer = self.recognizers.choose(options.language.as_deref())?;
        let mut best: Option<OcrDocument> = None;
        for rotation in self.document_orientation.candidate_rotations(&original)? {
            let reading = self.read_with_vote(&original, rotation, recognizer)?;
            if best
                .as_ref()
                .is_none_or(|current| reading.reading_evidence() > current.reading_evidence())
            {
                best = Some(reading);
            }
        }
        best.ok_or_else(|| EngineError::Inference("no orientation was proposed".to_string()))
    }
}
