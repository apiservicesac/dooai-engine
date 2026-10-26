use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// The document contradicts itself or states something impossible: do not trust it without a person.
    Error,
    /// Something expected is missing.
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueCode {
    LineAmount,
    SubtotalMismatch,
    TaxTotalMismatch,
    TaxAmount,
    TotalMismatch,
    InvalidCurrency,
    InvalidDate,
    DueBeforeIssue,
    MissingTotal,
    MissingNumber,
    MissingIssueDate,
    MissingSupplier,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValidationIssue {
    pub code: IssueCode,
    pub severity: Severity,
    pub message: String,
}

/// What checking a document found.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Validation {
    /// How many arithmetic checks had enough numbers to run. With none, nothing was verified.
    pub checks_performed: usize,
    pub issues: Vec<ValidationIssue>,
}

impl Validation {
    pub fn errors(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues.iter().filter(|issue| issue.severity == Severity::Error)
    }

    /// No contradictions were found (missing fields do not count).
    pub fn is_consistent(&self) -> bool {
        self.errors().next().is_none()
    }

    /// Consistent, and at least one check actually ran.
    pub fn is_verified(&self) -> bool {
        self.is_consistent() && self.checks_performed > 0
    }
}
