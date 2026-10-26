use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::document_line::DocumentLine;
use crate::party::Party;
use crate::tax_line::TaxLine;

/// What a commercial document (invoice, credit note, receipt...) states. Whatever the document does not state is
/// `null` or empty: nothing is guessed or calculated here, and nothing is specific to a country.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct ExtractedDocument {
    /// The number printed on the document.
    pub number: Option<String>,
    /// The date the document was issued, as YYYY-MM-DD.
    pub issue_date: Option<String>,
    /// The date payment is due, as YYYY-MM-DD.
    pub due_date: Option<String>,
    /// ISO 4217 code of the currency of the amounts: USD, EUR, PEN...
    pub currency: Option<String>,
    /// Who issued the document.
    pub supplier: Party,
    /// Who the document is addressed to.
    pub customer: Party,
    pub lines: Vec<DocumentLine>,
    /// The tax summary: one entry per tax or rate.
    pub taxes: Vec<TaxLine>,
    /// The sum of the rows before tax.
    pub subtotal: Option<f64>,
    /// The sum of all taxes.
    pub tax_total: Option<f64>,
    /// Document-level discounts (negative), charges such as shipping (positive) and rounding, added together.
    pub adjustment_total: Option<f64>,
    /// The amount to pay, with tax.
    pub total: Option<f64>,
    pub payment_terms: Option<String>,
}
