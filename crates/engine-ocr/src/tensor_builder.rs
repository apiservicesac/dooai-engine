use image::RgbImage;
use ndarray::Array4;

/// Order in which the network expects the color channels.
#[derive(Debug, Clone, Copy)]
pub enum ChannelOrder {
    BlueGreenRed,
    RedGreenBlue,
}

/// Builds the `[1, 3, height, width]` input of a network: channels in the order it was trained on,
/// scaled to 0..1, then normalized per channel.
pub struct TensorBuilder {
    channel_order: ChannelOrder,
    mean: [f32; 3],
    standard_deviation: [f32; 3],
}

impl TensorBuilder {
    pub fn new(channel_order: ChannelOrder, mean: [f32; 3], standard_deviation: [f32; 3]) -> Self {
        Self {
            channel_order,
            mean,
            standard_deviation,
        }
    }

    pub fn build(&self, image: &RgbImage) -> Array4<f32> {
        let shape = (1, 3, image.height() as usize, image.width() as usize);
        Array4::from_shape_fn(shape, |(_, channel, row, column)| {
            let pixel = image.get_pixel(column as u32, row as u32);
            let source_channel = match self.channel_order {
                ChannelOrder::BlueGreenRed => 2 - channel,
                ChannelOrder::RedGreenBlue => channel,
            };
            let value = pixel.0[source_channel] as f32 / 255.0;
            (value - self.mean[channel]) / self.standard_deviation[channel]
        })
    }
}

#[cfg(test)]
mod tests {
    use image::Rgb;

    use super::*;

    #[test]
    fn blue_green_red_puts_blue_in_the_first_channel() {
        let image = RgbImage::from_pixel(2, 1, Rgb([255, 0, 0]));
        let builder = TensorBuilder::new(ChannelOrder::BlueGreenRed, [0.5; 3], [0.5; 3]);

        let tensor = builder.build(&image);

        assert_eq!(tensor.shape(), &[1, 3, 1, 2]);
        assert_eq!(tensor[[0, 0, 0, 0]], -1.0); // blue
        assert_eq!(tensor[[0, 2, 0, 0]], 1.0); // red
    }

    #[test]
    fn red_green_blue_keeps_the_channels_as_they_are() {
        let image = RgbImage::from_pixel(1, 1, Rgb([255, 0, 0]));
        let builder = TensorBuilder::new(ChannelOrder::RedGreenBlue, [0.5; 3], [0.5; 3]);

        assert_eq!(builder.build(&image)[[0, 0, 0, 0]], 1.0);
    }
}
