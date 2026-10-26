use chrono::NaiveDate;

use crate::currency_codes::CurrencyCodes;
use crate::extracted_document::ExtractedDocument;
use crate::settings::{DateFormat, Tolerance};
use crate::validation::{IssueCode, Severity, Validation, ValidationIssue};

/// Checks that what was read from a document is consistent: every row, the sums, the taxes and the total add up,
/// the currency exists and the dates are real. It never changes the document and knows no country's rules.
pub struct DocumentValidator {
    checks_performed: usize,
    issues: Vec<ValidationIssue>,
}

impl DocumentValidator {
    pub fn validate(document: &ExtractedDocument) -> Validation {
        let mut validator = Self {
            checks_performed: 0,
            issues: Vec::new(),
        };
        validator.check_lines(document);
        validator.check_totals(document);
        validator.check_taxes(document);
        validator.check_currency(document);
        validator.check_dates(document);
        validator.check_completeness(document);
        Validation {
            checks_performed: validator.checks_performed,
            issues: validator.issues,
        }
    }

    fn check_lines(&mut self, document: &ExtractedDocument) {
        for (index, line) in document.lines.iter().enumerate() {
            if let (Some(quantity), Some(unit_price), Some(amount)) = (line.quantity, line.unit_price, line.amount) {
                let expected = quantity * unit_price - line.discount.unwrap_or(0.0);
                self.compare(
                    expected,
                    amount,
                    Tolerance::LINE,
                    IssueCode::LineAmount,
                    format!(
                        "row {}: {quantity} x {unit_price} should be {expected:.2}, the row says {amount:.2}",
                        index + 1
                    ),
                );
            }
        }
    }

    fn check_totals(&mut self, document: &ExtractedDocument) {
        let tolerance = Self::total_tolerance(document);
        let lines_sum = Self::sum_of(document.lines.iter().map(|line| line.amount));
        let taxes_sum = Self::sum_of(document.taxes.iter().map(|tax| tax.amount));

        if let (Some(sum), Some(subtotal)) = (lines_sum, document.subtotal) {
            self.compare(
                sum,
                subtotal,
                tolerance,
                IssueCode::SubtotalMismatch,
                format!("the rows add up to {sum:.2}, the subtotal says {subtotal:.2}"),
            );
        }
        if let (Some(sum), Some(tax_total)) = (taxes_sum, document.tax_total) {
            self.compare(
                sum,
                tax_total,
                tolerance,
                IssueCode::TaxTotalMismatch,
                format!("the taxes add up to {sum:.2}, the tax total says {tax_total:.2}"),
            );
        }
        let subtotal = document.subtotal.or(lines_sum);
        let tax_total = document.tax_total.or(taxes_sum);
        if let (Some(subtotal), Some(tax_total), Some(total)) = (subtotal, tax_total, document.total) {
            let adjustment = document.adjustment_total.unwrap_or(0.0);
            let expected = subtotal + tax_total + adjustment;
            self.compare(expected, total, tolerance, IssueCode::TotalMismatch, format!("subtotal {subtotal:.2} + tax {tax_total:.2} + adjustments {adjustment:.2} is {expected:.2}, the total says {total:.2}"));
        }
    }

    fn check_taxes(&mut self, document: &ExtractedDocument) {
        let tolerance = Self::total_tolerance(document);
        for (index, tax) in document.taxes.iter().enumerate() {
            if let (Some(rate), Some(base), Some(amount)) = (tax.rate, tax.base, tax.amount) {
                let expected = base * rate / 100.0;
                self.compare(
                    expected,
                    amount,
                    tolerance,
                    IssueCode::TaxAmount,
                    format!(
                        "tax {}: {rate}% of {base:.2} is {expected:.2}, the document says {amount:.2}",
                        index + 1
                    ),
                );
            }
        }
    }

    fn check_currency(&mut self, document: &ExtractedDocument) {
        if let Some(code) = &document.currency
            && !CurrencyCodes::is_valid(code)
        {
            self.error(
                IssueCode::InvalidCurrency,
                format!("'{code}' is not an ISO 4217 currency code"),
            );
        }
    }

    fn check_dates(&mut self, document: &ExtractedDocument) {
        let issue = self.parse_date(&document.issue_date, "issue date");
        let due = self.parse_date(&document.due_date, "due date");
        if let (Some(issue), Some(due)) = (issue, due) {
            self.checks_performed += 1;
            if due < issue {
                self.error(
                    IssueCode::DueBeforeIssue,
                    format!("the due date {due} is before the issue date {issue}"),
                );
            }
        }
    }

    fn check_completeness(&mut self, document: &ExtractedDocument) {
        if document.total.is_none() {
            self.warn(IssueCode::MissingTotal, "the total was not found");
        }
        if document.number.as_deref().is_none_or(|number| number.trim().is_empty()) {
            self.warn(IssueCode::MissingNumber, "the document number was not found");
        }
        if document.issue_date.is_none() {
            self.warn(IssueCode::MissingIssueDate, "the issue date was not found");
        }
        if document
            .supplier
            .name
            .as_deref()
            .is_none_or(|name| name.trim().is_empty())
        {
            self.warn(IssueCode::MissingSupplier, "the issuer was not found");
        }
    }

    fn parse_date(&mut self, value: &Option<String>, name: &str) -> Option<NaiveDate> {
        let text = value.as_deref()?;
        match NaiveDate::parse_from_str(text, DateFormat::ISO) {
            Ok(date) => Some(date),
            Err(_) => {
                self.error(
                    IssueCode::InvalidDate,
                    format!("the {name} '{text}' is not a real date in YYYY-MM-DD form"),
                );
                None
            }
        }
    }

    /// The sum of the amounts, only when every one of them is known.
    fn sum_of(amounts: impl Iterator<Item = Option<f64>>) -> Option<f64> {
        let amounts: Vec<Option<f64>> = amounts.collect();
        if amounts.is_empty() || amounts.iter().any(Option::is_none) {
            return None;
        }
        Some(amounts.iter().flatten().sum())
    }

    fn total_tolerance(document: &ExtractedDocument) -> f64 {
        Tolerance::TOTAL_BASE + Tolerance::TOTAL_PER_LINE * document.lines.len() as f64
    }

    fn compare(&mut self, expected: f64, found: f64, tolerance: f64, code: IssueCode, message: String) {
        self.checks_performed += 1;
        if (expected - found).abs() > tolerance {
            self.error(code, message);
        }
    }

    fn error(&mut self, code: IssueCode, message: String) {
        self.issues.push(ValidationIssue {
            code,
            severity: Severity::Error,
            message,
        });
    }

    fn warn(&mut self, code: IssueCode, message: &str) {
        self.issues.push(ValidationIssue {
            code,
            severity: Severity::Warning,
            message: message.to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document_line::DocumentLine;
    use crate::party::Party;
    use crate::tax_line::TaxLine;

    fn row(quantity: f64, unit_price: f64, amount: f64) -> DocumentLine {
        DocumentLine {
            quantity: Some(quantity),
            unit_price: Some(unit_price),
            amount: Some(amount),
            ..DocumentLine::default()
        }
    }

    /// Two rows of 100.00 and 50.00, 18% tax, total 177.00.
    fn consistent_invoice() -> ExtractedDocument {
        ExtractedDocument {
            number: Some("F001-123".to_string()),
            issue_date: Some("2026-10-25".to_string()),
            due_date: Some("2026-11-24".to_string()),
            currency: Some("PEN".to_string()),
            supplier: Party {
                name: Some("Andina SAC".to_string()),
                ..Party::default()
            },
            lines: vec![row(2.0, 50.0, 100.0), row(1.0, 50.0, 50.0)],
            taxes: vec![TaxLine {
                name: Some("IGV".to_string()),
                rate: Some(18.0),
                base: Some(150.0),
                amount: Some(27.0),
            }],
            subtotal: Some(150.0),
            tax_total: Some(27.0),
            total: Some(177.0),
            ..ExtractedDocument::default()
        }
    }

    fn codes(validation: &Validation) -> Vec<IssueCode> {
        validation.issues.iter().map(|issue| issue.code).collect()
    }

    #[test]
    fn a_document_that_adds_up_is_verified() {
        let validation = DocumentValidator::validate(&consistent_invoice());

        assert!(validation.is_verified(), "{validation:?}");
        assert!(validation.issues.is_empty());
        assert!(validation.checks_performed >= 6);
    }

    #[test]
    fn finds_a_row_that_does_not_multiply() {
        let mut document = consistent_invoice();
        document.lines[0].amount = Some(120.0);

        let validation = DocumentValidator::validate(&document);

        assert!(codes(&validation).contains(&IssueCode::LineAmount), "{validation:?}");
        assert!(codes(&validation).contains(&IssueCode::SubtotalMismatch));
        assert!(!validation.is_consistent());
    }

    #[test]
    fn finds_a_total_that_does_not_add_up() {
        let mut document = consistent_invoice();
        document.total = Some(188.0);

        assert_eq!(
            codes(&DocumentValidator::validate(&document)),
            [IssueCode::TotalMismatch]
        );
    }

    #[test]
    fn document_level_adjustments_are_part_of_the_total() {
        let mut document = consistent_invoice();
        document.adjustment_total = Some(-10.0);
        document.total = Some(167.0);

        assert!(DocumentValidator::validate(&document).is_verified());
    }

    #[test]
    fn finds_a_tax_that_is_not_its_rate_of_its_base() {
        let mut document = consistent_invoice();
        document.taxes[0].amount = Some(30.0);

        assert!(codes(&DocumentValidator::validate(&document)).contains(&IssueCode::TaxAmount));
    }

    #[test]
    fn a_rounding_difference_of_a_cent_is_tolerated() {
        let mut document = consistent_invoice();
        document.total = Some(177.01);

        assert!(DocumentValidator::validate(&document).is_verified());
    }

    #[test]
    fn rejects_invented_currencies_and_impossible_dates() {
        let mut document = consistent_invoice();
        document.currency = Some("SOLES".to_string());
        document.issue_date = Some("2026-02-30".to_string());

        let found = codes(&DocumentValidator::validate(&document));

        assert!(
            found.contains(&IssueCode::InvalidCurrency) && found.contains(&IssueCode::InvalidDate),
            "{found:?}"
        );
    }

    #[test]
    fn a_due_date_before_the_issue_date_is_an_error() {
        let mut document = consistent_invoice();
        document.due_date = Some("2026-09-01".to_string());

        assert!(codes(&DocumentValidator::validate(&document)).contains(&IssueCode::DueBeforeIssue));
    }

    #[test]
    fn with_too_few_numbers_nothing_is_verified_and_missing_fields_are_warnings() {
        let document = ExtractedDocument {
            total: Some(10.0),
            ..ExtractedDocument::default()
        };

        let validation = DocumentValidator::validate(&document);

        assert_eq!(validation.checks_performed, 0);
        assert!(validation.is_consistent() && !validation.is_verified());
        assert!(codes(&validation).contains(&IssueCode::MissingNumber));
        assert!(
            validation
                .issues
                .iter()
                .all(|issue| issue.severity == Severity::Warning)
        );
    }

    #[test]
    fn the_total_can_be_checked_from_the_rows_and_taxes_when_the_summary_is_missing() {
        let mut document = consistent_invoice();
        document.subtotal = None;
        document.tax_total = None;
        document.total = Some(190.0);

        assert!(codes(&DocumentValidator::validate(&document)).contains(&IssueCode::TotalMismatch));
    }
}
