use serde::Serialize;

use crate::{BoundingBox, TextQuad};

#[derive(Debug, Clone, Serialize)]
pub struct OcrLine {
    pub text: String,
    pub confidence: f32,
    pub polygon: TextQuad,
    pub bounding_box: BoundingBox,
}

impl OcrLine {
    pub fn new(text: String, confidence: f32, polygon: TextQuad) -> Self {
        Self {
            text,
            confidence,
            bounding_box: polygon.bounding_box(),
            polygon,
        }
    }
}
