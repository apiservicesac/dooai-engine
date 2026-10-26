use engine_core::Point;
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

    /// This turn followed by `other`.
    pub fn then(&self, other: Rotation) -> Rotation {
        Rotation {
            clockwise_degrees: (self.clockwise_degrees + other.clockwise_degrees) % 360,
        }
    }

    pub fn clockwise_degrees(&self) -> u16 {
        self.clockwise_degrees
    }

    /// Where a point of the turned image sits in the image before it was turned.
    /// `original_size` is the (width, height) of the image before the turn.
    pub fn restore(&self, point: Point, original_size: (f32, f32)) -> Point {
        let (width, height) = original_size;
        match self.clockwise_degrees {
            90 => Point::new(point.y, height - point.x),
            180 => Point::new(width - point.x, height - point.y),
            270 => Point::new(width - point.y, point.x),
            _ => point,
        }
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
    fn turns_add_up_around_the_circle() {
        assert_eq!(
            Rotation::correcting(270).then(Rotation::HALF_TURN).clockwise_degrees(),
            270
        );
        assert_eq!(Rotation::HALF_TURN.then(Rotation::HALF_TURN).clockwise_degrees(), 0);
    }

    #[test]
    fn restoring_a_point_undoes_every_turn() {
        let original_size = (4.0, 2.0);
        let marker = Point::new(0.5, 0.5);
        for tilt in [0, 90, 180, 270] {
            let rotation = Rotation::correcting(tilt);
            let mut image = RgbImage::new(4, 2);
            image.put_pixel(0, 0, Rgb([255, 0, 0]));
            let turned = rotation.apply(image);
            let (column, row, _) = turned.enumerate_pixels().find(|(_, _, pixel)| pixel[0] == 255).unwrap();

            let restored = rotation.restore(Point::new(column as f32 + 0.5, row as f32 + 0.5), original_size);

            assert_eq!(restored, marker, "tilt {tilt}");
        }
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
