//! The document model's fixed values in one place.

/// How the lines of an [`OcrDocument`](crate::OcrDocument) are joined into text.
pub struct DocumentLayout;

impl DocumentLayout {
    /// Between lines that share a visual row, so table columns stay apart.
    pub const COLUMN_SEPARATOR: &'static str = "\t";
    pub const ROW_SEPARATOR: &'static str = "\n";
}
