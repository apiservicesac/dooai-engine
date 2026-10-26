use engine_core::VectorMath;
use ndarray::{ArrayView2, ArrayView3, Axis};

/// Averages the vectors of the real tokens of each text (padding does not count) and scales the result to length 1.
pub struct MeanPooling;

impl MeanPooling {
    /// `hidden_states` is `[texts, tokens, dimensions]` and `attention_mask` is `[texts, tokens]`.
    pub fn pool(hidden_states: ArrayView3<f32>, attention_mask: ArrayView2<i64>) -> Vec<Vec<f32>> {
        hidden_states
            .axis_iter(Axis(0))
            .zip(attention_mask.axis_iter(Axis(0)))
            .map(|(tokens, mask)| {
                let mut sum = vec![0.0; tokens.ncols()];
                let mut real_tokens = 0.0_f32;
                for (token, &is_real) in tokens.axis_iter(Axis(0)).zip(mask.iter()) {
                    if is_real == 0 {
                        continue;
                    }
                    real_tokens += 1.0;
                    for (total, value) in sum.iter_mut().zip(token.iter()) {
                        *total += value;
                    }
                }
                let mean: Vec<f32> = sum.iter().map(|total| total / real_tokens.max(1.0)).collect();
                VectorMath::normalized(&mean)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use ndarray::{arr2, arr3};

    use super::*;

    #[test]
    fn padding_tokens_do_not_count() {
        let hidden_states = arr3(&[[[1.0, 0.0], [3.0, 0.0], [0.0, 100.0]]]);
        let attention_mask = arr2(&[[1, 1, 0]]);

        let pooled = MeanPooling::pool(hidden_states.view(), attention_mask.view());

        assert_eq!(pooled, vec![vec![1.0, 0.0]]);
    }

    #[test]
    fn every_text_gets_its_own_unit_vector() {
        let hidden_states = arr3(&[[[3.0, 4.0]], [[0.0, 2.0]]]);
        let attention_mask = arr2(&[[1], [1]]);

        let pooled = MeanPooling::pool(hidden_states.view(), attention_mask.view());

        assert_eq!(pooled, vec![vec![0.6, 0.8], vec![0.0, 1.0]]);
    }
}
