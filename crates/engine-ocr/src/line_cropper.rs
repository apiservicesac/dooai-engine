use engine_core::TextQuad;
use image::RgbImage;
use image::imageops;
use imageproc::geometric_transformations::{Border, Interpolation, Projection, warp_into};

use crate::settings::LineCrop;

/// Cuts a detected line out of the page and straightens it with a perspective transform,
/// so tilted and skewed lines reach the recognizer as flat, horizontal text.
pub struct LineCropper;

impl LineCropper {
    pub fn crop(image: &RgbImage, quad: &TextQuad) -> Option<RgbImage> {
        let width = quad.width().round() as u32;
        let height = quad.height().round() as u32;
        if width < LineCrop::MIN_SIDE || height < LineCrop::MIN_SIDE {
            return None;
        }

        let source = quad.corners().map(|corner| (corner.x, corner.y));
        let flat = [
            (0.0, 0.0),
            (width as f32, 0.0),
            (width as f32, height as f32),
            (0.0, height as f32),
        ];
        let projection = Projection::from_control_points(source, flat)?;
        let mut line = RgbImage::new(width, height);
        warp_into(image, projection, Interpolation::Bicubic, Border::Replicate, &mut line);

        let is_vertical = height as f32 >= width as f32 * LineCrop::VERTICAL_ASPECT_RATIO;
        Some(if is_vertical { imageops::rotate270(&line) } else { line })
    }
}

#[cfg(test)]
mod tests {
    use engine_core::Point;
    use image::Rgb;

    use super::*;

    #[test]
    fn straightens_a_tilted_line_into_an_upright_crop() {
        let mut page = RgbImage::from_pixel(200, 200, Rgb([255, 255, 255]));
        // A black bar, 100 x 20, turned 30 degrees about its top-left corner at (50, 50).
        let (sine, cosine) = 30f32.to_radians().sin_cos();
        for along in 0..100 {
            for across in 0..20 {
                let (along, across) = (along as f32, across as f32);
                let column = 50.0 + along * cosine - across * sine;
                let row = 50.0 + along * sine + across * cosine;
                page.put_pixel(column as u32, row as u32, Rgb([0, 0, 0]));
            }
        }
        let quad = TextQuad::from_unordered([
            Point::new(50.0, 50.0),
            Point::new(50.0 + 100.0 * cosine, 50.0 + 100.0 * sine),
            Point::new(50.0 + 100.0 * cosine - 20.0 * sine, 50.0 + 100.0 * sine + 20.0 * cosine),
            Point::new(50.0 - 20.0 * sine, 50.0 + 20.0 * cosine),
        ]);

        let crop = LineCropper::crop(&page, &quad).unwrap();

        assert_eq!(crop.dimensions(), (100, 20));
        let dark_pixels = crop.pixels().filter(|pixel| pixel[0] < 128).count();
        assert!(
            dark_pixels > 100 * 20 * 8 / 10,
            "only {dark_pixels} of 2000 pixels are dark"
        );
    }

    #[test]
    fn skips_crops_too_small_to_read() {
        let page = RgbImage::new(50, 50);
        let quad = TextQuad::from_unordered([
            Point::new(5.0, 5.0),
            Point::new(6.0, 5.0),
            Point::new(6.0, 6.0),
            Point::new(5.0, 6.0),
        ]);

        assert!(LineCropper::crop(&page, &quad).is_none());
    }
}
