use crate::OcrLine;

/// Puts lines in reading order: rows from top to bottom, each row from left to right.
/// Positions are measured in the page's own frame, so a tilted photo reads like a straight page.
pub struct ReadingOrder;

struct PlacedLine {
    line: OcrLine,
    across: f32,
    down: f32,
}

impl ReadingOrder {
    pub fn rows(lines: Vec<OcrLine>) -> Vec<Vec<OcrLine>> {
        let page_tilt = Self::page_tilt(&lines);
        let mut placed: Vec<PlacedLine> = lines.into_iter().map(|line| Self::place(line, page_tilt)).collect();
        placed.sort_by(|first, second| first.down.total_cmp(&second.down));

        let mut rows: Vec<Vec<PlacedLine>> = Vec::new();
        for candidate in placed {
            match rows.last_mut() {
                Some(row) if Self::sits_on_row(&row[0], &candidate) => row.push(candidate),
                _ => rows.push(vec![candidate]),
            }
        }
        rows.into_iter()
            .map(|mut row| {
                row.sort_by(|first, second| first.across.total_cmp(&second.across));
                row.into_iter().map(|placed_line| placed_line.line).collect()
            })
            .collect()
    }

    /// The median tilt of the horizontal lines; vertical lines say nothing about the page.
    fn page_tilt(lines: &[OcrLine]) -> f32 {
        let mut tilts: Vec<f32> = lines
            .iter()
            .filter(|line| line.polygon.width() >= line.polygon.height())
            .map(|line| line.polygon.tilt_radians())
            .collect();
        tilts.sort_by(|first, second| first.total_cmp(second));
        tilts.get(tilts.len() / 2).copied().unwrap_or(0.0)
    }

    fn place(line: OcrLine, page_tilt: f32) -> PlacedLine {
        let center = line.polygon.center();
        let (sine, cosine) = page_tilt.sin_cos();
        PlacedLine {
            across: center.x * cosine + center.y * sine,
            down: center.y * cosine - center.x * sine,
            line,
        }
    }

    fn sits_on_row(reference: &PlacedLine, candidate: &PlacedLine) -> bool {
        (candidate.down - reference.down).abs() <= reference.line.polygon.height() / 2.0
    }
}
