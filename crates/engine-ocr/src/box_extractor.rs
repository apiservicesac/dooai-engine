use engine_core::{Point, TextQuad};
use image::{GrayImage, Luma};
use imageproc::contours::{BorderType, find_contours};
use imageproc::drawing::draw_polygon_mut;
use imageproc::geometry::min_area_rect;
use imageproc::point::Point as PixelPoint;
use ndarray::ArrayView2;

use crate::detector_settings::DetectorSettings;
use crate::settings::DetectorInput;

const FOREGROUND: Luma<u8> = Luma([255]);

/// Turns the detector's probability map into text lines (DB post-processing): threshold the map, trace each
/// region's outline, fit the smallest rotated rectangle around it, score it, and grow it back by the unclip distance.
pub struct BoxExtractor {
    settings: DetectorSettings,
}

impl BoxExtractor {
    pub fn new(settings: DetectorSettings) -> Self {
        Self { settings }
    }

    /// Lines are returned in the coordinates of the probability map.
    pub fn extract(&self, probability_map: ArrayView2<f32>) -> Vec<TextQuad> {
        let bitmap = self.binarize(&probability_map);
        find_contours::<i32>(&bitmap)
            .into_iter()
            .filter(|contour| contour.border_type == BorderType::Outer)
            .take(DetectorInput::MAX_REGIONS)
            .filter_map(|contour| self.to_text_quad(&contour.points, &probability_map))
            .collect()
    }

    fn binarize(&self, probability_map: &ArrayView2<f32>) -> GrayImage {
        let (rows, columns) = probability_map.dim();
        GrayImage::from_fn(columns as u32, rows as u32, |column, row| {
            let probability = probability_map[[row as usize, column as usize]];
            Luma([if probability >= self.settings.threshold { 255 } else { 0 }])
        })
    }

    /// The map marks a shrunk core of each text line, so the rectangle is grown back by the DB unclip
    /// distance: area * ratio / perimeter.
    fn to_text_quad(&self, outline: &[PixelPoint<i32>], probability_map: &ArrayView2<f32>) -> Option<TextQuad> {
        let corners = min_area_rect(outline).map(|corner| Point::new(corner.x as f32, corner.y as f32));
        let core = TextQuad::from_unordered(corners);
        if Self::shorter_side(&core) < DetectorInput::MIN_SIDE
            || self.mean_probability_inside(probability_map, &core) < self.settings.box_threshold
        {
            return None;
        }
        let area = core.width() * core.height();
        let perimeter = 2.0 * (core.width() + core.height());
        let grown = core.expanded(area * self.settings.unclip_ratio / perimeter);
        (Self::shorter_side(&grown) >= DetectorInput::MIN_SIDE).then_some(grown)
    }

    fn shorter_side(quad: &TextQuad) -> f32 {
        quad.width().min(quad.height())
    }

    /// Mean probability over the pixels covered by the rectangle.
    fn mean_probability_inside(&self, probability_map: &ArrayView2<f32>, quad: &TextQuad) -> f32 {
        let (rows, columns) = probability_map.dim();
        let bounds = quad.bounding_box();
        let left = (bounds.left.floor().max(0.0) as usize).min(columns - 1);
        let top = (bounds.top.floor().max(0.0) as usize).min(rows - 1);
        let right = (bounds.right.ceil() as usize).min(columns - 1);
        let bottom = (bounds.bottom.ceil() as usize).min(rows - 1);

        let outline: Vec<PixelPoint<i32>> = quad
            .corners()
            .iter()
            .map(|corner| {
                PixelPoint::new(
                    corner.x.round() as i32 - left as i32,
                    corner.y.round() as i32 - top as i32,
                )
            })
            .collect();
        let mut mask = GrayImage::new((right - left + 1) as u32, (bottom - top + 1) as u32);
        draw_polygon_mut(&mut mask, &outline, FOREGROUND);

        let (mut sum, mut count) = (0.0, 0);
        for (column, row, pixel) in mask.enumerate_pixels() {
            if pixel[0] > 0 {
                sum += probability_map[[top + row as usize, left + column as usize]];
                count += 1;
            }
        }
        if count == 0 { 0.0 } else { sum / count as f32 }
    }
}

#[cfg(test)]
mod tests {
    use ndarray::{Array2, s};

    use super::*;

    fn extractor() -> BoxExtractor {
        BoxExtractor::new(DetectorSettings {
            threshold: 0.3,
            box_threshold: 0.6,
            unclip_ratio: 1.5,
        })
    }

    #[test]
    fn finds_one_quad_per_separate_region() {
        let mut probability_map = Array2::<f32>::zeros((40, 100));
        probability_map.slice_mut(s![10..16, 5..40]).fill(0.9);
        probability_map.slice_mut(s![25..31, 50..90]).fill(0.9);

        let quads = extractor().extract(probability_map.view());

        assert_eq!(quads.len(), 2);
        let first = quads
            .iter()
            .map(|quad| quad.bounding_box())
            .find(|bounds| bounds.left < 10.0)
            .unwrap();
        assert!(first.left < 5.0 && first.right > 39.0 && first.top < 10.0 && first.bottom > 15.0);
    }

    #[test]
    fn fits_a_tilted_region_with_a_tilted_rectangle() {
        let mut probability_map = Array2::<f32>::zeros((120, 200));
        for column in 20..170 {
            let top = 20 + (column - 20) / 5; // descends 1 px per 5 px: about 11 degrees
            for row in top..top + 8 {
                probability_map[[row, column]] = 0.95;
            }
        }

        let quads = extractor().extract(probability_map.view());

        assert_eq!(quads.len(), 1);
        let tilt_degrees = quads[0].tilt_radians().to_degrees();
        assert!((tilt_degrees - 11.3).abs() < 2.0, "tilt was {tilt_degrees}");
    }

    #[test]
    fn ignores_tiny_and_low_confidence_regions() {
        let mut probability_map = Array2::<f32>::zeros((40, 100));
        probability_map.slice_mut(s![5..7, 5..40]).fill(0.9);
        probability_map.slice_mut(s![20..28, 5..40]).fill(0.4);

        assert!(extractor().extract(probability_map.view()).is_empty());
    }
}
