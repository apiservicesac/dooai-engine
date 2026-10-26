use engine_core::EngineError;
use pdf_oxide::document::PdfDocument;

/// The text a PDF carries inside, page by page. Digital PDFs have it; scans usually do not.
pub struct PdfTextLayer {
    document: PdfDocument,
}

impl PdfTextLayer {
    pub fn open(bytes: &[u8]) -> Result<Self, EngineError> {
        let document = PdfDocument::from_bytes(bytes.to_vec())
            .map_err(|error| EngineError::InvalidDocument(format!("the PDF cannot be opened: {error}")))?;
        Ok(Self { document })
    }

    pub fn page_count(&self) -> Result<usize, EngineError> {
        self.document
            .page_count()
            .map_err(|error| EngineError::InvalidDocument(format!("the PDF has no readable pages: {error}")))
    }

    /// `page_index` starts at 0.
    pub fn page_text(&self, page_index: usize) -> Result<String, EngineError> {
        self.document.extract_text(page_index).map_err(|error| {
            EngineError::InvalidDocument(format!("page {} has no readable text: {error}", page_index + 1))
        })
    }
}
