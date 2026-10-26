use crate::settings::LanguageScripts;

/// Maps a language code to the recognizer that reads its alphabet.
pub struct LanguageCatalog;

impl LanguageCatalog {
    /// Accepts `ru`, `RU`, `pt-BR`, `pt_br`, or a recognizer name such as `cyrillic`.
    pub fn recognizer_for(language: &str) -> Option<&'static str> {
        let normalized = language.trim().to_lowercase();
        let code = normalized.split(['-', '_']).next().unwrap_or_default();
        LanguageScripts::TABLE
            .iter()
            .find(|(recognizer, languages)| *recognizer == code || languages.contains(&code))
            .map(|(recognizer, _)| *recognizer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_recognizer_of_a_language() {
        assert_eq!(LanguageCatalog::recognizer_for("ru"), Some("cyrillic"));
        assert_eq!(LanguageCatalog::recognizer_for("ko"), Some("korean"));
        assert_eq!(LanguageCatalog::recognizer_for("es"), Some("general"));
    }

    #[test]
    fn ignores_case_and_region() {
        assert_eq!(LanguageCatalog::recognizer_for("PT-BR"), Some("general"));
        assert_eq!(LanguageCatalog::recognizer_for("sr_RS"), Some("cyrillic"));
    }

    #[test]
    fn accepts_a_recognizer_name_and_rejects_unknown_codes() {
        assert_eq!(LanguageCatalog::recognizer_for("cyrillic"), Some("cyrillic"));
        assert_eq!(LanguageCatalog::recognizer_for("xx"), None);
    }
}
