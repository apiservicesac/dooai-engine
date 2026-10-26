use image::RgbImage;
use image::imageops;

/// A turn by a multiple of 90 degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rotation {
    clockwise_degrees: u16,
}

impl Rotation {
    pub const HALF_TURN: Rotation = Rotation { clockwise_degrees: 180 };

    /// The turn that undoes content tilted `tilt_degrees` clockwise.
    pub fn correcting(tilt_degrees: u16) -> Self {
        Self {
            clockwise_degrees: (360 - tilt_degrees % 360) % 360,
        }
    }

    pub fn clockwise_degrees(&self) -> u16 {
        self.clockwise_degrees
    }

    pub fn apply(&self, image: RgbImage) -> RgbImage {
        match self.clockwise_degrees {
            90 => imageops::rotate90(&image),
            180 => imageops::rotate180(&image),
            270 => imageops::rotate270(&image),
            _ => image,
        }
    }
}

#[cfg(test)]
mod tests {
    use image::Rgb;

    use super::*;

    #[test]
    fn correcting_a_clockwise_tilt_turns_the_other_way() {
        assert_eq!(Rotation::correcting(0).clockwise_degrees(), 0);
        assert_eq!(Rotation::correcting(90).clockwise_degrees(), 270);
        assert_eq!(Rotation::correcting(180), Rotation::HALF_TURN);
        assert_eq!(Rotation::correcting(270).clockwise_degrees(), 90);
    }

    #[test]
    fn a_clockwise_quarter_turn_swaps_the_sides() {
        let mut image = RgbImage::new(4, 2);
        image.put_pixel(0, 0, Rgb([255, 0, 0]));

        let turned = Rotation::correcting(270).apply(image);

        assert_eq!(turned.dimensions(), (2, 4));
        assert_eq!(turned.get_pixel(1, 0), &Rgb([255, 0, 0]));
    }
}
