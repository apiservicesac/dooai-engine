mod classification;
mod lexicon;
mod lexicon_classifier;
mod phrase_search;
mod settings;
mod text_folding;

pub use classification::{ClassScore, Classification};
pub use lexicon::{Lexicon, LexiconClass};
pub use lexicon_classifier::LexiconClassifier;
pub use settings::ClassifierSettings;
