use serde::Serialize;

use crate::OcrDocument;
use crate::settings::DocumentLayout;

/// One page of a [`ReadDocument`], numbered from 1.
#[derive(Debug, Clone, Serialize)]
pub struct ReadPage {
    pub page: usize,
    #[serde(flatten)]
    pub content: OcrDocument,
}

/// Everything read from one file: an image is a document of one page, a PDF has one page per sheet.
#[derive(Debug, Clone, Serialize)]
pub struct ReadDocument {
    /// The text of every page, in order.
    pub text: String,
    pub pages: Vec<ReadPage>,
}

impl ReadDocument {
    pub fn from_pages(pages: Vec<ReadPage>) -> Self {
        let texts: Vec<&str> = pages.iter().map(|page| page.content.text.as_str()).collect();
        Self {
            text: texts.join(DocumentLayout::PAGE_SEPARATOR),
            pages,
        }
    }

    pub fn single_page(content: OcrDocument) -> Self {
        Self::from_pages(vec![ReadPage { page: 1, content }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_the_text_of_the_pages_in_order() {
        let pages = vec![
            ReadPage {
                page: 1,
                content: OcrDocument::from_text("first".to_string(), "test"),
            },
            ReadPage {
                page: 2,
                content: OcrDocument::from_text("second".to_string(), "test"),
            },
        ];

        assert_eq!(ReadDocument::from_pages(pages).text, "first\n\nsecond");
    }

    #[test]
    fn an_image_is_a_document_of_one_page() {
        let document = ReadDocument::single_page(OcrDocument::from_text("only".to_string(), "test"));

        assert_eq!(
            (document.pages.len(), document.pages[0].page, document.text.as_str()),
            (1, 1, "only")
        );
    }
}
