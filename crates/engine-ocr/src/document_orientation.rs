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

    pub fn upright_rotation(&self, image: &RgbImage) -> Result<Rotation, EngineError> {
        let classification = self.classifier.classify(image)?;
        let tilt = DocumentTilts::CLASSES
            .get(classification.class_index)
            .copied()
            .ok_or_else(|| {
                EngineError::Inference(format!("unknown orientation class {}", classification.class_index))
            })?;
        Ok(Rotation::correcting(tilt))
    }
}
