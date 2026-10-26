use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::tax_id::TaxId;

/// A company or person named on a document.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct Party {
    pub name: Option<String>,
    pub tax_ids: Vec<TaxId>,
    pub address: Option<String>,
}
