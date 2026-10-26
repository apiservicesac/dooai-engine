use std::sync::Mutex;

use engine_core::EngineError;
use ort::environment::Environment;

static ENVIRONMENT: Mutex<Option<Environment>> = Mutex::new(None);

/// The ONNX Runtime environment of the process. ONNX Runtime allows exactly one, so every model shares it.
pub struct OnnxEnvironment;

impl OnnxEnvironment {
    pub fn shared() -> Result<Environment, EngineError> {
        let mut environment = ENVIRONMENT
            .lock()
            .map_err(|_| EngineError::ModelLoad("the ONNX Runtime environment lock was poisoned".to_string()))?;
        if environment.is_none() {
            let created = ort::init()
                .build()
                .map_err(|error| EngineError::ModelLoad(error.to_string()))?;
            *environment = Some(created);
        }
        Ok(environment.clone().expect("the environment was just created"))
    }
}
