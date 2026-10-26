use std::path::Path;

use engine_core::EngineError;
use ndarray::Array2;
use tokenizers::{PaddingParams, PaddingStrategy, Tokenizer, TruncationParams};

use crate::settings::EmbeddingInput;

/// A batch of texts as the network reads it: token ids and an attention mask, padded to the longest text.
pub struct TokenBatch {
    pub token_ids: Array2<i64>,
    pub attention_mask: Array2<i64>,
}

pub struct SentenceTokenizer {
    tokenizer: Tokenizer,
}

impl SentenceTokenizer {
    pub fn load(path: &Path) -> Result<Self, EngineError> {
        let mut tokenizer = Tokenizer::from_file(path)
            .map_err(|error| EngineError::ModelLoad(format!("{}: {error}", path.display())))?;
        let pad_id = tokenizer
            .token_to_id(EmbeddingInput::PAD_TOKEN)
            .ok_or_else(|| EngineError::ModelLoad("the tokenizer has no padding token".to_string()))?;
        tokenizer
            .with_truncation(Some(TruncationParams {
                max_length: EmbeddingInput::MAX_TOKENS,
                ..TruncationParams::default()
            }))
            .map_err(|error| EngineError::ModelLoad(error.to_string()))?;
        tokenizer.with_padding(Some(PaddingParams {
            strategy: PaddingStrategy::BatchLongest,
            pad_id,
            pad_token: EmbeddingInput::PAD_TOKEN.to_string(),
            ..PaddingParams::default()
        }));
        Ok(Self { tokenizer })
    }

    pub fn encode(&self, texts: Vec<String>) -> Result<TokenBatch, EngineError> {
        let encodings = self
            .tokenizer
            .encode_batch(texts, true)
            .map_err(|error| EngineError::Inference(format!("tokenizing failed: {error}")))?;
        let rows = encodings.len();
        let columns = encodings.first().map_or(0, |encoding| encoding.get_ids().len());
        let flatten = |values: fn(&tokenizers::Encoding) -> &[u32]| -> Vec<i64> {
            encodings
                .iter()
                .flat_map(|encoding| values(encoding).iter().map(|&value| i64::from(value)))
                .collect()
        };
        let shape_error = |error: ndarray::ShapeError| EngineError::Inference(error.to_string());
        Ok(TokenBatch {
            token_ids: Array2::from_shape_vec((rows, columns), flatten(|encoding| encoding.get_ids()))
                .map_err(shape_error)?,
            attention_mask: Array2::from_shape_vec((rows, columns), flatten(|encoding| encoding.get_attention_mask()))
                .map_err(shape_error)?,
        })
    }
}
