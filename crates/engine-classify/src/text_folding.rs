use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// Lowercases a text and removes accents, so "FACTURA ELECTRÓNICA" and "factura electronica" are the same.
pub struct TextFolding;

impl TextFolding {
    pub fn fold(text: &str) -> String {
        text.to_lowercase()
            .nfd()
            .filter(|character| !is_combining_mark(*character))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_case_and_accents() {
        assert_eq!(TextFolding::fold("FACTURA ELECTRÓNICA"), "factura electronica");
        assert_eq!(TextFolding::fold("Société Générale"), "societe generale");
    }

    #[test]
    fn leaves_other_scripts_readable() {
        assert_eq!(TextFolding::fold("发票 Счёт"), "发票 счет");
    }
}
