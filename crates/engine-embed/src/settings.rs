//! The embedding model's fixed values in one place.

/// Where the model lives under the models directory.
pub struct EmbeddingFiles;

impl EmbeddingFiles {
    pub const FOLDER: &'static str = "embedding";
    pub const MODEL: &'static str = "model.onnx";
    pub const TOKENIZER: &'static str = "tokenizer.json";
}

/// multilingual-e5-small, by Microsoft Research (MIT): 100 languages, 384 dimensions.
pub struct EmbeddingModel;

impl EmbeddingModel {
    pub const NAME: &'static str = "multilingual-e5-small";
    pub const DIMENSIONS: usize = 384;
}

/// How texts are prepared for the model.
pub struct EmbeddingInput;

impl EmbeddingInput {
    /// Longer texts are cut to this many tokens.
    pub const MAX_TOKENS: usize = 512;
    /// E5 expects a prefix. "query: " is the one for symmetric tasks such as similarity and classification.
    pub const TEXT_PREFIX: &'static str = "query: ";
    pub const PAD_TOKEN: &'static str = "<pad>";
    /// Texts are embedded in batches of this size, to bound memory.
    pub const BATCH_SIZE: usize = 16;
}
