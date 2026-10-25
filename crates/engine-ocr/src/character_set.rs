use std::fs;
use std::path::Path;

use engine_core::EngineError;
use serde::Deserialize;

use crate::settings::Decoding;

/// Output classes of the recognizer: the CTC blank, the dictionary, then the space.
#[derive(Debug, Clone)]
pub struct CharacterSet {
    symbols: Vec<String>,
}

#[derive(Deserialize)]
struct RecognizerConfigFile {
    #[serde(rename = "PostProcess")]
    post_process: PostProcessSection,
}

#[derive(Deserialize)]
struct PostProcessSection {
    character_dict: Vec<String>,
}

impl CharacterSet {
    pub fn from_dictionary(dictionary: Vec<String>) -> Self {
        let mut symbols = Vec::with_capacity(dictionary.len() + 2);
        symbols.push(Decoding::BLANK_SYMBOL.to_string());
        symbols.extend(dictionary);
        symbols.push(Decoding::SPACE_SYMBOL.to_string());
        Self { symbols }
    }

    pub fn from_config_file(path: &Path) -> Result<Self, EngineError> {
        let content = fs::read_to_string(path)
            .map_err(|error| EngineError::Configuration(format!("{}: {error}", path.display())))?;
        let config: RecognizerConfigFile = serde_yaml_ng::from_str(&content)
            .map_err(|error| EngineError::Configuration(format!("{}: {error}", path.display())))?;
        Ok(Self::from_dictionary(config.post_process.character_dict))
    }

    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    pub fn symbol(&self, class_index: usize) -> Option<&str> {
        self.symbols.get(class_index).map(String::as_str)
    }
}
