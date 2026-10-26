use std::ops::Range;

/// Finds a phrase in a folded text as a whole word, never inside a longer word and never over text already claimed.
pub struct PhraseSearch;

impl PhraseSearch {
    /// True for scripts written without spaces between words: there a phrase may sit inside a run of characters.
    pub fn is_written_without_spaces(phrase: &str) -> bool {
        phrase
            .chars()
            .any(|character| matches!(character as u32, 0x2E80..=0x9FFF | 0xAC00..=0xD7AF | 0xF900..=0xFAFF))
    }

    /// The first occurrence of `phrase` that is a whole word and does not touch a `claimed` range.
    pub fn find(text: &str, phrase: &str, claimed: &[Range<usize>]) -> Option<Range<usize>> {
        let whole_words = !Self::is_written_without_spaces(phrase);
        text.match_indices(phrase)
            .map(|(start, found)| start..start + found.len())
            .find(|range| {
                (!whole_words || Self::is_whole_word(text, range))
                    && !claimed
                        .iter()
                        .any(|other| range.start < other.end && other.start < range.end)
            })
    }

    fn is_whole_word(text: &str, range: &Range<usize>) -> bool {
        let before = text[..range.start].chars().next_back();
        let after = text[range.end..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_whole_words_only() {
        assert_eq!(PhraseSearch::find("la factura 12", "factura", &[]), Some(3..10));
        assert_eq!(PhraseSearch::find("refactura", "factura", &[]), None);
        assert_eq!(PhraseSearch::find("facturas", "factura", &[]), None);
    }

    #[test]
    fn finds_phrases_with_spaces() {
        assert!(PhraseSearch::find("nota de credito no 5", "nota de credito", &[]).is_some());
    }

    #[test]
    fn finds_phrases_inside_runs_of_cjk_characters() {
        assert!(PhraseSearch::find("北京发票号码2026", "发票", &[]).is_some());
    }

    #[test]
    fn skips_text_that_was_already_claimed() {
        let text = "tax invoice 7";

        assert_eq!(PhraseSearch::find(text, "invoice", &[0..11]), None);
        assert_eq!(PhraseSearch::find(text, "invoice", &[0..3]), Some(4..11));
    }
}
