use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A tax identifier of a company, whatever the country calls it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct TaxId {
    /// What the document calls it, as printed: RUC, NIF, CIF, VAT, EIN, CNPJ, GSTIN, RFC...
    pub scheme: Option<String>,
    /// The number, as printed.
    pub value: String,
}
