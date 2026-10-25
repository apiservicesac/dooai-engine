use serde::Serialize;

use crate::BoundingBox;

#[derive(Debug, Clone, Serialize)]
pub struct OcrLine {
    pub text: String,
    pub confidence: f32,
    pub bounding_box: BoundingBox,
}
