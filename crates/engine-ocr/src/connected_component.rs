use engine_core::BoundingBox;

/// A group of adjacent pixels above the detection threshold.
pub struct ConnectedComponent {
    left: usize,
    top: usize,
    right: usize,
    bottom: usize,
    probability_sum: f32,
    pixel_count: usize,
}

impl ConnectedComponent {
    pub fn starting_at(row: usize, column: usize, probability: f32) -> Self {
        Self {
            left: column,
            top: row,
            right: column,
            bottom: row,
            probability_sum: probability,
            pixel_count: 1,
        }
    }

    pub fn include(&mut self, row: usize, column: usize, probability: f32) {
        self.left = self.left.min(column);
        self.top = self.top.min(row);
        self.right = self.right.max(column);
        self.bottom = self.bottom.max(row);
        self.probability_sum += probability;
        self.pixel_count += 1;
    }

    pub fn shortest_side(&self) -> f32 {
        let width = self.right - self.left + 1;
        let height = self.bottom - self.top + 1;
        width.min(height) as f32
    }

    pub fn mean_probability(&self) -> f32 {
        self.probability_sum / self.pixel_count as f32
    }

    /// Box covering every pixel of the component, edges included.
    pub fn bounding_box(&self) -> BoundingBox {
        BoundingBox::new(
            self.left as f32,
            self.top as f32,
            (self.right + 1) as f32,
            (self.bottom + 1) as f32,
        )
    }
}
