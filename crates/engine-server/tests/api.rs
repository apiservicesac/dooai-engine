use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use engine_core::{BoundingBox, EngineError, OcrDocument, OcrEngine, OcrLine};
use engine_server::{Application, ServerConfig};
use http_body_util::BodyExt;
use tower::ServiceExt;

/// Answers "Total 118.00" for the bytes `valid`, and rejects anything else as a bad image.
struct FakeOcrEngine;

impl OcrEngine for FakeOcrEngine {
    fn read(&self, encoded_image: &[u8]) -> Result<OcrDocument, EngineError> {
        if encoded_image != b"valid" {
            return Err(EngineError::InvalidImage("not an image".to_string()));
        }
        Ok(OcrDocument::from_lines(vec![OcrLine {
            text: "Total 118.00".to_string(),
            confidence: 0.99,
            bounding_box: BoundingBox::new(0.0, 0.0, 100.0, 20.0),
        }]))
    }
}

struct ApiScenario;

impl ApiScenario {
    fn application() -> Application {
        let config = ServerConfig {
            address: "127.0.0.1:0".parse().unwrap(),
            models_directory: "unused".into(),
            max_upload_bytes: 1024,
        };
        Application::new(config, Arc::new(FakeOcrEngine))
    }

    async fn post_image(body: &'static [u8]) -> (StatusCode, String) {
        let request = Request::post("/v1/ocr").body(Body::from(body)).unwrap();
        let response = Self::application().router().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }
}

#[tokio::test]
async fn health_reports_ok() {
    let request = Request::get("/health").body(Body::empty()).unwrap();

    let response = ApiScenario::application().router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn ocr_returns_the_document_as_json() {
    let (status, body) = ApiScenario::post_image(b"valid").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(r#""text":"Total 118.00""#), "{body}");
}

#[tokio::test]
async fn ocr_explains_an_invalid_image() {
    let (status, body) = ApiScenario::post_image(b"garbage").await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.contains("invalid image"), "{body}");
}
