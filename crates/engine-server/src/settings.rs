//! The server's fixed values in one place.

/// Environment variables that configure the server.
pub struct EnvironmentVariables;

impl EnvironmentVariables {
    pub const ADDRESS: &'static str = "ENGINE_ADDRESS";
    pub const MODELS_DIRECTORY: &'static str = "ENGINE_MODELS_DIR";
    pub const MAX_UPLOAD_BYTES: &'static str = "ENGINE_MAX_UPLOAD_BYTES";
    /// A JSON file with the kinds of document to recognize and the words that name them; without it the built-in
    /// lexicon is used.
    pub const LEXICON_FILE: &'static str = "ENGINE_LEXICON_FILE";
    /// A language model that speaks the OpenAI chat completions API turns on `/v1/extract`.
    pub const LLM_URL: &'static str = "ENGINE_LLM_URL";
    pub const LLM_MODEL: &'static str = "ENGINE_LLM_MODEL";
    pub const LLM_API_KEY: &'static str = "ENGINE_LLM_API_KEY";
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
    pub const CLASSIFY: &'static str = "/v1/classify";
    pub const EXTRACT: &'static str = "/v1/extract";
}

/// How much one text request may ask for.
pub struct RequestLimits;

impl RequestLimits {
    pub const MAX_TEXTS: usize = 128;
    pub const MAX_CANDIDATES: usize = 5000;
    pub const MAX_TEXT_CHARACTERS: usize = 20_000;
    /// A whole document may be sent to classify; only its start is read.
    pub const MAX_CLASSIFY_CHARACTERS: usize = 200_000;
    pub const DEFAULT_TOP_K: usize = 5;
    pub const MAX_TOP_K: usize = 100;
    pub const DEFAULT_MIN_SCORE: f32 = 0.0;
    /// Bodies of the JSON endpoints, in bytes: 5000 candidates of a few hundred characters fit.
    pub const MAX_JSON_BYTES: usize = 8 * 1024 * 1024;
}
