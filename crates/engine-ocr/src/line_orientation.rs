use std::path::Path;

use engine_core::EngineError;
use image::RgbImage;

use crate::classifier_input::ClassifierInput;
use crate::image_classifier::ImageClassifier;
use crate::settings::ClassifierSettings;

const UPSIDE_DOWN_CLASS: usize = 1;

/// Tells whether a single cropped line is upside down.
pub struct LineOrientation {
    classifier: ImageClassifier,
}

impl LineOrientation {
    pub fn load(model_path: &Path) -> Result<Self, EngineError> {
        let input = ClassifierInput::Fixed {
            width: ClassifierSettings::LINE_WIDTH,
            height: ClassifierSettings::LINE_HEIGHT,
        };
        Ok(Self {
            classifier: ImageClassifier::load(model_path, input)?,
        })
    }

    pub fn is_upside_down(&self, line: &RgbImage) -> Result<bool, EngineError> {
        Ok(self.classifier.classify(line)?.class_index == UPSIDE_DOWN_CLASS)
    }
}
