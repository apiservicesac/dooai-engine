use std::path::PathBuf;
use std::sync::Arc;

use engine_core::{EngineError, OcrDocument, OcrEngine, OcrOptions, ReadDocument};
use engine_document::DocumentReader;
use engine_ocr::PaddleOcrEngine;

const MODELS_DIRECTORY_VARIABLE: &str = "ENGINE_MODELS_DIR";
const DIGITAL_PDF: &[u8] = include_bytes!("fixtures/digital.pdf");
const SCANNED_PDF: &[u8] = include_bytes!("fixtures/scanned.pdf");
const MIXED_PDF: &[u8] = include_bytes!("fixtures/mixed.pdf");
const LONG_PDF: &[u8] = include_bytes!("fixtures/long.pdf");
const TEXT_LAYER: &str = "pdf-text-layer";

/// An OCR engine that must never be asked to read anything.
struct UnreachableOcrEngine;

impl OcrEngine for UnreachableOcrEngine {
    fn read(&self, _: &[u8], _: &OcrOptions) -> Result<OcrDocument, EngineError> {
        Err(EngineError::InvalidImage("OCR was not expected here".to_string()))
    }
}

struct PdfScenario;

impl PdfScenario {
    fn read_without_ocr(file: &[u8]) -> Result<ReadDocument, EngineError> {
        DocumentReader::new(Arc::new(UnreachableOcrEngine)).read(file, &OcrOptions::default())
    }

    /// A reader backed by the real models, or `None` when they are not installed.
    fn read_with_ocr(file: &[u8]) -> Option<ReadDocument> {
        let directory = PathBuf::from(std::env::var_os(MODELS_DIRECTORY_VARIABLE)?);
        if !directory.is_dir() {
            return None;
        }
        let engine = PaddleOcrEngine::load(&directory).expect("models load");
        Some(
            DocumentReader::new(Arc::new(engine))
                .read(file, &OcrOptions::default())
                .expect("the PDF is read"),
        )
    }
}

#[test]
fn reads_a_digital_pdf_from_its_text_layer_without_ocr() {
    let document = PdfScenario::read_without_ocr(DIGITAL_PDF).unwrap();

    assert_eq!(document.pages.len(), 1);
    assert_eq!(document.pages[0].content.engine, TEXT_LAYER);
    assert!(document.text.contains("Total USD 118.00"), "{}", document.text);
}

#[test]
fn refuses_a_pdf_with_too_many_pages() {
    let error = PdfScenario::read_without_ocr(LONG_PDF).unwrap_err();

    assert!(
        matches!(&error, EngineError::InvalidDocument(message) if message.contains("31 pages")),
        "{error}"
    );
}

#[test]
fn hands_anything_else_to_the_ocr_engine() {
    let error = PdfScenario::read_without_ocr(b"not a document").unwrap_err();

    assert!(matches!(error, EngineError::InvalidImage(_)));
}

#[test]
fn reads_a_scanned_pdf_by_ocr() {
    let Some(document) = PdfScenario::read_with_ocr(SCANNED_PDF) else {
        eprintln!("skipped: set {MODELS_DIRECTORY_VARIABLE} to run the model tests");
        return;
    };

    assert!(document.pages[0].content.engine.starts_with("paddle-ocr"));
    assert!(document.text.contains("118.00"), "{}", document.text);
}

#[test]
fn reads_each_page_of_a_mixed_pdf_its_own_way() {
    let Some(document) = PdfScenario::read_with_ocr(MIXED_PDF) else {
        return;
    };

    assert_eq!(document.pages.len(), 2);
    assert_eq!(document.pages[0].content.engine, TEXT_LAYER);
    assert!(document.pages[1].content.engine.starts_with("paddle-ocr"));
    assert!(
        document.pages[1].page == 2 && document.text.matches("118.00").count() == 2,
        "{}",
        document.text
    );
}
