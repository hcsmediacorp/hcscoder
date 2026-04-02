//! OpenRouter HTTP client — streaming SSE, attribution headers, timeouts.

use std::pin::Pin;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use futures_util::stream::StreamExt;
use reqwest::{Client as ReqwestClient, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_stream::Stream;

use crate::hcscoder_openrouter::auth;

const USER_AGENT: &str = concat!(
    "hcscoder/",
    env!("CARGO_PKG_VERSION"),
    " (hcsmedia; +https://github.com/hcsmediacorp/hcscoder)"
);
/// Public source repo (OpenRouter `HTTP-Referer` / attribution).
const REFERER: &str = "https://github.com/hcsmediacorp/hcscoder";
/// OpenRouter optional attribution (`X-Title` / `X-OpenRouter-Title`).
const APP_TITLE: &str = "hcscoder by hcsmedia";

/// Extract a short user-facing message from OpenRouter JSON error bodies (falls back to truncated raw text).
fn summarize_error_response_body(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "(no response body)".to_string();
    }
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if let Some(m) = v
            .pointer("/error/message")
            .or_else(|| v.get("message"))
            .and_then(|x| x.as_str())
        {
            return m.to_string();
        }
    }
    const MAX: usize = 400;
    let count = trimmed.chars().count();
    if count > MAX {
        trimmed.chars().take(MAX).collect::<String>() + "…"
    } else {
        trimmed.to_string()
    }
}

fn friendly_http_error(status: u16, body_summary: String) -> String {
    let detail = if body_summary.is_empty() {
        "(no details)".to_string()
    } else {
        body_summary
    };
    match status {
        401 => format!(
            "OpenRouter authentication failed (401). Check OPENROUTER_API_KEY or ~/.hcscoder/openrouter_api_key. {}",
            detail
        ),
        402 => format!(
            "OpenRouter credits or billing issue (402). Add credits at https://openrouter.ai/ or use a model id with the :free suffix. {}",
            detail
        ),
        429 => format!(
            "OpenRouter rate limit (429). Wait and retry, lower request volume, or try a :free model. {}",
            detail
        ),
        502 => format!("OpenRouter bad gateway (502). Retry later. {}", detail),
        503 => format!("OpenRouter service unavailable (503). Retry later. {}", detail),
        _ => format!("OpenRouter HTTP error ({status}). {}", detail),
    }
}

/// OpenRouter API client
#[derive(Debug, Clone)]
pub struct HcscoderApiClient {
    client: ReqwestClient,
    api_key: String,
    base_url: String,
    model: String,
    timeout_secs: u64,
    /// When set, request body includes `models` + `route: "fallback"` per OpenRouter.
    fallback_models: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HcscoderUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Option<HcscoderUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

/// Streaming chunk (also parse via [`serde_json::Value`] for mid-stream `error` fields).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub model: String,
    pub choices: Vec<ChunkChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ChunkChoice {
    pub index: u32,
    pub delta: Delta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct Delta {
    pub role: Option<String>,
    pub content: Option<String>,
}

fn build_http_client(timeout_secs: u64) -> Result<ReqwestClient> {
    ReqwestClient::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(timeout_secs))
        .connect_timeout(Duration::from_secs(30))
        .pool_idle_timeout(Duration::from_secs(90))
        .build()
        .context("failed to build reqwest HTTP client")
}

impl HcscoderApiClient {
    pub fn new(model: String) -> Result<Self> {
        let api_key = auth::get_api_key()?;
        let timeout_secs = 60;
        let client = build_http_client(timeout_secs)?;
        Ok(Self {
            client,
            api_key,
            base_url: "https://openrouter.ai/api/v1".to_string(),
            model,
            timeout_secs,
            fallback_models: None,
        })
    }

    pub fn with_config(model: String, api_key: String, base_url: Option<String>) -> Result<Self> {
        let timeout_secs = 60;
        let client = build_http_client(timeout_secs)?;
        Ok(Self {
            client,
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string()),
            model,
            timeout_secs,
            fallback_models: None,
        })
    }

    /// Request timeout (seconds) used for the HTTP client.
    #[must_use]
    pub fn timeout_secs(&self) -> u64 {
        self.timeout_secs
    }

    /// Chain fallback models (OpenRouter `models` + `route: "fallback"`). Primary `model` is listed first.
    #[must_use]
    pub fn with_fallback_models(mut self, extra_models: Vec<String>) -> Self {
        self.fallback_models = Some(extra_models);
        self
    }

    fn apply_model_routing(&self, body: &mut serde_json::Value) {
        if let Some(extra) = &self.fallback_models {
            let mut models = vec![self.model.clone()];
            models.extend(extra.iter().cloned());
            body["models"] = json!(models);
            body["route"] = json!("fallback");
        }
    }

    fn openrouter_headers(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", REFERER)
            .header("X-Title", APP_TITLE)
            .header("X-OpenRouter-Title", APP_TITLE)
            // OpenRouter marketplace category (optional; see App Attribution docs)
            .header("X-OpenRouter-Categories", "cli-agent")
    }

    /// Non-streaming completion with bounded retries on transient failures.
    pub async fn create_completion(
        &self,
        messages: Vec<ChatMessage>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<ChatCompletionResponse> {
        const MAX_RETRIES: u32 = 3;
        let mut attempt = 0u32;
        loop {
            attempt += 1;
            let url = format!("{}/chat/completions", self.base_url);
            let mut body = json!({
                "model": self.model,
                "messages": messages,
            });
            if let Some(temp) = temperature {
                body["temperature"] = json!(temp);
            }
            if let Some(tokens) = max_tokens {
                body["max_tokens"] = json!(tokens);
            }
            self.apply_model_routing(&mut body);

            let response = self
                .openrouter_headers(self.client.post(&url).json(&body))
                .send()
                .await
                .context("failed to send request to OpenRouter")?;

            let status = response.status();
            let code = status.as_u16();
            // 401/402: auth/billing — do not retry. 429/502/503: transient — retry.
            if (code == 429 || code == 502 || code == 503) && attempt < MAX_RETRIES {
                let delay = Duration::from_millis(200 * u64::from(1u32 << attempt));
                tokio::time::sleep(delay).await;
                continue;
            }
            return self.handle_response(response).await;
        }
    }

    pub async fn create_stream(
        &self,
        messages: Vec<ChatMessage>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        const MAX_RETRIES: u32 = 3;
        let url = format!("{}/chat/completions", self.base_url);
        let mut attempt = 0u32;
        loop {
            attempt += 1;
            let mut body = json!({
                "model": self.model,
                "messages": messages,
                "stream": true,
            });
            if let Some(temp) = temperature {
                body["temperature"] = json!(temp);
            }
            if let Some(tokens) = max_tokens {
                body["max_tokens"] = json!(tokens);
            }
            self.apply_model_routing(&mut body);

            let response = self
                .openrouter_headers(self.client.post(&url).json(&body))
                .send()
                .await
                .context("failed to send streaming request to OpenRouter")?;

            let status = response.status();
            let code = status.as_u16();
            if (code == 429 || code == 502 || code == 503) && attempt < MAX_RETRIES {
                let delay = Duration::from_millis(200 * u64::from(1u32 << attempt));
                tokio::time::sleep(delay).await;
                continue;
            }
            return Self::handle_stream(response).await;
        }
    }

    async fn handle_response(&self, response: Response) -> Result<ChatCompletionResponse> {
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("(body read error: {})", e));
            let code = status.as_u16();
            let summary = summarize_error_response_body(&error_text);
            bail!("{}", friendly_http_error(code, summary));
        }
        response
            .json::<ChatCompletionResponse>()
            .await
            .context("failed to parse OpenRouter JSON response")
    }

    async fn handle_stream(
        response: Response,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|e| format!("(body read error: {})", e));
            let code = status.as_u16();
            let summary = summarize_error_response_body(&error_text);
            bail!("{}", friendly_http_error(code, summary));
        }

        let mut byte_stream = response.bytes_stream();
        let out = async_stream::stream! {
            let mut line_buf = String::new();
            while let Some(chunk) = byte_stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        line_buf.push_str(&String::from_utf8_lossy(&bytes));
                        loop {
                            match line_buf.find('\n') {
                                None => break,
                                Some(pos) => {
                                    let line = line_buf[..pos].trim_end_matches('\r').to_string();
                                    line_buf.drain(..pos + 1);
                                    if line.is_empty() {
                                        continue;
                                    }
                                    // SSE comment / keep-alive lines (RFC 8895)
                                    if line.starts_with(':') {
                                        continue;
                                    }
                                    if let Some(data) = line.strip_prefix("data: ") {
                                        let data = data.trim();
                                        if data.is_empty() {
                                            continue;
                                        }
                                        if data == "[DONE]" {
                                            return;
                                        }
                                        // Per OpenRouter docs: ignore occasional non-JSON noise; handle mid-stream errors.
                                        let v: serde_json::Value = match serde_json::from_str(data) {
                                            Ok(v) => v,
                                            Err(_) => continue,
                                        };
                                        if let Some(err) = v.get("error") {
                                            let msg = err
                                                .get("message")
                                                .and_then(|m| m.as_str())
                                                .unwrap_or("provider error");
                                            yield Err(anyhow::anyhow!("OpenRouter stream error: {}", msg));
                                            return;
                                        }
                                        if let Some(choices) = v.get("choices").and_then(|c| c.as_array()) {
                                            if let Some(fr) = choices
                                                .first()
                                                .and_then(|c| c.get("finish_reason"))
                                                .and_then(|f| f.as_str())
                                            {
                                                if fr == "error" {
                                                    yield Err(anyhow::anyhow!(
                                                        "OpenRouter stream terminated with finish_reason=error"
                                                    ));
                                                    return;
                                                }
                                            }
                                        }
                                        if let Some(content) = v
                                            .get("choices")
                                            .and_then(|c| c.as_array())
                                            .and_then(|a| a.first())
                                            .and_then(|ch| ch.get("delta"))
                                            .and_then(|d| d.get("content"))
                                            .and_then(|c| c.as_str())
                                        {
                                            if !content.is_empty() {
                                                yield Ok(content.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => yield Err(anyhow::anyhow!("stream read: {}", e)),
                }
            }
        };

        Ok(Box::pin(out))
    }

    pub fn model(&self) -> &str {
        &self.model
    }
}

#[cfg(test)]
mod post_release_audit {
    // #region agent log
    use super::{friendly_http_error, summarize_error_response_body, APP_TITLE, REFERER};
    use std::io::Write;

    fn append_agent_ndjson(payload: &str) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("debug-e5cd16.log");
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(f, "{}", payload);
        }
    }
    // #endregion

    #[test]
    fn audit_openrouter_headers_match_spec() {
        assert_eq!(APP_TITLE, "hcscoder by hcsmedia");
        assert_eq!(
            REFERER,
            "https://github.com/hcsmediacorp/hcscoder",
            "canonical public repo URL for attribution"
        );
        // #region agent log
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let line = format!(
            r#"{{"sessionId":"e5cd16","runId":"audit","hypothesisId":"H1","location":"client.rs:post_release_audit","message":"title and referer constants","data":{{"APP_TITLE":"{}","REFERER":"{}"}},"timestamp":{}}}"#,
            APP_TITLE, REFERER, ts
        );
        append_agent_ndjson(&line);
        // #endregion
    }

    #[test]
    fn error_body_json_becomes_short_message() {
        let raw = r#"{"error":{"message":"Insufficient credits"}}"#;
        assert_eq!(
            summarize_error_response_body(raw),
            "Insufficient credits"
        );
        let friendly = friendly_http_error(402, summarize_error_response_body(raw));
        assert!(
            friendly.contains("402"),
            "{}",
            friendly
        );
        assert!(
            friendly.contains("Insufficient credits"),
            "{}",
            friendly
        );
        assert!(
            !friendly.contains("\"error\""),
            "should not echo raw JSON: {}",
            friendly
        );
    }
}
