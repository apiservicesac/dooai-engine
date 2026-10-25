use ndarray::ArrayView2;

use crate::character_set::CharacterSet;
use crate::recognized_text::RecognizedText;
use crate::settings::Decoding;

/// Greedy CTC decoding: best class per time step, repeats collapsed, blanks dropped.
pub struct CtcDecoder {
    character_set: CharacterSet,
}

impl CtcDecoder {
    pub fn new(character_set: CharacterSet) -> Self {
        Self { character_set }
    }

    pub fn class_count(&self) -> usize {
        self.character_set.len()
    }

    /// `probabilities` has one row per time step and one column per class.
    pub fn decode(&self, probabilities: ArrayView2<f32>) -> RecognizedText {
        let mut text = String::new();
        let mut probability_sum = 0.0;
        let mut kept_steps = 0;
        let mut previous_class = Decoding::BLANK_INDEX;

        for step in probabilities.rows() {
            let (best_class, best_probability) = step
                .iter()
                .copied()
                .enumerate()
                .max_by(|first, second| first.1.total_cmp(&second.1))
                .unwrap_or((Decoding::BLANK_INDEX, 0.0));

            if best_class != Decoding::BLANK_INDEX
                && best_class != previous_class
                && let Some(symbol) = self.character_set.symbol(best_class)
            {
                text.push_str(symbol);
                probability_sum += best_probability;
                kept_steps += 1;
            }
            previous_class = best_class;
        }

        let confidence = if kept_steps == 0 {
            0.0
        } else {
            probability_sum / kept_steps as f32
        };
        RecognizedText { text, confidence }
    }
}

#[cfg(test)]
mod tests {
    use ndarray::arr2;

    use super::*;

    fn decoder() -> CtcDecoder {
        CtcDecoder::new(CharacterSet::from_dictionary(vec!["a".to_string(), "b".to_string()]))
    }

    #[test]
    fn collapses_repeats_and_drops_blanks() {
        // classes: 0 blank, 1 "a", 2 "b", 3 space
        let probabilities = arr2(&[
            [0.1, 0.8, 0.05, 0.05],
            [0.1, 0.8, 0.05, 0.05],
            [0.9, 0.05, 0.03, 0.02],
            [0.1, 0.8, 0.05, 0.05],
            [0.1, 0.05, 0.8, 0.05],
        ]);

        let recognized = decoder().decode(probabilities.view());

        assert_eq!(recognized.text, "aab");
        assert!((recognized.confidence - 0.8).abs() < 1e-6);
    }

    #[test]
    fn silence_decodes_to_empty_text_with_zero_confidence() {
        let probabilities = arr2(&[[0.9, 0.05, 0.03, 0.02]]);

        let recognized = decoder().decode(probabilities.view());

        assert_eq!(recognized.text, "");
        assert_eq!(recognized.confidence, 0.0);
    }
}
