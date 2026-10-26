use std::collections::{HashMap, HashSet};

use engine_core::EngineError;
use serde::Deserialize;

use crate::settings::ClassifierSettings;
use crate::text_folding::TextFolding;

const BUILT_IN_LEXICON: &str = include_str!("../data/document-lexicon.json");

/// One kind of document: the words that name it, and phrases typical of it, in any language.
#[derive(Debug, Clone, Deserialize)]
pub struct LexiconClass {
    pub label: String,
    /// What the kind is called ("Invoice", "Factura"...): found in the title, they settle the matter.
    pub titles: Vec<String>,
    /// Phrases that point to the kind without naming it ("amount due", "recibí conforme").
    pub terms: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Lexicon {
    pub classes: Vec<LexiconClass>,
}

impl Lexicon {
    /// The kinds of document Odoo deals with: invoices, credit notes, delivery notes, receipts, contracts,
    /// purchase orders, quotations and bank statements, in some twenty languages.
    pub fn built_in() -> Self {
        Self::from_json(BUILT_IN_LEXICON).expect("the built-in lexicon is valid")
    }

    pub fn from_json(json: &str) -> Result<Self, EngineError> {
        let lexicon: Lexicon = serde_json::from_str(json)
            .map_err(|error| EngineError::Configuration(format!("the lexicon is not valid: {error}")))?;
        lexicon.validate()?;
        Ok(lexicon)
    }

    /// A phrase may belong to one class only: otherwise the answer would depend on the order of the file.
    /// Repeating it inside its own class is harmless.
    fn validate(&self) -> Result<(), EngineError> {
        if self.classes.is_empty() {
            return Err(EngineError::Configuration("the lexicon has no classes".to_string()));
        }
        let mut labels = HashSet::new();
        let mut owners: HashMap<String, &str> = HashMap::new();
        for class in &self.classes {
            if class.label == ClassifierSettings::UNKNOWN_LABEL || !labels.insert(class.label.as_str()) {
                return Err(EngineError::Configuration(format!(
                    "the class '{}' is repeated or uses a reserved name",
                    class.label
                )));
            }
            if class.titles.iter().all(|title| title.trim().is_empty()) {
                return Err(EngineError::Configuration(format!(
                    "the class '{}' has no titles",
                    class.label
                )));
            }
            for phrase in class.titles.iter().chain(&class.terms) {
                let folded = TextFolding::fold(phrase.trim());
                if folded.is_empty() {
                    continue;
                }
                let owner = *owners.entry(folded).or_insert(class.label.as_str());
                if owner != class.label {
                    return Err(EngineError::Configuration(format!(
                        "the phrase '{phrase}' belongs to both '{owner}' and '{}'",
                        class.label
                    )));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_built_in_lexicon_is_valid_and_wide() {
        let lexicon = Lexicon::built_in();

        assert_eq!(lexicon.classes.len(), 8);
        assert!(lexicon.classes.iter().all(|class| class.titles.len() >= 20));
    }

    #[test]
    fn refuses_repeated_empty_reserved_and_shared_phrases() {
        for json in [
            r#"{"classes": []}"#,
            r#"{"classes": [{"label": "a", "titles": ["x"], "terms": []}, {"label": "a", "titles": ["y"], "terms": []}]}"#,
            r#"{"classes": [{"label": "a", "titles": [" "], "terms": []}]}"#,
            r#"{"classes": [{"label": "unknown", "titles": ["x"], "terms": []}]}"#,
            r#"{"classes": [{"label": "a", "titles": ["Factura"], "terms": []}, {"label": "b", "titles": ["factura"], "terms": []}]}"#,
            "not json",
        ] {
            assert!(Lexicon::from_json(json).is_err(), "{json}");
        }
    }
}
