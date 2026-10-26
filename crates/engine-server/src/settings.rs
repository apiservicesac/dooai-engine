//! The server's fixed values in one place.

/// Environment variables that configure the server.
pub struct EnvironmentVariables;

impl EnvironmentVariables {
    pub const ADDRESS: &'static str = "ENGINE_ADDRESS";
    pub const MODELS_DIRECTORY: &'static str = "ENGINE_MODELS_DIR";
    pub const MAX_UPLOAD_BYTES: &'static str = "ENGINE_MAX_UPLOAD_BYTES";
}

/// What the server uses when a variable is not set.
pub struct Defaults;

impl Defaults {
    pub const ADDRESS: &'static str = "0.0.0.0:8080";
    pub const MODELS_DIRECTORY: &'static str = "models";
    pub const MAX_UPLOAD_BYTES: usize = 20 * 1024 * 1024;
}

pub struct ApiPaths;

impl ApiPaths {
    pub const HEALTH: &'static str = "/health";
    pub const OCR: &'static str = "/v1/ocr";
}
