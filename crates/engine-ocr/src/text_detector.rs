use std::path::Path;

use engine_core::{BoundingBox, EngineError};
use image::RgbImage;
use image::imageops::{self, FilterType};
use ndarray::Ix4;
use ort::environment::Environment;

use crate::box_extractor::BoxExtractor;
use crate::detector_settings::DetectorSettings;
use crate::onnx_model::OnnxModel;
use crate::settings::DetectorInput;
use crate::tensor_builder::TensorBuilder;

/// Finds where the text is: one box per text line, in the original image coordinates.
pub struct TextDetector {
    model: OnnxModel,
    tensor_builder: TensorBuilder,
    box_extractor: BoxExtractor,
}

impl TextDetector {
    pub fn load(environment: &Environment, model_path: &Path, settings: DetectorSettings) -> Result<Self, EngineError> {
        Ok(Self {
            model: OnnxModel::load(environment, model_path)?,
            tensor_builder: TensorBuilder::new(DetectorInput::MEAN, DetectorInput::STANDARD_DEVIATION),
            box_extractor: BoxExtractor::new(settings),
        })
    }

    pub fn detect(&self, image: &RgbImage) -> Result<Vec<BoundingBox>, EngineError> {
        let (resized_width, resized_height) = Self::network_size(image.width(), image.height());
        let resized = imageops::resize(image, resized_width, resized_height, FilterType::Triangle);

        let output = self.model.run(self.tensor_builder.build(&resized))?;
        let probability_maps = output
            .into_dimensionality::<Ix4>()
            .map_err(|error| EngineError::Inference(format!("unexpected detector output: {error}")))?;
        let probability_map = probability_maps.index_axis(ndarray::Axis(0), 0);
        let probability_map = probability_map.index_axis(ndarray::Axis(0), 0);

        let horizontal_factor = image.width() as f32 / resized_width as f32;
        let vertical_factor = image.height() as f32 / resized_height as f32;
        Ok(self
            .box_extractor
            .extract(probability_map)
            .into_iter()
            .map(|text_box| {
                text_box
                    .scaled(horizontal_factor, vertical_factor)
                    .clamped(image.width() as f32, image.height() as f32)
            })
            .collect())
    }

    /// Shrinks large images to the maximum side and rounds both sides to the network's multiple.
    fn network_size(width: u32, height: u32) -> (u32, u32) {
        let scale = (DetectorInput::MAX_SIDE as f32 / width.max(height) as f32).min(1.0);
        (
            Self::round_to_multiple(width as f32 * scale),
            Self::round_to_multiple(height as f32 * scale),
        )
    }

    fn round_to_multiple(side: f32) -> u32 {
        let multiple = DetectorInput::SIDE_MULTIPLE as f32;
        ((side / multiple).round() as u32).max(1) * DetectorInput::SIDE_MULTIPLE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn large_images_are_scaled_down_to_a_multiple_of_32() {
        assert_eq!(TextDetector::network_size(1920, 1080), (960, 544));
    }

    #[test]
    fn small_images_are_not_scaled_up() {
        assert_eq!(TextDetector::network_size(100, 40), (96, 32));
    }
}
