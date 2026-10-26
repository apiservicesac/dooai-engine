use std::path::{Path, PathBuf};

use engine_core::EngineError;

use crate::settings::ModelFolders;

/// The ONNX graph and the YAML config of one model.
#[derive(Debug, Clone)]
pub struct ModelFiles {
    pub model: PathBuf,
    pub config: PathBuf,
}

impl ModelFiles {
    pub fn in_directory(directory: &Path) -> Result<Self, EngineError> {
        let files = Self {
            model: directory.join(ModelFolders::MODEL_FILE),
            config: directory.join(ModelFolders::CONFIG_FILE),
        };
        if let Some(missing) = [&files.model, &files.config].into_iter().find(|path| !path.is_file()) {
            return Err(EngineError::ModelLoad(format!(
                "missing model file {}; run scripts/download-models.sh",
                missing.display()
            )));
        }
        Ok(files)
    }
}
