use std::fs;
use std::path::{Path, PathBuf};

use engine_core::EngineError;

use crate::model_files::ModelFiles;
use crate::settings::ModelFolders;

/// Finds every model under the models directory.
pub struct ModelLayout {
    root: PathBuf,
}

impl ModelLayout {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    pub fn detector(&self) -> Result<ModelFiles, EngineError> {
        ModelFiles::in_directory(&self.root.join(ModelFolders::DETECTOR))
    }

    pub fn document_orientation(&self) -> Result<ModelFiles, EngineError> {
        ModelFiles::in_directory(&self.root.join(ModelFolders::DOCUMENT_ORIENTATION))
    }

    pub fn line_orientation(&self) -> Result<ModelFiles, EngineError> {
        ModelFiles::in_directory(&self.root.join(ModelFolders::LINE_ORIENTATION))
    }

    /// Every installed recognizer with its name, the general one first. The general one must exist.
    pub fn recognizers(&self) -> Result<Vec<(String, ModelFiles)>, EngineError> {
        let directory = self.root.join(ModelFolders::RECOGNITION);
        let general = ModelFiles::in_directory(&directory.join(ModelFolders::GENERAL_RECOGNIZER))?;
        let mut recognizers = vec![(ModelFolders::GENERAL_RECOGNIZER.to_string(), general)];

        let entries = fs::read_dir(&directory)
            .map_err(|error| EngineError::ModelLoad(format!("{}: {error}", directory.display())))?;
        let mut names: Vec<String> = entries
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|name| name != ModelFolders::GENERAL_RECOGNIZER)
            .collect();
        names.sort();
        for name in names {
            let files = ModelFiles::in_directory(&directory.join(&name))?;
            recognizers.push((name, files));
        }
        Ok(recognizers)
    }
}
