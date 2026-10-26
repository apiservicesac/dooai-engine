use engine_core::EngineError;
use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_syntax::Pdf;
use hayro::vello_cpu::color::palette::css::WHITE;
use hayro::{PixmapSettings, RenderCache, RenderSettings, render};

use crate::settings::PdfSettings;

/// Draws the pages of a PDF as images, so scans can be read by OCR.
pub struct PdfRasterizer {
    pdf: Pdf,
}

impl PdfRasterizer {
    pub fn open(bytes: &[u8]) -> Result<Self, EngineError> {
        let pdf = Pdf::new(bytes.to_vec())
            .map_err(|error| EngineError::InvalidDocument(format!("the PDF cannot be drawn: {error:?}")))?;
        Ok(Self { pdf })
    }

    /// The page as a PNG on a white background. `page_index` starts at 0.
    pub fn render_png(&self, page_index: usize) -> Result<Vec<u8>, EngineError> {
        let pages = self.pdf.pages();
        let page = pages
            .get(page_index)
            .ok_or_else(|| EngineError::InvalidDocument(format!("the PDF has no page {}", page_index + 1)))?;

        let (width, height) = page.render_dimensions();
        let scale = (PdfSettings::RENDER_DPI / PdfSettings::POINTS_PER_INCH)
            .min(PdfSettings::MAX_RENDER_SIDE / width.max(height));
        let pixmap = render(
            page,
            &RenderCache::new(),
            &InterpreterSettings::default(),
            &RenderSettings::default(),
            &PixmapSettings {
                x_scale: scale,
                y_scale: scale,
                bg_color: WHITE,
            },
        );
        pixmap.into_png().map_err(|error| {
            EngineError::InvalidDocument(format!("page {} could not be encoded: {error}", page_index + 1))
        })
    }
}
