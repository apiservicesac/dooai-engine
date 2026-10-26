use std::sync::Arc;

use engine_core::{EngineError, TextEmbedder, VectorMath};
use serde::{Deserialize, Serialize};

/// A text the caller wants to compare against, with the id the caller knows it by.
#[derive(Debug, Clone, Deserialize)]
pub struct SimilarityCandidate {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SimilarMatch {
    pub id: String,
    /// Cosine similarity, from -1 to 1; above 0.9 the texts usually mean the same.
    pub score: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct RankingOptions {
    pub top_k: usize,
    pub min_score: f32,
}

/// Finds which candidates mean the most like a query: duplicate contacts, similar products, bank lines and invoices.
pub struct TextRanker {
    embedder: Arc<dyn TextEmbedder>,
}

impl TextRanker {
    pub fn new(embedder: Arc<dyn TextEmbedder>) -> Self {
        Self { embedder }
    }

    /// The best candidates, most similar first, each at least `min_score` similar to the query.
    pub fn rank(
        &self,
        query: &str,
        candidates: &[SimilarityCandidate],
        options: RankingOptions,
    ) -> Result<Vec<SimilarMatch>, EngineError> {
        let mut texts = vec![query.to_string()];
        texts.extend(candidates.iter().map(|candidate| candidate.text.clone()));
        let vectors = self.embedder.embed(&texts)?;
        let (query_vector, candidate_vectors) = vectors
            .split_first()
            .ok_or_else(|| EngineError::Inference("the embedder returned no vectors".to_string()))?;

        let mut matches: Vec<SimilarMatch> = candidates
            .iter()
            .zip(candidate_vectors)
            .map(|(candidate, vector)| SimilarMatch {
                id: candidate.id.clone(),
                score: VectorMath::cosine(query_vector, vector),
            })
            .filter(|similar| similar.score >= options.min_score)
            .collect();
        matches.sort_by(|first, second| second.score.total_cmp(&first.score));
        matches.truncate(options.top_k);
        Ok(matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Embeds a text as how many "a" and "b" letters it has, which is enough to tell directions apart.
    struct LetterEmbedder;

    impl TextEmbedder for LetterEmbedder {
        fn model_name(&self) -> &str {
            "letters"
        }

        fn dimensions(&self) -> usize {
            2
        }

        fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EngineError> {
            Ok(texts
                .iter()
                .map(|text| {
                    let count = |letter| text.chars().filter(|&character| character == letter).count() as f32;
                    VectorMath::normalized(&[count('a'), count('b')])
                })
                .collect())
        }
    }

    fn candidate(id: &str, text: &str) -> SimilarityCandidate {
        SimilarityCandidate {
            id: id.to_string(),
            text: text.to_string(),
        }
    }

    #[test]
    fn returns_the_closest_candidates_first_and_respects_the_limits() {
        let ranker = TextRanker::new(Arc::new(LetterEmbedder));
        let candidates = [
            candidate("far", "bbb"),
            candidate("near", "aab"),
            candidate("exact", "aaa"),
        ];

        let matches = ranker
            .rank(
                "aa",
                &candidates,
                RankingOptions {
                    top_k: 2,
                    min_score: 0.0,
                },
            )
            .unwrap();

        assert_eq!(
            matches.iter().map(|similar| similar.id.as_str()).collect::<Vec<_>>(),
            ["exact", "near"]
        );
    }

    #[test]
    fn drops_candidates_below_the_minimum_score() {
        let ranker = TextRanker::new(Arc::new(LetterEmbedder));
        let candidates = [candidate("far", "bbb"), candidate("exact", "aaa")];

        let matches = ranker
            .rank(
                "aa",
                &candidates,
                RankingOptions {
                    top_k: 5,
                    min_score: 0.5,
                },
            )
            .unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, "exact");
    }
}
