use serde::Serialize;

/// Axis-aligned rectangle in pixels, with the origin at the top-left corner.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct BoundingBox {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl BoundingBox {
    pub fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    pub fn height(&self) -> f32 {
        self.bottom - self.top
    }

    pub fn expanded(&self, distance: f32) -> Self {
        Self::new(
            self.left - distance,
            self.top - distance,
            self.right + distance,
            self.bottom + distance,
        )
    }

    pub fn scaled(&self, horizontal_factor: f32, vertical_factor: f32) -> Self {
        Self::new(
            self.left * horizontal_factor,
            self.top * vertical_factor,
            self.right * horizontal_factor,
            self.bottom * vertical_factor,
        )
    }

    pub fn clamped(&self, max_width: f32, max_height: f32) -> Self {
        Self::new(
            self.left.clamp(0.0, max_width),
            self.top.clamp(0.0, max_height),
            self.right.clamp(0.0, max_width),
            self.bottom.clamp(0.0, max_height),
        )
    }

    /// True when the vertical center of `other` falls inside this box's height.
    pub fn shares_row_with(&self, other: &BoundingBox) -> bool {
        let other_center = (other.top + other.bottom) / 2.0;
        other_center >= self.top && other_center <= self.bottom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scales_and_clamps_to_image_bounds() {
        let box_in_resized_image = BoundingBox::new(10.0, 20.0, 110.0, 70.0);

        let restored = box_in_resized_image.scaled(2.0, 2.0).clamped(150.0, 100.0);

        assert_eq!(restored, BoundingBox::new(20.0, 40.0, 150.0, 100.0));
    }

    #[test]
    fn detects_boxes_on_the_same_row() {
        let reference = BoundingBox::new(0.0, 100.0, 50.0, 120.0);

        assert!(reference.shares_row_with(&BoundingBox::new(60.0, 102.0, 90.0, 122.0)));
        assert!(!reference.shares_row_with(&BoundingBox::new(0.0, 140.0, 50.0, 160.0)));
    }
}
