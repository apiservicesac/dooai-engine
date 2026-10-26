use std::path::Path;

use engine_core::{EngineError, TextEmbedder};
use engine_onnx::OnnxModel;
use ndarray::Ix3;

use crate::mean_pooling::MeanPooling;
use crate::sentence_tokenizer::SentenceTokenizer;
use crate::settings::{EmbeddingFiles, EmbeddingInput, EmbeddingModel};

/// multilingual-e5-small on ONNX Runtime: mean-pooled, unit-length vectors of 384 dimensions.
pub struct E5Embedder {
    tokenizer: SentenceTokenizer,
    model: OnnxModel,
}

impl E5Embedder {
    pub fn load(models_directory: &Path) -> Result<Self, EngineError> {
        let folder = models_directory.join(EmbeddingFiles::FOLDER);
        Ok(Self {
            tokenizer: SentenceTokenizer::load(&folder.join(EmbeddingFiles::TOKENIZER))?,
            model: OnnxModel::load(&folder.join(EmbeddingFiles::MODEL))?,
        })
    }

    pub fn is_installed(models_directory: &Path) -> bool {
        models_directory
            .join(EmbeddingFiles::FOLDER)
            .join(EmbeddingFiles::MODEL)
            .is_file()
    }

    fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EngineError> {
        let prefixed = texts
            .iter()
            .map(|text| format!("{}{text}", EmbeddingInput::TEXT_PREFIX))
            .collect();
        let batch = self.tokenizer.encode(prefixed)?;
        let hidden_states = self
            .model
            .run_tokens(batch.token_ids, batch.attention_mask.clone())?
            .into_dimensionality::<Ix3>()
            .map_err(|error| EngineError::Inference(format!("unexpected embedding output: {error}")))?;
        Ok(MeanPooling::pool(hidden_states.view(), batch.attention_mask.view()))
    }
}

impl TextEmbedder for E5Embedder {
    fn model_name(&self) -> &str {
        EmbeddingModel::NAME
    }

    fn dimensions(&self) -> usize {
        EmbeddingModel::DIMENSIONS
    }

    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EngineError> {
        let mut vectors = Vec::with_capacity(texts.len());
        for batch in texts.chunks(EmbeddingInput::BATCH_SIZE) {
            vectors.extend(self.embed_batch(batch)?);
        }
        Ok(vectors)
    }
}
