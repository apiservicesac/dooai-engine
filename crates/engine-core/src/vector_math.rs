/// The few vector operations the engine needs.
pub struct VectorMath;

impl VectorMath {
    pub fn dot(first: &[f32], second: &[f32]) -> f32 {
        first.iter().zip(second).map(|(left, right)| left * right).sum()
    }

    pub fn norm(vector: &[f32]) -> f32 {
        Self::dot(vector, vector).sqrt()
    }

    /// The vector scaled to length 1; the zero vector stays as it is.
    pub fn normalized(vector: &[f32]) -> Vec<f32> {
        let norm = Self::norm(vector);
        if norm == 0.0 {
            return vector.to_vec();
        }
        vector.iter().map(|component| component / norm).collect()
    }

    /// 1 for the same direction, 0 for unrelated, -1 for opposite.
    pub fn cosine(first: &[f32], second: &[f32]) -> f32 {
        let norms = Self::norm(first) * Self::norm(second);
        if norms == 0.0 {
            return 0.0;
        }
        Self::dot(first, second) / norms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizing_gives_unit_length() {
        let vector = VectorMath::normalized(&[3.0, 4.0]);

        assert!((VectorMath::norm(&vector) - 1.0).abs() < 1e-6);
        assert_eq!(vector, vec![0.6, 0.8]);
    }

    #[test]
    fn cosine_ranges_from_opposite_to_identical() {
        assert!((VectorMath::cosine(&[1.0, 0.0], &[2.0, 0.0]) - 1.0).abs() < 1e-6);
        assert!(VectorMath::cosine(&[1.0, 0.0], &[0.0, 5.0]).abs() < 1e-6);
        assert!((VectorMath::cosine(&[1.0, 0.0], &[-1.0, 0.0]) + 1.0).abs() < 1e-6);
    }

    #[test]
    fn the_zero_vector_is_similar_to_nothing() {
        assert_eq!(VectorMath::cosine(&[0.0, 0.0], &[1.0, 1.0]), 0.0);
        assert_eq!(VectorMath::normalized(&[0.0, 0.0]), vec![0.0, 0.0]);
    }
}
