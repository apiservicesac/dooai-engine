use std::ops::Range;

use crate::classification::{ClassScore, Classification};
use crate::lexicon::Lexicon;
use crate::phrase_search::PhraseSearch;
use crate::settings::ClassifierSettings;
use crate::text_folding::TextFolding;

#[derive(Clone, Copy, PartialEq)]
enum PhraseKind {
    Title,
    Term,
}

struct Entry {
    phrase: String,
    class_index: usize,
    kind: PhraseKind,
}

/// Tells what kind of document a text is from the words it uses, in any of the lexicon's languages.
/// A kind's name in the title weighs the most; when the evidence is thin or ambiguous it answers `unknown`
/// instead of guessing, so the caller can ask something stronger.
pub struct LexiconClassifier {
    labels: Vec<String>,
    /// Longest phrase first, so "tax invoice" is read as one phrase and not also as "invoice".
    entries: Vec<Entry>,
}

impl LexiconClassifier {
    pub fn new(lexicon: Lexicon) -> Self {
        let mut entries = Vec::new();
        for (class_index, class) in lexicon.classes.iter().enumerate() {
            let phrases = class
                .titles
                .iter()
                .map(|title| (title, PhraseKind::Title))
                .chain(class.terms.iter().map(|term| (term, PhraseKind::Term)));
            for (phrase, kind) in phrases {
                let phrase = TextFolding::fold(phrase.trim());
                if !phrase.is_empty() {
                    entries.push(Entry {
                        phrase,
                        class_index,
                        kind,
                    });
                }
            }
        }
        entries.sort_by_key(|entry| std::cmp::Reverse(entry.phrase.chars().count()));
        Self {
            labels: lexicon.classes.into_iter().map(|class| class.label).collect(),
            entries,
        }
    }

    pub fn labels(&self) -> Vec<&str> {
        self.labels.iter().map(String::as_str).collect()
    }

    pub fn classify(&self, text: &str) -> Classification {
        let text: String = text.chars().take(ClassifierSettings::MAX_CHARACTERS).collect();
        let folded = TextFolding::fold(&text);
        let title_end = folded
            .char_indices()
            .nth(ClassifierSettings::TITLE_REGION_CHARACTERS)
            .map_or(folded.len(), |(offset, _)| offset);

        let mut scores = vec![0_u32; self.labels.len()];
        let mut matches: Vec<Vec<&str>> = vec![Vec::new(); self.labels.len()];
        let mut claimed: Vec<Range<usize>> = Vec::new();
        for entry in &self.entries {
            let Some(found) = PhraseSearch::find(&folded, &entry.phrase, &claimed) else {
                continue;
            };
            scores[entry.class_index] += match (entry.kind, found.start < title_end) {
                (PhraseKind::Title, true) => ClassifierSettings::TITLE_WEIGHT,
                (PhraseKind::Title, false) => ClassifierSettings::BODY_TITLE_WEIGHT,
                (PhraseKind::Term, _) => ClassifierSettings::TERM_WEIGHT,
            };
            matches[entry.class_index].push(&entry.phrase);
            claimed.push(found);
        }
        self.decide(&scores, &matches)
    }

    fn decide(&self, scores: &[u32], matches: &[Vec<&str>]) -> Classification {
        let mut ranking: Vec<ClassScore> = self
            .labels
            .iter()
            .zip(scores)
            .filter(|(_, score)| **score > 0)
            .map(|(label, &score)| ClassScore {
                label: label.clone(),
                score,
            })
            .collect();
        ranking.sort_by_key(|class| std::cmp::Reverse(class.score));

        let best = ranking.first().map_or(0, |class| class.score);
        let second = ranking.get(1).map_or(0, |class| class.score);
        if best < ClassifierSettings::MIN_SCORE || best - second < ClassifierSettings::MIN_MARGIN {
            return Classification {
                label: ClassifierSettings::UNKNOWN_LABEL.to_string(),
                confidence: 0.0,
                matched: Vec::new(),
                scores: ranking,
            };
        }
        let winner = self
            .labels
            .iter()
            .position(|label| *label == ranking[0].label)
            .unwrap_or_default();
        Classification {
            label: ranking[0].label.clone(),
            confidence: 1.0 - (second + 1) as f32 / (best + 1) as f32,
            matched: matches[winner].iter().map(|phrase| phrase.to_string()).collect(),
            scores: ranking,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classifier() -> LexiconClassifier {
        LexiconClassifier::new(Lexicon::built_in())
    }

    #[test]
    fn a_title_in_any_language_names_the_kind() {
        for (text, label) in [
            ("FACTURA ELECTRÓNICA F001-123 Cliente: Andina SAC", "invoice"),
            ("Rechnung Nr. 2026-0042 Zahlungsbedingungen: 14 Tage", "invoice"),
            ("发票号码 2026-0042 购买方 北京新华贸易有限公司", "invoice"),
            ("GUÍA DE REMISIÓN ELECTRÓNICA T001-45 Transportista", "delivery_note"),
            ("Bon de livraison N° 221 Livré à Société Orion", "delivery_note"),
            ("Kontoauszug IBAN DE89 Anfangssaldo Endsaldo", "bank_statement"),
        ] {
            let classification = classifier().classify(text);
            assert_eq!(classification.label, label, "{text}: {classification:?}");
        }
    }

    #[test]
    fn a_credit_note_that_mentions_the_invoice_is_still_a_credit_note() {
        let classification =
            classifier().classify("NOTA DE CRÉDITO BC01-12\nFactura afectada: F001-123\nMotivo: devolución");

        assert_eq!(classification.label, "credit_note");
        assert!(
            classification.matched.contains(&"nota de credito".to_string()),
            "{classification:?}"
        );
    }

    #[test]
    fn the_longest_phrase_wins() {
        let classification = classifier().classify("Faktura korygująca nr 5 do faktury");

        assert_eq!(classification.label, "credit_note", "{classification:?}");
    }

    #[test]
    fn a_kind_found_far_from_the_title_counts_for_less() {
        let filler = "x ".repeat(ClassifierSettings::TITLE_REGION_CHARACTERS);
        let in_title = classifier().classify("Invoice 7");
        let in_body = classifier().classify(&format!("{filler} invoice 7"));

        assert_eq!(in_title.scores[0].score, ClassifierSettings::TITLE_WEIGHT);
        assert_eq!(in_body.scores[0].score, ClassifierSettings::BODY_TITLE_WEIGHT);
    }

    #[test]
    fn thin_or_ambiguous_evidence_is_unknown() {
        for text in [
            "",
            "Dear Mr. Smith, thank you for the meeting.",
            "Invoice and delivery note and receipt",
        ] {
            let classification = classifier().classify(text);
            assert_eq!(
                classification.label,
                ClassifierSettings::UNKNOWN_LABEL,
                "{text}: {classification:?}"
            );
            assert_eq!(classification.confidence, 0.0);
        }
    }

    #[test]
    fn a_lone_title_in_the_title_region_is_enough() {
        assert_eq!(classifier().classify("Invoice").label, "invoice");
    }

    #[test]
    fn confidence_grows_with_the_lead() {
        let narrow = classifier().classify("Invoice 7 credit note ref");
        let clear = classifier().classify("Invoice 7 amount due payment terms");

        assert!(clear.confidence > narrow.confidence || narrow.label == ClassifierSettings::UNKNOWN_LABEL);
        assert!(clear.confidence > 0.7, "{clear:?}");
    }

    #[test]
    fn a_word_inside_a_longer_word_does_not_count() {
        assert_eq!(
            classifier().classify("refactura prefacturación").label,
            ClassifierSettings::UNKNOWN_LABEL
        );
    }
}
