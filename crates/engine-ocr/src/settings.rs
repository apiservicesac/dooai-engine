//! The OCR pipeline's fixed values in one place.

/// Where each model lives under the models directory: one folder per model, holding `model.onnx` and `model.yml`.
pub struct ModelFolders;

impl ModelFolders {
    pub const MODEL_FILE: &'static str = "model.onnx";
    pub const CONFIG_FILE: &'static str = "model.yml";
    pub const DETECTOR: &'static str = "detection";
    pub const DOCUMENT_ORIENTATION: &'static str = "orientation/document";
    pub const LINE_ORIENTATION: &'static str = "orientation/textline";
    /// Parent of one folder per recognizer, named after its alphabet.
    pub const RECOGNITION: &'static str = "recognition";
    /// The recognizer used when no language is asked for; every installation has it.
    pub const GENERAL_RECOGNIZER: &'static str = "general";
}

/// Which recognizer reads which language. Latin-alphabet languages are read by the general recognizer.
pub struct LanguageScripts;

impl LanguageScripts {
    /// Same as naming no language: the general recognizer reads the page.
    pub const AUTOMATIC: &'static str = "auto";
    pub const TABLE: &'static [(&'static str, &'static [&'static str])] = &[
        (
            "general",
            &[
                "zh", "ja", "en", "es", "fr", "de", "pt", "it", "nl", "pl", "cs", "sk", "ro", "hu", "tr", "vi", "id",
                "ms", "ca", "da", "sv", "no", "fi", "hr", "sl", "lt", "lv", "et",
            ],
        ),
        (
            "cyrillic",
            &["ru", "uk", "be", "bg", "sr", "mk", "kk", "ky", "mn", "tg"],
        ),
        ("korean", &["ko"]),
        ("greek", &["el"]),
        ("arabic", &["ar", "fa", "ur", "ps", "ku", "sd", "ug"]),
        ("thai", &["th"]),
        ("devanagari", &["hi", "mr", "ne", "sa"]),
    ];
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
    /// Detected regions with a shorter side than this many pixels are noise.
    pub const MIN_SIDE: f32 = 3.0;
    /// At most this many text regions are kept per image.
    pub const MAX_REGIONS: usize = 3000;
}

/// How an image is prepared for the two orientation classifiers (RGB, ImageNet statistics).
pub struct ClassifierSettings;

impl ClassifierSettings {
    pub const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
    pub const STANDARD_DEVIATION: [f32; 3] = [0.229, 0.224, 0.225];
    /// Document classifier: shorter side scaled to this, then a centered square crop.
    pub const DOCUMENT_SHORT_SIDE: u32 = 256;
    pub const DOCUMENT_CROP_SIDE: u32 = 224;
    /// Text line classifier: every line is resized to this fixed size.
    pub const LINE_WIDTH: u32 = 160;
    pub const LINE_HEIGHT: u32 = 80;
}

/// Classes of the document orientation classifier: how many degrees clockwise the page content is turned.
pub struct DocumentTilts;

impl DocumentTilts {
    pub const CLASSES: [u16; 4] = [0, 90, 180, 270];
    /// When the best class beats the runner-up by less than this, the classifier is unsure and both are tried.
    pub const MIN_CONFIDENT_MARGIN: f32 = 0.3;
}

/// How the lines vote on whether the whole page is upside down.
pub struct PageVote;

impl PageVote {
    /// With fewer lines than this the vote means nothing.
    pub const MIN_LINES: usize = 3;
}

/// How a detected line is cut out of the image.
pub struct LineCrop;

impl LineCrop {
    /// Crops shorter than this many pixels on a side are skipped.
    pub const MIN_SIDE: u32 = 2;
    /// Crops at least this many times taller than wide are vertical text and are laid down before reading.
    pub const VERTICAL_ASPECT_RATIO: f32 = 1.5;
}

/// How a cropped line is prepared for the text recognizer (BGR).
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
