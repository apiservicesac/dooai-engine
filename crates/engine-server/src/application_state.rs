use std::sync::Arc;

use engine_classify::LexiconClassifier;
use engine_core::TextEmbedder;
use engine_document::DocumentReader;
use engine_extract::LanguageModelExtractor;

/// What every request handler shares.
#[derive(Clone)]
pub struct ApplicationState {
    pub document_reader: Arc<DocumentReader>,
    /// Present when the embedding model is installed.
    pub embedder: Option<Arc<dyn TextEmbedder>>,
    pub classifier: Arc<LexiconClassifier>,
    /// Present when a language model is configured.
    pub extractor: Option<Arc<LanguageModelExtractor>>,
}

impl ApplicationState {
    pub fn capabilities(&self) -> Vec<&'static str> {
        let mut capabilities = vec!["ocr", "classification"];
        if self.embedder.is_some() {
            capabilities.push("embeddings");
        }
        if self.extractor.is_some() {
            capabilities.push("extraction");
        }
        capabilities
    }
}
