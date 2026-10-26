//! Runs the real model; skipped unless ENGINE_MODELS_DIR points at `scripts/download-models.sh` output.

use std::path::PathBuf;

use engine_core::{TextEmbedder, VectorMath};
use engine_embed::E5Embedder;

const MODELS_DIRECTORY_VARIABLE: &str = "ENGINE_MODELS_DIR";

struct EmbeddingScenario {
    embedder: E5Embedder,
}

impl EmbeddingScenario {
    fn load() -> Option<Self> {
        let directory = PathBuf::from(std::env::var_os(MODELS_DIRECTORY_VARIABLE)?);
        Some(Self {
            embedder: E5Embedder::load(&directory).expect("the embedding model loads"),
        })
    }

    fn similarity(&self, first: &str, second: &str) -> f32 {
        let vectors = self.embedder.embed(&[first.to_string(), second.to_string()]).unwrap();
        VectorMath::cosine(&vectors[0], &vectors[1])
    }
}

macro_rules! scenario {
    () => {
        match EmbeddingScenario::load() {
            Some(scenario) => scenario,
            None => {
                eprintln!("skipped: set {MODELS_DIRECTORY_VARIABLE} to run the model tests");
                return;
            }
        }
    };
}

#[test]
fn gives_one_unit_vector_per_text() {
    let scenario = scenario!();

    let vectors = scenario
        .embedder
        .embed(&["Factura 001".to_string(), "Invoice 001".to_string()])
        .unwrap();

    assert_eq!(vectors.len(), 2);
    assert_eq!(vectors[0].len(), scenario.embedder.dimensions());
    assert!((VectorMath::norm(&vectors[0]) - 1.0).abs() < 1e-4);
}

#[test]
fn understands_the_same_meaning_across_languages() {
    let scenario = scenario!();

    let translated = scenario.similarity("Delivery note for two pallets", "Guía de remisión por dos paletas");
    let unrelated = scenario.similarity("Delivery note for two pallets", "Receta de pastel de manzana");

    assert!(
        translated > unrelated + 0.05,
        "translated {translated}, unrelated {unrelated}"
    );
}

#[test]
fn recognizes_a_contact_written_differently() {
    let scenario = scenario!();

    let same_company = scenario.similarity("Distribuciones Peñalver S.A.", "DISTRIBUCIONES PENALVER SA");
    let other_company = scenario.similarity("Distribuciones Peñalver S.A.", "Comercial Andina S.A.C.");

    assert!(
        same_company > other_company + 0.05,
        "same {same_company}, other {other_company}"
    );
}

#[test]
fn a_batch_gives_the_same_vectors_as_one_text_at_a_time() {
    let scenario = scenario!();
    let texts = [
        "Total a pagar 118.00".to_string(),
        "Una frase bastante más larga para que el relleno sea distinto en el lote".to_string(),
    ];

    let together = scenario.embedder.embed(&texts).unwrap();
    let alone = scenario.embedder.embed(&texts[1..]).unwrap();

    // The model is int8 with dynamic quantization, so the scale depends on the whole batch: a small difference is normal.
    let agreement = VectorMath::cosine(&together[1], &alone[0]);
    assert!(agreement > 0.99, "batch vs alone: {agreement}");
}
