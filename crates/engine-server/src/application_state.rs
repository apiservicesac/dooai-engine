use std::sync::Arc;

use engine_document::DocumentReader;

/// What every request handler shares.
#[derive(Clone)]
pub struct ApplicationState {
    pub document_reader: Arc<DocumentReader>,
}
