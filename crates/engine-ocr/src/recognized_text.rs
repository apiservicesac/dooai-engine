#[derive(Debug, Clone, PartialEq)]
pub struct RecognizedText {
    pub text: String,
    pub confidence: f32,
}
