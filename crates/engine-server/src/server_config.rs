use std::net::SocketAddr;
use std::path::PathBuf;

use engine_core::EngineError;

use crate::settings::{Defaults, EnvironmentVariables};

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub address: SocketAddr,
    pub models_directory: PathBuf,
    pub max_upload_bytes: usize,
    pub lexicon_file: Option<PathBuf>,
}

impl ServerConfig {
    pub fn from_environment() -> Result<Self, EngineError> {
        let address = Self::read(EnvironmentVariables::ADDRESS, Defaults::ADDRESS);
        let max_upload_bytes = Self::read(EnvironmentVariables::MAX_UPLOAD_BYTES, "");
        let lexicon_file = std::env::var(EnvironmentVariables::LEXICON_FILE)
            .ok()
            .filter(|path| !path.is_empty());
        Ok(Self {
            lexicon_file: lexicon_file.map(PathBuf::from),
            address: address
                .parse()
                .map_err(|error| Self::invalid(EnvironmentVariables::ADDRESS, &address, error))?,
            models_directory: PathBuf::from(Self::read(
                EnvironmentVariables::MODELS_DIRECTORY,
                Defaults::MODELS_DIRECTORY,
            )),
            max_upload_bytes: if max_upload_bytes.is_empty() {
                Defaults::MAX_UPLOAD_BYTES
            } else {
                max_upload_bytes
                    .parse()
                    .map_err(|error| Self::invalid(EnvironmentVariables::MAX_UPLOAD_BYTES, &max_upload_bytes, error))?
            },
        })
    }

    fn read(variable: &str, default: &str) -> String {
        std::env::var(variable).unwrap_or_else(|_| default.to_string())
    }

    fn invalid(variable: &str, value: &str, error: impl std::fmt::Display) -> EngineError {
        EngineError::Configuration(format!("{variable}={value:?}: {error}"))
    }
}
