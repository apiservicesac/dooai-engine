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
    pub const EMBED: &'static str = "/v1/embed";
    pub const SIMILAR: &'static str = "/v1/similar";
}

/// How much one text request may ask for.
pub struct RequestLimits;

impl RequestLimits {
    pub const MAX_TEXTS: usize = 128;
    pub const MAX_CANDIDATES: usize = 5000;
    pub const MAX_TEXT_CHARACTERS: usize = 20_000;
    pub const DEFAULT_TOP_K: usize = 5;
    pub const MAX_TOP_K: usize = 100;
    pub const DEFAULT_MIN_SCORE: f32 = 0.0;
    /// Bodies of the JSON endpoints, in bytes: 5000 candidates of a few hundred characters fit.
    pub const MAX_JSON_BYTES: usize = 8 * 1024 * 1024;
}
