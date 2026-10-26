use engine_core::EngineError;
use image::RgbImage;
use ort::environment::Environment;

use crate::character_set::CharacterSet;
use crate::language_catalog::LanguageCatalog;
use crate::model_layout::ModelLayout;
use crate::recognized_text::RecognizedText;
use crate::settings::LanguageScripts;
use crate::text_recognizer::TextRecognizer;

/// A recognizer and the alphabet it was trained on.
pub struct NamedRecognizer {
    pub name: String,
    recognizer: TextRecognizer,
}

impl NamedRecognizer {
    pub fn recognize(&self, line: &RgbImage) -> Result<RecognizedText, EngineError> {
        self.recognizer.recognize(line)
    }
}

/// Every installed recognizer, and the choice of which one reads a page.
pub struct RecognizerSet {
    recognizers: Vec<NamedRecognizer>,
}

impl RecognizerSet {
    pub fn load(environment: &Environment, layout: &ModelLayout) -> Result<Self, EngineError> {
        let mut recognizers = Vec::new();
        for (name, files) in layout.recognizers()? {
            let character_set = CharacterSet::from_config_file(&files.config)?;
            recognizers.push(NamedRecognizer {
                name,
                recognizer: TextRecognizer::load(environment, &files.model, character_set)?,
            });
        }
        Ok(Self { recognizers })
    }

    /// The recognizer for the language the caller named; the general one when none was named.
    pub fn choose(&self, language: Option<&str>) -> Result<&NamedRecognizer, EngineError> {
        match language.map(str::trim) {
            None | Some("") => Ok(&self.recognizers[0]),
            Some(code) if code.eq_ignore_ascii_case(LanguageScripts::AUTOMATIC) => Ok(&self.recognizers[0]),
            Some(code) => self.for_language(code),
        }
    }

    fn for_language(&self, code: &str) -> Result<&NamedRecognizer, EngineError> {
        let name = LanguageCatalog::recognizer_for(code)
            .ok_or_else(|| EngineError::UnsupportedLanguage(format!("'{code}' is not a known language")))?;
        self.recognizers
            .iter()
            .find(|recognizer| recognizer.name == name)
            .ok_or_else(|| {
                EngineError::UnsupportedLanguage(format!(
                    "'{code}' needs the '{name}' recognizer, which is not installed; installed: {}",
                    self.installed_names().join(", ")
                ))
            })
    }

    fn installed_names(&self) -> Vec<&str> {
        self.recognizers
            .iter()
            .map(|recognizer| recognizer.name.as_str())
            .collect()
    }
}
