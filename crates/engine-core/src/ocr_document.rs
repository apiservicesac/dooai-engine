use serde::Serialize;

use crate::OcrLine;
use crate::reading_order::ReadingOrder;
use crate::settings::DocumentLayout;

/// Result of reading one image.
#[derive(Debug, Clone, Serialize)]
pub struct OcrDocument {
    /// Text in reading order, one line per visual row.
    pub text: String,
    /// Mean confidence of the lines; `None` when the engine reports none.
    pub confidence: Option<f32>,
    /// Lines with their geometry; empty when the engine gives none.
    pub lines: Vec<OcrLine>,
    /// Clockwise rotation applied to the image to make the text upright. Line geometry refers to that rotated image.
    pub rotation_degrees: u16,
    /// What read the image, e.g. `paddle-ocr/cyrillic`.
    pub engine: String,
}

impl OcrDocument {
    pub fn from_lines(lines: Vec<OcrLine>, engine: impl Into<String>, rotation_degrees: u16) -> Self {
        let rows = ReadingOrder::rows(lines);
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
            rotation_degrees,
            engine: engine.into(),
        }
    }

    /// For engines that return text only, such as vision-language models.
    pub fn from_text(text: String, engine: impl Into<String>) -> Self {
        Self {
            text,
            confidence: None,
            lines: Vec::new(),
            rotation_degrees: 0,
            engine: engine.into(),
        }
    }

    fn average_confidence(lines: &[OcrLine]) -> Option<f32> {
        if lines.is_empty() {
            return None;
        }
        Some(lines.iter().map(|line| line.confidence).sum::<f32>() / lines.len() as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Point, TextQuad};

    fn line(text: &str, left: f32, top: f32) -> OcrLine {
        let quad = TextQuad::from_unordered([
            Point::new(left, top),
            Point::new(left + 80.0, top),
            Point::new(left + 80.0, top + 20.0),
            Point::new(left, top + 20.0),
        ]);
        OcrLine::new(text.to_string(), 0.9, quad)
    }

    #[test]
    fn joins_lines_of_the_same_row_left_to_right() {
        let document = OcrDocument::from_lines(
            vec![
                line("Total", 300.0, 202.0),
                line("Invoice 001", 10.0, 10.0),
                line("USD 118.00", 400.0, 200.0),
            ],
            "test",
            0,
        );

        assert_eq!(document.text, "Invoice 001\nTotal\tUSD 118.00");
    }

    #[test]
    fn reads_a_tilted_page_row_by_row() {
        // Rows are 40 px apart but the page descends 12 px per 100 px to the right.
        let tilted = |text: &str, left: f32, top: f32| {
            let slope = 0.12;
            let drop = left * slope;
            let quad = TextQuad::from_unordered([
                Point::new(left, top + drop),
                Point::new(left + 80.0, top + drop + 80.0 * slope),
                Point::new(left + 80.0, top + drop + 80.0 * slope + 20.0),
                Point::new(left, top + drop + 20.0),
            ]);
            OcrLine::new(text.to_string(), 0.9, quad)
        };

        let document = OcrDocument::from_lines(
            vec![
                tilted("Subtotal", 0.0, 100.0),
                tilted("USD 100.00", 400.0, 100.0),
                tilted("Total", 0.0, 140.0),
                tilted("USD 118.00", 400.0, 140.0),
            ],
            "test",
            0,
        );

        assert_eq!(document.text, "Subtotal\tUSD 100.00\nTotal\tUSD 118.00");
    }

    #[test]
    fn empty_document_has_no_confidence() {
        let document = OcrDocument::from_lines(Vec::new(), "test", 0);

        assert_eq!(document.text, "");
        assert_eq!(document.confidence, None);
    }
}
