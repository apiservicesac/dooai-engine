use serde::Serialize;

use crate::OcrLine;
use crate::settings::DocumentLayout;

/// Result of reading one image: lines in reading order plus their joined text.
#[derive(Debug, Clone, Serialize)]
pub struct OcrDocument {
    pub text: String,
    pub confidence: f32,
    pub lines: Vec<OcrLine>,
}

impl OcrDocument {
    /// Orders lines top to bottom, then left to right inside each visual row.
    pub fn from_lines(mut lines: Vec<OcrLine>) -> Self {
        lines.sort_by(|first, second| first.bounding_box.top.total_cmp(&second.bounding_box.top));

        let mut rows: Vec<Vec<OcrLine>> = Vec::new();
        for line in lines {
            match rows.last_mut() {
                Some(row) if row[0].bounding_box.shares_row_with(&line.bounding_box) => row.push(line),
                _ => rows.push(vec![line]),
            }
        }
        for row in &mut rows {
            row.sort_by(|first, second| first.bounding_box.left.total_cmp(&second.bounding_box.left));
        }

        let text = rows
            .iter()
            .map(|row| {
                let texts: Vec<&str> = row.iter().map(|line| line.text.as_str()).collect();
                texts.join(DocumentLayout::COLUMN_SEPARATOR)
            })
            .collect::<Vec<_>>()
            .join(DocumentLayout::ROW_SEPARATOR);
        let ordered_lines: Vec<OcrLine> = rows.into_iter().flatten().collect();

        Self {
            text,
            confidence: Self::average_confidence(&ordered_lines),
            lines: ordered_lines,
        }
    }

    fn average_confidence(lines: &[OcrLine]) -> f32 {
        if lines.is_empty() {
            return 0.0;
        }
        lines.iter().map(|line| line.confidence).sum::<f32>() / lines.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BoundingBox;

    fn line(text: &str, left: f32, top: f32) -> OcrLine {
        OcrLine {
            text: text.to_string(),
            confidence: 0.9,
            bounding_box: BoundingBox::new(left, top, left + 80.0, top + 20.0),
        }
    }

    #[test]
    fn joins_lines_of_the_same_row_left_to_right() {
        let document = OcrDocument::from_lines(vec![
            line("Total", 300.0, 202.0),
            line("Invoice 001", 10.0, 10.0),
            line("USD 118.00", 400.0, 200.0),
        ]);

        assert_eq!(document.text, "Invoice 001\nTotal\tUSD 118.00");
    }

    #[test]
    fn empty_document_has_zero_confidence() {
        let document = OcrDocument::from_lines(Vec::new());

        assert_eq!(document.text, "");
        assert_eq!(document.confidence, 0.0);
    }
}
