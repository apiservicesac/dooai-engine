use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use engine_core::{
    EngineError, OcrDocument, OcrEngine, OcrLine, OcrOptions, Point, TextEmbedder, TextQuad, VectorMath,
};
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

/// Embeds a text as how many "a" and "b" letters it has.
struct LetterEmbedder;

impl TextEmbedder for LetterEmbedder {
    fn model_name(&self) -> &str {
        "letters"
    }

    fn dimensions(&self) -> usize {
        2
    }

    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EngineError> {
        Ok(texts
            .iter()
            .map(|text| {
                let count = |letter| text.chars().filter(|&character| character == letter).count() as f32;
                VectorMath::normalized(&[count('a'), count('b')])
            })
            .collect())
    }
}

struct ApiScenario;

impl ApiScenario {
    fn config() -> ServerConfig {
        ServerConfig {
            address: "127.0.0.1:0".parse().unwrap(),
            models_directory: "unused".into(),
            max_upload_bytes: 1024,
        }
    }

    fn application() -> Application {
        Application::new(Self::config(), Arc::new(FakeOcrEngine))
    }

    fn application_with_embeddings() -> Application {
        Self::application().with_embedder(Arc::new(LetterEmbedder))
    }

    async fn post_json(application: Application, uri: &str, body: &str) -> (StatusCode, String) {
        let request = Request::post(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let response = application.router().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
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

#[tokio::test]
async fn health_lists_the_capabilities_that_are_installed() {
    let request = || Request::get("/health").body(Body::empty()).unwrap();

    let without = ApiScenario::application().router().oneshot(request()).await.unwrap();
    let with = ApiScenario::application_with_embeddings()
        .router()
        .oneshot(request())
        .await
        .unwrap();

    let read = |response: axum::response::Response| async move {
        String::from_utf8(response.into_body().collect().await.unwrap().to_bytes().to_vec()).unwrap()
    };
    assert!(read(without).await.contains(r#""capabilities":["ocr"]"#));
    assert!(read(with).await.contains(r#""capabilities":["ocr","embeddings"]"#));
}

#[tokio::test]
async fn embed_returns_one_vector_per_text() {
    let (status, body) = ApiScenario::post_json(
        ApiScenario::application_with_embeddings(),
        "/v1/embed",
        r#"{"texts":["aa","bb"]}"#,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains(r#""model":"letters""#) && body.contains(r#""embeddings":[[1.0,0.0],[0.0,1.0]]"#),
        "{body}"
    );
}

#[tokio::test]
async fn embed_refuses_an_empty_list_of_texts() {
    let (status, body) = ApiScenario::post_json(
        ApiScenario::application_with_embeddings(),
        "/v1/embed",
        r#"{"texts":[]}"#,
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body.contains("between 1 and"), "{body}");
}

#[tokio::test]
async fn similar_ranks_the_candidates_by_meaning() {
    let body = r#"{"query":"aa","candidates":[{"id":"far","text":"bbb"},{"id":"near","text":"aab"},{"id":"exact","text":"aaa"}],"top_k":2}"#;

    let (status, answer) =
        ApiScenario::post_json(ApiScenario::application_with_embeddings(), "/v1/similar", body).await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        answer.find("exact").unwrap() < answer.find("near").unwrap() && !answer.contains("far"),
        "{answer}"
    );
}

#[tokio::test]
async fn text_endpoints_answer_501_when_the_model_is_not_installed() {
    let (status, body) = ApiScenario::post_json(ApiScenario::application(), "/v1/embed", r#"{"texts":["a"]}"#).await;

    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    assert!(body.contains("not installed"), "{body}");
}
