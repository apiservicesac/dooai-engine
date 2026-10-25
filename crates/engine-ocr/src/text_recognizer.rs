use std::path::Path;

use engine_core::EngineError;
use image::RgbImage;
use image::imageops::{self, FilterType};
use ndarray::{Axis, Ix3};
use ort::environment::Environment;

use crate::character_set::CharacterSet;
use crate::ctc_decoder::CtcDecoder;
use crate::onnx_model::OnnxModel;
use crate::recognized_text::RecognizedText;
use crate::settings::RecognizerInput;
use crate::tensor_builder::TensorBuilder;

/// Reads the text of one cropped line.
pub struct TextRecognizer {
    model: OnnxModel,
    tensor_builder: TensorBuilder,
    decoder: CtcDecoder,
}

impl TextRecognizer {
    pub fn load(
        environment: &Environment,
        model_path: &Path,
        character_set: CharacterSet,
    ) -> Result<Self, EngineError> {
        Ok(Self {
            model: OnnxModel::load(environment, model_path)?,
            tensor_builder: TensorBuilder::new(RecognizerInput::MEAN, RecognizerInput::STANDARD_DEVIATION),
            decoder: CtcDecoder::new(character_set),
        })
    }

    pub fn recognize(&self, line_image: &RgbImage) -> Result<RecognizedText, EngineError> {
        let width = Self::network_width(line_image.width(), line_image.height());
        let resized = imageops::resize(line_image, width, RecognizerInput::HEIGHT, FilterType::Triangle);

        let output = self.model.run(self.tensor_builder.build(&resized))?;
        let sequences = output
            .into_dimensionality::<Ix3>()
            .map_err(|error| EngineError::Inference(format!("unexpected recognizer output: {error}")))?;
        let probabilities = sequences.index_axis(Axis(0), 0);

        if probabilities.ncols() != self.decoder.class_count() {
            return Err(EngineError::Inference(format!(
                "the recognizer has {} classes but its dictionary has {}",
                probabilities.ncols(),
                self.decoder.class_count()
            )));
        }
        Ok(self.decoder.decode(probabilities))
    }

    /// Keeps the aspect ratio at the fixed network height.
    fn network_width(width: u32, height: u32) -> u32 {
        let scaled_width = (RecognizerInput::HEIGHT as f32 * width as f32 / height as f32).ceil() as u32;
        scaled_width.clamp(RecognizerInput::MIN_WIDTH, RecognizerInput::MAX_WIDTH)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_follows_the_aspect_ratio_within_limits() {
        assert_eq!(TextRecognizer::network_width(200, 24), 400);
        assert_eq!(TextRecognizer::network_width(1, 100), RecognizerInput::MIN_WIDTH);
        assert_eq!(TextRecognizer::network_width(100_000, 20), RecognizerInput::MAX_WIDTH);
    }
}
