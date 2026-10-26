use serde::Serialize;

use crate::{BoundingBox, Point};

/// The four corners of a text line, in the line's own frame: top-left, top-right, bottom-right, bottom-left.
/// Tilted text keeps its tilt, so a line can be cropped and straightened exactly.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(transparent)]
pub struct TextQuad {
    corners: [Point; 4],
}

impl TextQuad {
    /// Orders four corners of a rectangle given in any order.
    pub fn from_unordered(mut points: [Point; 4]) -> Self {
        points.sort_by(|first, second| first.x.total_cmp(&second.x));
        let mut left = [points[0], points[1]];
        let mut right = [points[2], points[3]];
        left.sort_by(|first, second| first.y.total_cmp(&second.y));
        right.sort_by(|first, second| first.y.total_cmp(&second.y));
        Self {
            corners: [left[0], right[0], right[1], left[1]],
        }
    }

    pub fn corners(&self) -> &[Point; 4] {
        &self.corners
    }

    pub fn top_left(&self) -> Point {
        self.corners[0]
    }

    pub fn top_right(&self) -> Point {
        self.corners[1]
    }

    pub fn bottom_right(&self) -> Point {
        self.corners[2]
    }

    pub fn bottom_left(&self) -> Point {
        self.corners[3]
    }

    /// Length of the longer horizontal edge.
    pub fn width(&self) -> f32 {
        let top = self.top_left().distance_to(&self.top_right());
        let bottom = self.bottom_left().distance_to(&self.bottom_right());
        top.max(bottom)
    }

    /// Length of the longer vertical edge.
    pub fn height(&self) -> f32 {
        let left = self.top_left().distance_to(&self.bottom_left());
        let right = self.top_right().distance_to(&self.bottom_right());
        left.max(right)
    }

    pub fn center(&self) -> Point {
        let corner_count = self.corners.len() as f32;
        let x = self.corners.iter().map(|corner| corner.x).sum::<f32>() / corner_count;
        let y = self.corners.iter().map(|corner| corner.y).sum::<f32>() / corner_count;
        Point::new(x, y)
    }

    /// Angle of the top edge in radians; positive when the line descends to the right.
    pub fn tilt_radians(&self) -> f32 {
        let (left, right) = (self.top_left(), self.top_right());
        (right.y - left.y).atan2(right.x - left.x)
    }

    pub fn bounding_box(&self) -> BoundingBox {
        let xs = self.corners.iter().map(|corner| corner.x);
        let ys = self.corners.iter().map(|corner| corner.y);
        BoundingBox::new(
            xs.clone().fold(f32::MAX, f32::min),
            ys.clone().fold(f32::MAX, f32::min),
            xs.fold(f32::MIN, f32::max),
            ys.fold(f32::MIN, f32::max),
        )
    }

    /// Pushes every side outward by `distance`, along the quad's own axes.
    pub fn expanded(&self, distance: f32) -> Self {
        let across = self.top_left().direction_to(&self.top_right());
        let down = self.top_left().direction_to(&self.bottom_left());
        let pushed = |corner: Point, across_sign: f32, down_sign: f32| {
            corner
                .moved_along(&across, across_sign * distance)
                .moved_along(&down, down_sign * distance)
        };
        Self {
            corners: [
                pushed(self.top_left(), -1.0, -1.0),
                pushed(self.top_right(), 1.0, -1.0),
                pushed(self.bottom_right(), 1.0, 1.0),
                pushed(self.bottom_left(), -1.0, 1.0),
            ],
        }
    }

    pub fn scaled(&self, horizontal_factor: f32, vertical_factor: f32) -> Self {
        Self {
            corners: self
                .corners
                .map(|corner| Point::new(corner.x * horizontal_factor, corner.y * vertical_factor)),
        }
    }

    pub fn clamped(&self, max_width: f32, max_height: f32) -> Self {
        Self {
            corners: self
                .corners
                .map(|corner| Point::new(corner.x.clamp(0.0, max_width), corner.y.clamp(0.0, max_height))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upright_rectangle() -> TextQuad {
        TextQuad::from_unordered([
            Point::new(100.0, 50.0),
            Point::new(10.0, 10.0),
            Point::new(10.0, 50.0),
            Point::new(100.0, 10.0),
        ])
    }

    #[test]
    fn orders_corners_clockwise_from_the_top_left() {
        let quad = upright_rectangle();

        assert_eq!(quad.top_left(), Point::new(10.0, 10.0));
        assert_eq!(quad.top_right(), Point::new(100.0, 10.0));
        assert_eq!(quad.bottom_right(), Point::new(100.0, 50.0));
        assert_eq!(quad.bottom_left(), Point::new(10.0, 50.0));
        assert_eq!((quad.width(), quad.height()), (90.0, 40.0));
    }

    #[test]
    fn expanding_an_upright_rectangle_grows_every_side() {
        let bounds = upright_rectangle().expanded(5.0).bounding_box();

        assert_eq!(bounds, BoundingBox::new(5.0, 5.0, 105.0, 55.0));
    }

    #[test]
    fn expanding_keeps_a_tilted_rectangle_tilted() {
        let tilted = TextQuad::from_unordered([
            Point::new(0.0, 0.0),
            Point::new(86.6, 50.0),
            Point::new(76.6, 67.3),
            Point::new(-10.0, 17.3),
        ]);

        let expanded = tilted.expanded(4.0);

        assert!((expanded.tilt_radians() - tilted.tilt_radians()).abs() < 1e-3);
        assert!((expanded.width() - (tilted.width() + 8.0)).abs() < 0.1);
    }

    #[test]
    fn scales_and_clamps_to_the_image() {
        let restored = upright_rectangle().scaled(2.0, 2.0).clamped(150.0, 80.0);

        assert_eq!(restored.bounding_box(), BoundingBox::new(20.0, 20.0, 150.0, 80.0));
    }
}
