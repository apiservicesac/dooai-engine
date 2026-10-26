use serde::Serialize;

/// A position in pixels, with the origin at the top-left corner of the image.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Point) -> f32 {
        (other.x - self.x).hypot(other.y - self.y)
    }

    /// Unit vector pointing from this point to `other`; the zero vector when they coincide.
    pub fn direction_to(&self, other: &Point) -> Point {
        let distance = self.distance_to(other);
        if distance == 0.0 {
            return Point::new(0.0, 0.0);
        }
        Point::new((other.x - self.x) / distance, (other.y - self.y) / distance)
    }

    pub fn moved_along(&self, direction: &Point, distance: f32) -> Point {
        Point::new(self.x + direction.x * distance, self.y + direction.y * distance)
    }
}
