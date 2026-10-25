//! The OCR pipeline's fixed values in one place.

/// Files of a PP-OCR model tier, as laid out by `scripts/download-models.sh`.
pub struct ModelFileNames;

impl ModelFileNames {
    pub const DETECTOR_MODEL: &'static str = "det.onnx";
    pub const DETECTOR_CONFIG: &'static str = "det.yml";
    pub const RECOGNIZER_MODEL: &'static str = "rec.onnx";
    pub const RECOGNIZER_CONFIG: &'static str = "rec.yml";
}

/// How an image is prepared for the text detector.
pub struct DetectorInput;

impl DetectorInput {
    /// Longer sides are scaled down to this many pixels; smaller images are never scaled up.
    pub const MAX_SIDE: u32 = 960;
    /// The network needs both sides to be a multiple of this.
    pub const SIDE_MULTIPLE: u32 = 32;
    /// The detector was trained on BGR images normalized with the ImageNet statistics.
    pub const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
    pub const STANDARD_DEVIATION: [f32; 3] = [0.229, 0.224, 0.225];
    /// Detected regions with a shorter side than this are noise.
    pub const MIN_COMPONENT_SIDE: f32 = 3.0;
}

/// How a cropped line is prepared for the text recognizer.
pub struct RecognizerInput;

impl RecognizerInput {
    /// Fixed height of the network input; the width follows the aspect ratio.
    pub const HEIGHT: u32 = 48;
    pub const MIN_WIDTH: u32 = 16;
    pub const MAX_WIDTH: u32 = 3200;
    pub const MEAN: [f32; 3] = [0.5, 0.5, 0.5];
    pub const STANDARD_DEVIATION: [f32; 3] = [0.5, 0.5, 0.5];
}

/// How the recognizer's output becomes text.
pub struct Decoding;

impl Decoding {
    /// Class of the CTC "no character" symbol.
    pub const BLANK_INDEX: usize = 0;
    pub const BLANK_SYMBOL: &'static str = "";
    pub const SPACE_SYMBOL: &'static str = " ";
    /// Lines recognized with less confidence than this are dropped.
    pub const MIN_LINE_CONFIDENCE: f32 = 0.5;
}
