use engine_core::BoundingBox;
use ndarray::{Array2, ArrayView2};

use crate::connected_component::ConnectedComponent;
use crate::detector_settings::DetectorSettings;
use crate::settings::DetectorInput;

/// Turns the detector's probability map into text boxes (DB post-processing).
/// Boxes are axis-aligned, which suits scanned and photographed documents.
pub struct BoxExtractor {
    settings: DetectorSettings,
}

impl BoxExtractor {
    pub fn new(settings: DetectorSettings) -> Self {
        Self { settings }
    }

    /// Boxes are returned in the coordinates of the probability map.
    pub fn extract(&self, probability_map: ArrayView2<f32>) -> Vec<BoundingBox> {
        let (rows, columns) = probability_map.dim();
        let mut visited = Array2::from_elem((rows, columns), false);
        let mut boxes = Vec::new();

        for row in 0..rows {
            for column in 0..columns {
                if visited[[row, column]] || !self.is_text_pixel(probability_map[[row, column]]) {
                    continue;
                }
                let component = self.grow_component(&probability_map, &mut visited, row, column);
                if let Some(text_box) = self.to_text_box(&component) {
                    boxes.push(text_box);
                }
            }
        }
        boxes
    }

    fn is_text_pixel(&self, probability: f32) -> bool {
        probability >= self.settings.threshold
    }

    /// Flood fill over the 8 neighbours of each pixel.
    fn grow_component(
        &self,
        probability_map: &ArrayView2<f32>,
        visited: &mut Array2<bool>,
        start_row: usize,
        start_column: usize,
    ) -> ConnectedComponent {
        let (rows, columns) = probability_map.dim();
        let mut component =
            ConnectedComponent::starting_at(start_row, start_column, probability_map[[start_row, start_column]]);
        visited[[start_row, start_column]] = true;
        let mut pending = vec![(start_row, start_column)];

        while let Some((row, column)) = pending.pop() {
            let row_range = row.saturating_sub(1)..=(row + 1).min(rows - 1);
            for neighbour_row in row_range {
                let column_range = column.saturating_sub(1)..=(column + 1).min(columns - 1);
                for neighbour_column in column_range {
                    let probability = probability_map[[neighbour_row, neighbour_column]];
                    if visited[[neighbour_row, neighbour_column]] || !self.is_text_pixel(probability) {
                        continue;
                    }
                    visited[[neighbour_row, neighbour_column]] = true;
                    component.include(neighbour_row, neighbour_column, probability);
                    pending.push((neighbour_row, neighbour_column));
                }
            }
        }
        component
    }

    /// The map marks a shrunk core of each text line, so the box is grown back
    /// by the DB unclip distance: area * ratio / perimeter.
    fn to_text_box(&self, component: &ConnectedComponent) -> Option<BoundingBox> {
        if component.shortest_side() < DetectorInput::MIN_COMPONENT_SIDE
            || component.mean_probability() < self.settings.box_threshold
        {
            return None;
        }
        let core = component.bounding_box();
        let perimeter = 2.0 * (core.width() + core.height());
        let unclip_distance = core.width() * core.height() * self.settings.unclip_ratio / perimeter;
        Some(core.expanded(unclip_distance))
    }
}

#[cfg(test)]
mod tests {
    use ndarray::Array2;

    use super::*;

    fn extractor() -> BoxExtractor {
        BoxExtractor::new(DetectorSettings {
            threshold: 0.3,
            box_threshold: 0.6,
            unclip_ratio: 1.5,
        })
    }

    #[test]
    fn finds_one_box_per_separate_region() {
        let mut probability_map = Array2::<f32>::zeros((40, 100));
        probability_map.slice_mut(ndarray::s![10..16, 5..40]).fill(0.9);
        probability_map.slice_mut(ndarray::s![25..31, 50..90]).fill(0.9);

        let boxes = extractor().extract(probability_map.view());

        assert_eq!(boxes.len(), 2);
        assert!(boxes[0].left < 5.0 && boxes[0].right > 40.0);
        assert!(boxes[1].top < 25.0 && boxes[1].bottom > 31.0);
    }

    #[test]
    fn ignores_tiny_and_low_confidence_regions() {
        let mut probability_map = Array2::<f32>::zeros((40, 100));
        probability_map.slice_mut(ndarray::s![5..7, 5..40]).fill(0.9);
        probability_map.slice_mut(ndarray::s![20..28, 5..40]).fill(0.4);

        assert!(extractor().extract(probability_map.view()).is_empty());
    }
}
