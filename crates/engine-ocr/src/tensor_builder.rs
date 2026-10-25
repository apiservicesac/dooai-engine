use image::RgbImage;
use ndarray::Array4;

/// Builds the `[1, 3, height, width]` input of a PP-OCR network:
/// BGR channel order, scaled to 0..1, then normalized per channel.
pub struct TensorBuilder {
    mean: [f32; 3],
    standard_deviation: [f32; 3],
}

impl TensorBuilder {
    pub fn new(mean: [f32; 3], standard_deviation: [f32; 3]) -> Self {
        Self {
            mean,
            standard_deviation,
        }
    }

    pub fn build(&self, image: &RgbImage) -> Array4<f32> {
        let shape = (1, 3, image.height() as usize, image.width() as usize);
        Array4::from_shape_fn(shape, |(_, channel, row, column)| {
            let pixel = image.get_pixel(column as u32, row as u32);
            let blue_green_red_value = pixel.0[2 - channel] as f32 / 255.0;
            (blue_green_red_value - self.mean[channel]) / self.standard_deviation[channel]
        })
    }
}

#[cfg(test)]
mod tests {
    use image::Rgb;

    use super::*;

    #[test]
    fn puts_blue_in_the_first_channel_and_normalizes() {
        let image = RgbImage::from_pixel(2, 1, Rgb([255, 0, 0]));
        let builder = TensorBuilder::new([0.5; 3], [0.5; 3]);

        let tensor = builder.build(&image);

        assert_eq!(tensor.shape(), &[1, 3, 1, 2]);
        assert_eq!(tensor[[0, 0, 0, 0]], -1.0); // blue
        assert_eq!(tensor[[0, 2, 0, 0]], 1.0); // red
    }
}
