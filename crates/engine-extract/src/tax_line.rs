use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One tax of the document's tax summary.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct TaxLine {
    /// What the document calls it: VAT, IVA, IGV, sales tax...
    pub name: Option<String>,
    /// Rate in percent: 18 for 18%.
    pub rate: Option<f64>,
    /// The amount the tax is applied to.
    pub base: Option<f64>,
    pub amount: Option<f64>,
}
