//! Runs the real models; skipped unless ENGINE_MODELS_DIR points at `scripts/download-models.sh` output.

use std::path::PathBuf;

use engine_core::{EngineError, OcrDocument, OcrEngine, OcrOptions};
use engine_ocr::PaddleOcrEngine;

const MODELS_DIRECTORY_VARIABLE: &str = "ENGINE_MODELS_DIR";
const INVOICE: &[u8] = include_bytes!("fixtures/invoice.png");
const INVOICE_TILTED: &[u8] = include_bytes!("fixtures/invoice_tilted.png");
const INVOICE_UPSIDE_DOWN: &[u8] = include_bytes!("fixtures/invoice_upside_down.png");
const INVOICE_RUSSIAN: &[u8] = include_bytes!("fixtures/invoice_ru.png");

struct OcrScenario {
    engine: PaddleOcrEngine,
}

impl OcrScenario {
    fn load() -> Option<Self> {
        let directory = PathBuf::from(std::env::var_os(MODELS_DIRECTORY_VARIABLE)?);
        let engine = PaddleOcrEngine::load(&directory).expect("models load");
        Some(Self { engine })
    }

    fn read(&self, image: &[u8], language: Option<&str>) -> Result<OcrDocument, EngineError> {
        let options = OcrOptions {
            language: language.map(str::to_string),
        };
        self.engine.read(image, &options)
    }

    fn assert_contains(document: &OcrDocument, expected: &[&str]) {
        for text in expected {
            assert!(document.text.contains(text), "missing {text:?} in:\n{}", document.text);
        }
    }
}

macro_rules! scenario {
    () => {
        match OcrScenario::load() {
            Some(scenario) => scenario,
            None => {
                eprintln!("skipped: set {MODELS_DIRECTORY_VARIABLE} to run the model tests");
                return;
            }
        }
    };
}

#[test]
fn reads_the_text_of_an_invoice() {
    let scenario = scenario!();

    let document = scenario.read(INVOICE, None).unwrap();

    OcrScenario::assert_contains(&document, &["2026-0042", "Northwind", "2026-10-25", "118.00"]);
    assert_eq!(document.rotation_degrees, 0);
    assert_eq!(document.text.lines().last(), Some("Total\tUSD 118.00"));
}

#[test]
fn reads_a_tilted_photo() {
    let scenario = scenario!();

    let document = scenario.read(INVOICE_TILTED, None).unwrap();

    OcrScenario::assert_contains(&document, &["2026-0042", "Northwind", "118.00"]);
    assert_eq!(document.text.lines().last(), Some("Total\tUSD 118.00"));
}

#[test]
fn turns_an_upside_down_page_upright() {
    let scenario = scenario!();

    let document = scenario.read(INVOICE_UPSIDE_DOWN, None).unwrap();

    OcrScenario::assert_contains(&document, &["2026-0042", "118.00"]);
    assert_eq!(document.rotation_degrees, 180);
}

#[test]
fn reads_russian_with_the_cyrillic_recognizer() {
    let scenario = scenario!();

    let document = scenario.read(INVOICE_RUSSIAN, Some("ru")).unwrap();

    OcrScenario::assert_contains(&document, &["ФАКТУРА", "Поставщик", "1 500,00"]);
    assert_eq!(document.engine, "paddle-ocr/cyrillic");
}

#[test]
fn rejects_unknown_languages_and_bad_images() {
    let scenario = scenario!();

    assert!(matches!(
        scenario.read(INVOICE, Some("xx")),
        Err(EngineError::UnsupportedLanguage(_))
    ));
    assert!(matches!(
        scenario.read(b"not an image", None),
        Err(EngineError::InvalidImage(_))
    ));
}
