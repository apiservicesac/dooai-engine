use engine_core::EngineError;
use schemars::schema_for;
use serde_json::Value;

use crate::document_validator::DocumentValidator;
use crate::extracted_document::ExtractedDocument;
use crate::extraction::Extraction;
use crate::language_model::LanguageModel;
use crate::settings::ExtractionSettings;

/// Reads a document's text with a language model, then checks the arithmetic itself. When the numbers do not add up
/// it shows the model exactly what is wrong and asks again, once, before giving the result with its issues.
pub struct LanguageModelExtractor {
    model: LanguageModel,
    schema: Value,
}

impl LanguageModelExtractor {
    pub fn new(model: LanguageModel) -> Self {
        let schema = serde_json::to_value(schema_for!(ExtractedDocument)).expect("the schema is valid JSON");
        Self { model, schema }
    }

    pub fn model_name(&self) -> &str {
        self.model.name()
    }

    pub fn extract(&self, text: &str) -> Result<Extraction, EngineError> {
        let text: String = text.chars().take(ExtractionSettings::MAX_TEXT_CHARACTERS).collect();
        let mut prompt = Self::first_prompt(&text);
        let mut best: Option<Extraction> = None;

        for attempt in 1..=ExtractionSettings::MAX_REPAIR_ATTEMPTS + 1 {
            let answer = self
                .model
                .complete_json(ExtractionSettings::SYSTEM_PROMPT, &prompt, &self.schema)?;
            match serde_json::from_str::<ExtractedDocument>(&answer) {
                Ok(document) => {
                    let validation = DocumentValidator::validate(&document);
                    let consistent = validation.is_consistent();
                    prompt = Self::repair_prompt(
                        &text,
                        &answer,
                        &validation
                            .errors()
                            .map(|issue| issue.message.as_str())
                            .collect::<Vec<_>>(),
                    );
                    let candidate = Extraction {
                        document,
                        validation,
                        attempts: attempt,
                    };
                    if best.as_ref().is_none_or(|current| {
                        candidate.validation.errors().count() < current.validation.errors().count()
                    }) {
                        best = Some(candidate);
                    }
                    if consistent {
                        break;
                    }
                }
                Err(error) => {
                    prompt = Self::repair_prompt(
                        &text,
                        &answer,
                        &[&format!("the JSON does not follow the schema: {error}")],
                    );
                }
            }
        }
        best.map(|mut extraction| {
            extraction.attempts = extraction.attempts.max(1);
            extraction
        })
        .ok_or_else(|| EngineError::Inference("the language model did not answer with the expected JSON".to_string()))
    }

    fn first_prompt(text: &str) -> String {
        format!("Document text:\n<document>\n{text}\n</document>")
    }

    fn repair_prompt(text: &str, previous_answer: &str, problems: &[&str]) -> String {
        let list: Vec<String> = problems.iter().map(|problem| format!("- {problem}")).collect();
        format!(
            "{}\n\nYour previous answer was:\n{previous_answer}\n\nIt has these problems:\n{}\n\nRead the document again and answer with \
corrected JSON. Change only what the document says differently: if the document's own numbers really do not add up, \
keep the numbers as printed.",
            Self::first_prompt(text),
            list.join("\n")
        )
    }
}
