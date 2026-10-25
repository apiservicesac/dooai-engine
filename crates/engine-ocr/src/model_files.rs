use std::path::{Path, PathBuf};

use engine_core::EngineError;

use crate::settings::ModelFileNames;

/// The four files of a PP-OCR model tier, as laid out by `scripts/download-models.sh`.
#[derive(Debug, Clone)]
pub struct ModelFiles {
    pub detector_model: PathBuf,
    pub detector_config: PathBuf,
    pub recognizer_model: PathBuf,
    pub recognizer_config: PathBuf,
}

impl ModelFiles {
    pub fn in_directory(directory: &Path) -> Result<Self, EngineError> {
        let files = Self {
            detector_model: directory.join(ModelFileNames::DETECTOR_MODEL),
            detector_config: directory.join(ModelFileNames::DETECTOR_CONFIG),
            recognizer_model: directory.join(ModelFileNames::RECOGNIZER_MODEL),
            recognizer_config: directory.join(ModelFileNames::RECOGNIZER_CONFIG),
        };
        files.ensure_all_exist()?;
        Ok(files)
    }

    fn ensure_all_exist(&self) -> Result<(), EngineError> {
        let required = [
            &self.detector_model,
            &self.detector_config,
            &self.recognizer_model,
            &self.recognizer_config,
        ];
        match required.iter().find(|path| !path.is_file()) {
            Some(missing) => Err(EngineError::ModelLoad(format!(
                "missing model file {}; run scripts/download-models.sh",
                missing.display()
            ))),
            None => Ok(()),
        }
    }
}
