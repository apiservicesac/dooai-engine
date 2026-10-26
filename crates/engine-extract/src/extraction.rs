use serde::Serialize;

use crate::extracted_document::ExtractedDocument;
use crate::validation::Validation;

/// What was read from a document and whether it is consistent.
#[derive(Debug, Clone, Serialize)]
pub struct Extraction {
    pub document: ExtractedDocument,
    pub validation: Validation,
    /// How many times the model was asked: 1, plus one for each repair.
    pub attempts: usize,
}
