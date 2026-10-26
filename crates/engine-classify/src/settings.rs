//! The classifier's fixed values in one place.

pub struct ClassifierSettings;

impl ClassifierSettings {
    /// What a text gets when the evidence is too thin or ambiguous to name its kind.
    pub const UNKNOWN_LABEL: &'static str = "unknown";
    /// Only the start of a text is read: the beginning of a document says what it is.
    pub const MAX_CHARACTERS: usize = 20_000;
    /// The first this many characters are the title region: a document names its kind there.
    pub const TITLE_REGION_CHARACTERS: usize = 400;
    /// A kind's name found in the title region.
    pub const TITLE_WEIGHT: u32 = 5;
    /// A kind's name found later in the text: other documents mention it ("original invoice" in a credit note).
    pub const BODY_TITLE_WEIGHT: u32 = 2;
    /// A word or phrase typical of the kind, anywhere.
    pub const TERM_WEIGHT: u32 = 1;
    /// The best class needs at least this many points...
    pub const MIN_SCORE: u32 = 2;
    /// ...and this many more than the second best.
    pub const MIN_MARGIN: u32 = 2;
}
