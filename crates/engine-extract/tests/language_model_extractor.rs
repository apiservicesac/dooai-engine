use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

use engine_core::EngineError;
use engine_extract::{IssueCode, LanguageModel, LanguageModelExtractor, LanguageModelSettings};
use serde_json::json;

/// A server that speaks just enough of the chat completions API: it answers each request with the next canned
/// answer and remembers what it was sent.
struct StubLanguageModel {
    base_url: String,
    requests: Arc<Mutex<Vec<String>>>,
}

impl StubLanguageModel {
    fn answering(answers: Vec<(u16, String)>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base_url = format!("http://{}/v1", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&requests);
        thread::spawn(move || {
            for (status, content) in answers {
                let Ok((mut stream, _)) = listener.accept() else { return };
                let request = Self::read_request(&mut stream);
                seen.lock().unwrap().push(request);
                let body = if status == 200 {
                    json!({"choices": [{"message": {"content": content}}]}).to_string()
                } else {
                    json!({"error": {"message": content}}).to_string()
                };
                let response = format!(
                    "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(response.as_bytes()).unwrap();
            }
        });
        Self { base_url, requests }
    }

    fn read_request(stream: &mut std::net::TcpStream) -> String {
        let mut received = Vec::new();
        let mut buffer = [0_u8; 4096];
        loop {
            let count = stream.read(&mut buffer).unwrap();
            received.extend_from_slice(&buffer[..count]);
            let text = String::from_utf8_lossy(&received);
            if let Some(split) = text.find("\r\n\r\n") {
                let length = text[..split]
                    .lines()
                    .find_map(|line| {
                        line.to_lowercase()
                            .strip_prefix("content-length:")
                            .map(|value| value.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                if received.len() >= split + 4 + length {
                    return text.to_string();
                }
            }
            if count == 0 {
                return text.to_string();
            }
        }
    }

    fn extractor(&self) -> LanguageModelExtractor {
        LanguageModelExtractor::new(LanguageModel::new(LanguageModelSettings {
            base_url: self.base_url.clone(),
            model: "test-model".to_string(),
            api_key: Some("secret-key".to_string()),
        }))
    }

    fn requests(&self) -> Vec<String> {
        self.requests.lock().unwrap().clone()
    }
}

fn invoice_json(total: f64) -> String {
    json!({
        "number": "F001-123", "issue_date": "2026-10-25", "currency": "PEN",
        "supplier": {"name": "Andina SAC", "tax_ids": [{"scheme": "RUC", "value": "20512345678"}]},
        "lines": [{"description": "Chairs", "quantity": 2.0, "unit_price": 50.0, "amount": 100.0}],
        "taxes": [{"name": "IGV", "rate": 18.0, "base": 100.0, "amount": 18.0}],
        "subtotal": 100.0, "tax_total": 18.0, "total": total
    })
    .to_string()
}

const DOCUMENT_TEXT: &str = "FACTURA F001-123 Andina SAC RUC 20512345678 2 Chairs 100.00 IGV 18.00 Total 118.00";

#[test]
fn reads_a_consistent_answer_in_one_attempt_and_sends_what_it_should() {
    let server = StubLanguageModel::answering(vec![(200, invoice_json(118.0))]);

    let extraction = server.extractor().extract(DOCUMENT_TEXT).unwrap();

    assert_eq!(extraction.attempts, 1);
    assert!(extraction.validation.is_verified(), "{:?}", extraction.validation);
    assert_eq!(extraction.document.supplier.tax_ids[0].scheme.as_deref(), Some("RUC"));
    let request = &server.requests()[0];
    assert!(
        request.to_lowercase().contains("authorization: bearer secret-key"),
        "{request}"
    );
    let body: serde_json::Value = serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
    assert_eq!(body["model"], "test-model");
    assert_eq!(body["temperature"], 0.0);
    assert_eq!(body["response_format"]["type"], "json_schema");
    assert!(
        body["response_format"]["json_schema"]["schema"]
            .to_string()
            .contains("tax_ids")
    );
    let user = body["messages"][1]["content"].as_str().unwrap();
    assert!(user.contains("<document>") && user.contains("Total 118.00"), "{user}");
}

#[test]
fn shows_the_model_what_is_wrong_and_takes_the_corrected_answer() {
    let server = StubLanguageModel::answering(vec![(200, invoice_json(150.0)), (200, invoice_json(118.0))]);

    let extraction = server.extractor().extract(DOCUMENT_TEXT).unwrap();

    assert_eq!(extraction.attempts, 2);
    assert_eq!(extraction.document.total, Some(118.0));
    assert!(extraction.validation.is_verified());
    let repair = &server.requests()[1];
    assert!(
        repair.contains("previous answer") && repair.contains("the total says 150.00"),
        "{repair}"
    );
}

#[test]
fn after_one_repair_it_returns_the_result_with_its_issues() {
    let server = StubLanguageModel::answering(vec![(200, invoice_json(150.0)), (200, invoice_json(150.0))]);

    let extraction = server.extractor().extract(DOCUMENT_TEXT).unwrap();

    assert_eq!(
        extraction.attempts, 1,
        "the best attempt is the first of two equal ones"
    );
    assert!(!extraction.validation.is_consistent());
    assert!(
        extraction
            .validation
            .issues
            .iter()
            .any(|issue| issue.code == IssueCode::TotalMismatch)
    );
    assert_eq!(server.requests().len(), 2, "it asks once more, not forever");
}

#[test]
fn an_answer_that_does_not_follow_the_schema_is_repaired() {
    let server = StubLanguageModel::answering(vec![
        (200, r#"{"total": "one hundred"}"#.to_string()),
        (200, invoice_json(118.0)),
    ]);

    let extraction = server.extractor().extract(DOCUMENT_TEXT).unwrap();

    assert!(extraction.validation.is_verified());
    assert!(server.requests()[1].contains("does not follow the schema"));
}

#[test]
fn a_server_error_is_reported_without_the_key() {
    let server = StubLanguageModel::answering(vec![(500, "model overloaded".to_string())]);

    let error = server.extractor().extract(DOCUMENT_TEXT).unwrap_err();

    assert!(
        matches!(&error, EngineError::UpstreamUnavailable(message) if message.contains("500") && message.contains("overloaded")),
        "{error}"
    );
    assert!(!error.to_string().contains("secret-key"));
}

#[test]
fn an_unreachable_model_is_reported_without_the_address() {
    let extractor = LanguageModelExtractor::new(LanguageModel::new(LanguageModelSettings {
        base_url: "http://127.0.0.1:1/v1".to_string(),
        model: "test-model".to_string(),
        api_key: Some("secret-key".to_string()),
    }));

    let error = extractor.extract(DOCUMENT_TEXT).unwrap_err();

    assert!(error.to_string().contains("cannot be reached"), "{error}");
    assert!(
        !error.to_string().contains("127.0.0.1") && !error.to_string().contains("secret-key"),
        "{error}"
    );
}
