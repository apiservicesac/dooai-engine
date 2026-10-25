use std::fs;
use std::path::Path;

use engine_core::EngineError;
use serde::Deserialize;

/// Post-processing values of the detector, read from the model's own `inference.yml`.
#[derive(Debug, Clone, Copy)]
pub struct DetectorSettings {
    pub threshold: f32,
    pub box_threshold: f32,
    pub unclip_ratio: f32,
}

#[derive(Deserialize)]
struct DetectorConfigFile {
    #[serde(rename = "PostProcess")]
    post_process: PostProcessSection,
}

#[derive(Deserialize)]
struct PostProcessSection {
    #[serde(rename = "thresh")]
    threshold: f32,
    #[serde(rename = "box_thresh")]
    box_threshold: f32,
    unclip_ratio: f32,
}

impl DetectorSettings {
    pub fn from_config_file(path: &Path) -> Result<Self, EngineError> {
        let content = fs::read_to_string(path)
            .map_err(|error| EngineError::Configuration(format!("{}: {error}", path.display())))?;
        let config: DetectorConfigFile = serde_yaml_ng::from_str(&content)
            .map_err(|error| EngineError::Configuration(format!("{}: {error}", path.display())))?;
        Ok(Self {
            threshold: config.post_process.threshold,
            box_threshold: config.post_process.box_threshold,
            unclip_ratio: config.post_process.unclip_ratio,
        })
    }
}
