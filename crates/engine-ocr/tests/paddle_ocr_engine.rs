//! Runs the real PP-OCR models; skipped unless ENGINE_MODELS_DIR points at `scripts/download-models.sh` output.

use std::path::PathBuf;

use engine_core::OcrEngine;
use engine_ocr::PaddleOcrEngine;

const MODELS_DIRECTORY_VARIABLE: &str = "ENGINE_MODELS_DIR";
const INVOICE_IMAGE: &[u8] = include_bytes!("fixtures/invoice.png");

struct OcrScenario;

impl OcrScenario {
    fn models_directory() -> Option<PathBuf> {
        let directory = PathBuf::from(std::env::var_os(MODELS_DIRECTORY_VARIABLE)?);
        directory.is_dir().then_some(directory)
    }
}

#[test]
fn reads_the_text_of_an_invoice() {
    let Some(models_directory) = OcrScenario::models_directory() else {
        eprintln!("skipped: set {MODELS_DIRECTORY_VARIABLE} to run the model test");
        return;
    };
    let engine = PaddleOcrEngine::load(&models_directory).expect("models load");

    let document = engine.read(INVOICE_IMAGE).expect("image is read");

    println!("{}\nconfidence {:.3}", document.text, document.confidence);
    for expected in ["2026-0042", "Northwind", "2026-10-25", "118.00"] {
        assert!(
            document.text.contains(expected),
            "missing {expected:?} in:\n{}",
            document.text
        );
    }
}

#[test]
fn rejects_bytes_that_are_not_an_image() {
    let Some(models_directory) = OcrScenario::models_directory() else {
        return;
    };
    let engine = PaddleOcrEngine::load(&models_directory).expect("models load");

    assert!(engine.read(b"not an image").is_err());
}
