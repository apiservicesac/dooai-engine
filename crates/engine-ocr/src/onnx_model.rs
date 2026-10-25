use std::fmt::Display;
use std::path::Path;
use std::sync::Mutex;

use engine_core::EngineError;
use ndarray::{Array4, ArrayD};
use ort::environment::Environment;
use ort::session::Session;
use ort::value::Tensor;

/// One ONNX network with a single `f32` image input and a single `f32` output.
/// Inference takes the session lock, so concurrent callers run one at a time.
pub struct OnnxModel {
    session: Mutex<Session>,
}

impl OnnxModel {
    pub fn load(environment: &Environment, model_path: &Path) -> Result<Self, EngineError> {
        let session = Session::builder(environment)
            .and_then(|mut builder| builder.commit_from_file(model_path))
            .map_err(|error| EngineError::ModelLoad(format!("{}: {error}", model_path.display())))?;
        Ok(Self {
            session: Mutex::new(session),
        })
    }

    pub fn run(&self, input: Array4<f32>) -> Result<ArrayD<f32>, EngineError> {
        let tensor = Tensor::from_array(input).map_err(Self::inference_error)?;
        let mut session = self
            .session
            .lock()
            .map_err(|_| EngineError::Inference("model session lock was poisoned".to_string()))?;
        let outputs = session.run(ort::inputs![tensor]).map_err(Self::inference_error)?;
        let output = outputs[0].try_extract_array::<f32>().map_err(Self::inference_error)?;
        Ok(output.to_owned())
    }

    fn inference_error(error: impl Display) -> EngineError {
        EngineError::Inference(error.to_string())
    }
}
