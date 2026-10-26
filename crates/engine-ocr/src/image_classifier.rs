use std::path::Path;

use engine_core::EngineError;
use image::RgbImage;
use ndarray::Ix2;
use ort::environment::Environment;

use crate::classifier_input::ClassifierInput;
use crate::onnx_model::OnnxModel;
use crate::settings::ClassifierSettings;
use crate::tensor_builder::{ChannelOrder, TensorBuilder};

/// The most likely class of an image and how sure the network is.
#[derive(Debug, Clone, Copy)]
pub struct Classification {
    pub class_index: usize,
    pub score: f32,
}

/// An ONNX image classifier whose output is one probability per class.
pub struct ImageClassifier {
    model: OnnxModel,
    input: ClassifierInput,
    tensor_builder: TensorBuilder,
}

impl ImageClassifier {
    pub fn load(environment: &Environment, model_path: &Path, input: ClassifierInput) -> Result<Self, EngineError> {
        Ok(Self {
            model: OnnxModel::load(environment, model_path)?,
            input,
            tensor_builder: TensorBuilder::new(
                ChannelOrder::RedGreenBlue,
                ClassifierSettings::MEAN,
                ClassifierSettings::STANDARD_DEVIATION,
            ),
        })
    }

    pub fn classify(&self, image: &RgbImage) -> Result<Classification, EngineError> {
        self.rank(image)?
            .into_iter()
            .next()
            .ok_or_else(|| EngineError::Inference("the classifier returned no classes".to_string()))
    }

    /// Every class, most likely first.
    pub fn rank(&self, image: &RgbImage) -> Result<Vec<Classification>, EngineError> {
        let prepared = self.input.prepare(image);
        let output = self.model.run(self.tensor_builder.build(&prepared))?;
        let probabilities = output
            .into_dimensionality::<Ix2>()
            .map_err(|error| EngineError::Inference(format!("unexpected classifier output: {error}")))?;
        let mut classes: Vec<Classification> = probabilities
            .row(0)
            .iter()
            .enumerate()
            .map(|(class_index, &score)| Classification { class_index, score })
            .collect();
        classes.sort_by(|first, second| second.score.total_cmp(&first.score));
        Ok(classes)
    }
}
