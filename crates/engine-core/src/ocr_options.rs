/// What a caller can ask of one OCR request.
#[derive(Debug, Clone, Default)]
pub struct OcrOptions {
    /// A language code such as `ru` or `ko`. `None` (or `auto`) lets the engine choose.
    pub language: Option<String>,
}
