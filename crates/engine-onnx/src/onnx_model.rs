use std::fmt::Display;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use engine_core::EngineError;
use ndarray::{Array2, Array4, ArrayD};
use ort::session::{Session, SessionOutputs};
use ort::value::Tensor;

use crate::onnx_environment::OnnxEnvironment;

const TOKEN_IDS_INPUT: &str = "input_ids";
const ATTENTION_MASK_INPUT: &str = "attention_mask";
const TOKEN_TYPES_INPUT: &str = "token_type_ids";

/// One ONNX network with a single `f32` output. Inference takes the session lock,
/// so concurrent callers run one at a time.
pub struct OnnxModel {
    session: Mutex<Session>,
}

impl OnnxModel {
    pub fn load(model_path: &Path) -> Result<Self, EngineError> {
        let environment = OnnxEnvironment::shared()?;
        let session = Session::builder(&environment)
            .and_then(|mut builder| builder.commit_from_file(model_path))
            .map_err(|error| EngineError::ModelLoad(format!("{}: {error}", model_path.display())))?;
        Ok(Self {
            session: Mutex::new(session),
        })
    }

    /// A network with one image input, `[batch, channels, height, width]`.
    pub fn run_image(&self, input: Array4<f32>) -> Result<ArrayD<f32>, EngineError> {
        let tensor = Tensor::from_array(input).map_err(Self::inference_error)?;
        let mut session = self.session()?;
        let outputs = session.run(ort::inputs![tensor]).map_err(Self::inference_error)?;
        Self::first_output(&outputs)
    }

    /// A BERT-style text network: token ids and an attention mask, both `[batch, tokens]`.
    /// Every text is a single sentence, so its token types are all zero.
    pub fn run_tokens(&self, token_ids: Array2<i64>, attention_mask: Array2<i64>) -> Result<ArrayD<f32>, EngineError> {
        let token_types =
            Tensor::from_array(Array2::<i64>::zeros(token_ids.raw_dim())).map_err(Self::inference_error)?;
        let token_ids = Tensor::from_array(token_ids).map_err(Self::inference_error)?;
        let attention_mask = Tensor::from_array(attention_mask).map_err(Self::inference_error)?;
        let mut session = self.session()?;
        let outputs = session
            .run(ort::inputs![
                TOKEN_IDS_INPUT => token_ids,
                ATTENTION_MASK_INPUT => attention_mask,
                TOKEN_TYPES_INPUT => token_types
            ])
            .map_err(Self::inference_error)?;
        Self::first_output(&outputs)
    }

    fn session(&self) -> Result<MutexGuard<'_, Session>, EngineError> {
        self.session
            .lock()
            .map_err(|_| EngineError::Inference("model session lock was poisoned".to_string()))
    }

    fn first_output(outputs: &SessionOutputs<'_>) -> Result<ArrayD<f32>, EngineError> {
        let output = outputs[0].try_extract_array::<f32>().map_err(Self::inference_error)?;
        Ok(output.to_owned())
    }

    fn inference_error(error: impl Display) -> EngineError {
        EngineError::Inference(error.to_string())
    }
}
