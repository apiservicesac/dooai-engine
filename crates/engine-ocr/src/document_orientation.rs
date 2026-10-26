use std::path::Path;

use engine_core::EngineError;
use image::RgbImage;
use ort::environment::Environment;

use crate::classifier_input::ClassifierInput;
use crate::image_classifier::ImageClassifier;
use crate::rotation::Rotation;
use crate::settings::{ClassifierSettings, DocumentTilts};

/// Tells whether a page was photographed sideways or upside down, and how to turn it upright.
pub struct DocumentOrientation {
    classifier: ImageClassifier,
}

impl DocumentOrientation {
    pub fn load(environment: &Environment, model_path: &Path) -> Result<Self, EngineError> {
        let input = ClassifierInput::ShortSideCenterCrop {
            short_side: ClassifierSettings::DOCUMENT_SHORT_SIDE,
            crop_side: ClassifierSettings::DOCUMENT_CROP_SIDE,
        };
        Ok(Self {
            classifier: ImageClassifier::load(environment, model_path, input)?,
        })
    }

    /// The turns that may make the page upright: one when the classifier is sure, the two most likely when it is not.
    /// The caller reads each candidate and keeps the best reading.
    pub fn candidate_rotations(&self, image: &RgbImage) -> Result<Vec<Rotation>, EngineError> {
        let ranking = self.classifier.rank(image)?;
        let confident = match ranking.as_slice() {
            [best, runner_up, ..] => best.score - runner_up.score >= DocumentTilts::MIN_CONFIDENT_MARGIN,
            _ => true,
        };
        let proposed = if confident { &ranking[..1] } else { &ranking[..2] };
        proposed
            .iter()
            .map(|classification| {
                DocumentTilts::CLASSES
                    .get(classification.class_index)
                    .map(|&tilt| Rotation::correcting(tilt))
                    .ok_or_else(|| {
                        EngineError::Inference(format!("unknown orientation class {}", classification.class_index))
                    })
            })
            .collect()
    }
}
