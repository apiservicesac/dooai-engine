use std::sync::Arc;

use engine_core::{EngineError, OcrDocument, OcrEngine, OcrOptions, ReadDocument, ReadPage};

use crate::pdf_rasterizer::PdfRasterizer;
use crate::pdf_text_layer::PdfTextLayer;
use crate::settings::{FileSignatures, PdfSettings};

/// Reads a file of any supported kind: an image goes straight to OCR; a PDF is read page by page,
/// from its text layer when it has one and by OCR when it does not.
pub struct DocumentReader {
    ocr_engine: Arc<dyn OcrEngine>,
}

impl DocumentReader {
    pub fn new(ocr_engine: Arc<dyn OcrEngine>) -> Self {
        Self { ocr_engine }
    }

    pub fn read(&self, file: &[u8], options: &OcrOptions) -> Result<ReadDocument, EngineError> {
        if file.starts_with(FileSignatures::PDF) {
            return self.read_pdf(file, options);
        }
        Ok(ReadDocument::single_page(self.ocr_engine.read(file, options)?))
    }

    fn read_pdf(&self, file: &[u8], options: &OcrOptions) -> Result<ReadDocument, EngineError> {
        let text_layer = PdfTextLayer::open(file)?;
        let page_count = text_layer.page_count()?;
        if page_count == 0 || page_count > PdfSettings::MAX_PAGES {
            return Err(EngineError::InvalidDocument(format!(
                "the PDF has {page_count} pages; between 1 and {} are accepted",
                PdfSettings::MAX_PAGES
            )));
        }

        let mut rasterizer: Option<PdfRasterizer> = None;
        let mut pages = Vec::with_capacity(page_count);
        for page_index in 0..page_count {
            let content = match self.text_of(&text_layer, page_index) {
                Some(text) => OcrDocument::from_text(text, PdfSettings::TEXT_LAYER_ENGINE),
                None => {
                    let rasterizer = match &rasterizer {
                        Some(open) => open,
                        None => rasterizer.insert(PdfRasterizer::open(file)?),
                    };
                    self.ocr_engine.read(&rasterizer.render_png(page_index)?, options)?
                }
            };
            pages.push(ReadPage {
                page: page_index + 1,
                content,
            });
        }
        Ok(ReadDocument::from_pages(pages))
    }

    /// The text layer of a page, when it holds enough text to trust it over OCR.
    fn text_of(&self, text_layer: &PdfTextLayer, page_index: usize) -> Option<String> {
        let text = match text_layer.page_text(page_index) {
            Ok(text) => text,
            Err(error) => {
                tracing::warn!(%error, "text layer unreadable; the page will be read by OCR");
                return None;
            }
        };
        let characters = text.chars().filter(|character| !character.is_whitespace()).count();
        (characters >= PdfSettings::MIN_TEXT_LAYER_CHARACTERS).then_some(text)
    }
}
