use std::time::Duration;

use engine_core::EngineError;
use serde_json::{Value, json};

use crate::settings::ExtractionSettings;

/// Where a language model is and how to reach it: any server that speaks the OpenAI chat completions API
/// (OpenAI, vLLM, llama.cpp, Ollama, LM Studio...).
#[derive(Debug, Clone)]
pub struct LanguageModelSettings {
    /// Up to the API version, e.g. `http://localhost:8000/v1`.
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
}

pub struct LanguageModel {
    settings: LanguageModelSettings,
    agent: ureq::Agent,
}

impl LanguageModel {
    pub fn new(settings: LanguageModelSettings) -> Self {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(ExtractionSettings::TIMEOUT_SECONDS)))
            .http_status_as_error(false)
            .build()
            .into();
        Self { settings, agent }
    }

    pub fn name(&self) -> &str {
        &self.settings.model
    }

    /// The model's answer to `user`, constrained to the JSON `schema`.
    pub fn complete_json(&self, system: &str, user: &str, schema: &Value) -> Result<String, EngineError> {
        let body = json!({
            "model": self.settings.model,
            "temperature": ExtractionSettings::TEMPERATURE,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user},
            ],
            "response_format": {
                "type": "json_schema",
                "json_schema": {"name": ExtractionSettings::SCHEMA_NAME, "schema": schema},
            },
        });
        let url = format!("{}/chat/completions", self.settings.base_url.trim_end_matches('/'));
        let mut request = self.agent.post(&url);
        if let Some(key) = &self.settings.api_key {
            request = request.header("Authorization", format!("Bearer {key}"));
        }
        // The error of the transport may carry the address or a header: only a summary is reported.
        let mut response = request.send_json(&body).map_err(|error| {
            EngineError::Inference(format!(
                "the language model cannot be reached: {}",
                Self::summary(&error)
            ))
        })?;
        let status = response.status();
        let answer: Value = response.body_mut().read_json().map_err(|_| {
            EngineError::UpstreamUnavailable(format!("the language model answered {status} without JSON"))
        })?;
        if !status.is_success() {
            return Err(EngineError::UpstreamUnavailable(format!(
                "the language model answered {status}: {}",
                answer["error"]["message"].as_str().unwrap_or("no detail")
            )));
        }
        answer["choices"][0]["message"]["content"]
            .as_str()
            .map(str::to_string)
            .ok_or_else(|| EngineError::UpstreamUnavailable("the language model's answer has no content".to_string()))
    }

    fn summary(error: &ureq::Error) -> &'static str {
        match error {
            ureq::Error::Timeout(_) => "timeout",
            ureq::Error::Io(_) | ureq::Error::ConnectionFailed | ureq::Error::HostNotFound => "connection failed",
            _ => "request failed",
        }
    }
}
