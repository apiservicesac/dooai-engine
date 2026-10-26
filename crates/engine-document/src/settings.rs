//! The document reader's fixed values in one place.

/// What a file must start with to be read as a PDF.
pub struct FileSignatures;

impl FileSignatures {
    pub const PDF: &'static [u8] = b"%PDF-";
}

/// How PDF pages are read.
pub struct PdfSettings;

impl PdfSettings {
    /// More pages than this are refused: reading them all would take minutes.
    pub const MAX_PAGES: usize = 30;
    /// A page whose text layer has at least this many non-blank characters is read from it, not by OCR.
    pub const MIN_TEXT_LAYER_CHARACTERS: usize = 20;
    /// Resolution at which pages without a text layer are drawn before OCR.
    pub const RENDER_DPI: f32 = 200.0;
    pub const POINTS_PER_INCH: f32 = 72.0;
    /// Pages are never drawn larger than this many pixels on a side.
    pub const MAX_RENDER_SIDE: f32 = 4000.0;
    /// Name given to a page read from the text layer of a PDF.
    pub const TEXT_LAYER_ENGINE: &'static str = "pdf-text-layer";
}
