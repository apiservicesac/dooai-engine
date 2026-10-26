use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use engine_core::{EngineError, OcrDocument, OcrEngine, OcrLine, OcrOptions, Point, TextQuad};
use engine_server::{Application, ServerConfig};
use http_body_util::BodyExt;
use tower::ServiceExt;

/// Answers "Total 118.00" (tagged with the requested language) for the bytes `valid`,
/// rejects anything else as a bad image and the language `xx` as unsupported.
struct FakeOcrEngine;

impl OcrEngine for FakeOcrEngine {
    fn read(&self, encoded_image: &[u8], options: &OcrOptions) -> Result<OcrDocument, EngineError> {
        if encoded_image != b"valid" {
            return Err(EngineError::InvalidImage("not an image".to_string()));
        }
        let language = options.language.as_deref().unwrap_or("auto");
        if language == "xx" {
            return Err(EngineError::UnsupportedLanguage(
                "'xx' is not a known language".to_string(),
            ));
        }
        let quad = TextQuad::from_unordered([
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 20.0),
            Point::new(0.0, 20.0),
        ]);
        Ok(OcrDocument::from_lines(
            vec![OcrLine::new("Total 118.00".to_string(), 0.99, quad)],
            format!("fake/{language}"),
            0,
        ))
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

    async fn post_image(uri: &str, body: &'static [u8]) -> (StatusCode, String) {
        let request = Request::post(uri).body(Body::from(body)).unwrap();
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
    let (status, body) = ApiScenario::post_image("/v1/ocr", b"valid").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(r#""text":"Total 118.00""#), "{body}");
}

#[tokio::test]
async fn ocr_explains_an_invalid_image() {
    let (status, body) = ApiScenario::post_image("/v1/ocr", b"garbage").await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.contains("invalid image"), "{body}");
}

#[tokio::test]
async fn ocr_passes_the_requested_language_to_the_engine() {
    let (status, body) = ApiScenario::post_image("/v1/ocr?language=ru", b"valid").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(r#""engine":"fake/ru""#), "{body}");
}

#[tokio::test]
async fn ocr_rejects_an_unsupported_language_as_a_bad_request() {
    let (status, body) = ApiScenario::post_image("/v1/ocr?language=xx", b"valid").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.contains("not a known language"), "{body}");
}
