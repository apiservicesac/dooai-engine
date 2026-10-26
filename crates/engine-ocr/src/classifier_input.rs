use image::RgbImage;
use image::imageops::{self, FilterType};

/// How an image is cut to the fixed size an orientation classifier expects.
#[derive(Debug, Clone, Copy)]
pub enum ClassifierInput {
    /// Scale the shorter side, then crop a centered square.
    ShortSideCenterCrop { short_side: u32, crop_side: u32 },
    /// Resize to an exact size.
    Fixed { width: u32, height: u32 },
}

impl ClassifierInput {
    pub fn prepare(&self, image: &RgbImage) -> RgbImage {
        match *self {
            Self::ShortSideCenterCrop { short_side, crop_side } => {
                let scale = short_side as f32 / image.width().min(image.height()) as f32;
                let width = ((image.width() as f32 * scale).round() as u32).max(crop_side);
                let height = ((image.height() as f32 * scale).round() as u32).max(crop_side);
                let resized = imageops::resize(image, width, height, FilterType::Triangle);
                let left = (width - crop_side) / 2;
                let top = (height - crop_side) / 2;
                imageops::crop_imm(&resized, left, top, crop_side, crop_side).to_image()
            }
            Self::Fixed { width, height } => imageops::resize(image, width, height, FilterType::Triangle),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_crop_yields_a_square_of_the_crop_side() {
        let image = RgbImage::new(600, 900);
        let input = ClassifierInput::ShortSideCenterCrop {
            short_side: 256,
            crop_side: 224,
        };

        assert_eq!(input.prepare(&image).dimensions(), (224, 224));
    }

    #[test]
    fn fixed_input_ignores_the_aspect_ratio() {
        let image = RgbImage::new(500, 40);

        assert_eq!(
            ClassifierInput::Fixed { width: 160, height: 80 }
                .prepare(&image)
                .dimensions(),
            (160, 80)
        );
    }
}
