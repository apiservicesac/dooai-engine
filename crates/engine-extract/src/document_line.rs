use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One row of what is being sold.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct DocumentLine {
    pub description: Option<String>,
    /// The product or service code printed on the row.
    pub code: Option<String>,
    pub quantity: Option<f64>,
    /// Unit of measure as printed (kg, pcs, hours...).
    pub unit: Option<String>,
    /// Price of one unit before tax.
    pub unit_price: Option<f64>,
    /// Discount of the row as an amount, not a percentage.
    pub discount: Option<f64>,
    /// Tax rate of the row in percent, if the row states it.
    pub tax_rate: Option<f64>,
    /// What the row costs before tax, after its discount.
    pub amount: Option<f64>,
}
