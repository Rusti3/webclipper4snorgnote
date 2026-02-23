use std::fmt::{Display, Formatter};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum ClipType {
    #[serde(rename = "full_page")]
    FullPage,
    #[serde(rename = "selection")]
    Selection,
}

impl Display for ClipType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FullPage => write!(f, "full_page"),
            Self::Selection => write!(f, "selection"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct HelperClipPayload {
    #[serde(rename = "type")]
    pub clip_type: ClipType,
    pub title: String,
    pub url: String,
    #[serde(rename = "contentMarkdown")]
    pub content_markdown: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelperClip {
    pub clip_id: Uuid,
    pub payload: HelperClipPayload,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct HelperHealth {
    pub ok: bool,
    #[serde(rename = "clipsInMemory")]
    pub clips_in_memory: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct HelperClient {
    base_url: String,
    agent: ureq::Agent,
}

#[derive(Debug, Deserialize)]
struct RawHelperClipResponse {
    #[serde(rename = "clipId")]
    clip_id: String,
    payload: HelperClipPayload,
    #[serde(rename = "expiresAt")]
    expires_at: Option<String>,
}

impl HelperClient {
    pub fn new(base_url: String, timeout: Duration) -> Result<Self> {
        let sanitized = sanitize_base_url(&base_url)?;
        let agent = ureq::AgentBuilder::new().timeout(timeout).build();
        Ok(Self {
            base_url: sanitized,
            agent,
        })
    }

    pub fn fetch_clip(&self, clip_id: Uuid) -> Result<HelperClip> {
        let url = format!("{}/clips/{}", self.base_url, clip_id);
        let response = self
            .agent
            .get(&url)
            .call()
            .map_err(|err| map_http_error(err, "fetch clip"))?;
        let raw: RawHelperClipResponse = response
            .into_json()
            .context("helper returned invalid JSON for clip payload")?;
        let parsed_clip_id = Uuid::parse_str(raw.clip_id.trim())
            .with_context(|| format!("helper returned invalid clipId: {}", raw.clip_id.trim()))?;
        Ok(HelperClip {
            clip_id: parsed_clip_id,
            payload: raw.payload,
            expires_at: raw.expires_at,
        })
    }

    pub fn delete_clip(&self, clip_id: Uuid) -> Result<()> {
        let url = format!("{}/clips/{}", self.base_url, clip_id);
        self.agent
            .delete(&url)
            .call()
            .map_err(|err| map_http_error(err, "delete clip"))?;
        Ok(())
    }

    pub fn health(&self) -> Result<HelperHealth> {
        let url = format!("{}/health", self.base_url);
        let response = self
            .agent
            .get(&url)
            .call()
            .map_err(|err| map_http_error(err, "helper health check"))?;
        let body: HelperHealth = response
            .into_json()
            .context("helper returned invalid JSON for health")?;
        Ok(body)
    }
}

fn sanitize_base_url(base_url: &str) -> Result<String> {
    let trimmed = base_url.trim();
    if trimmed.is_empty() {
        bail!("helper base URL must not be empty");
    }
    let parsed =
        url::Url::parse(trimmed).with_context(|| format!("invalid helper base URL: {trimmed}"))?;
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        bail!("helper base URL must be http or https: {trimmed}");
    }
    Ok(trimmed.trim_end_matches('/').to_string())
}

fn map_http_error(err: ureq::Error, action: &str) -> anyhow::Error {
    match err {
        ureq::Error::Status(status, response) => {
            let body = response.into_string().unwrap_or_default();
            let message = extract_error_message(&body);
            anyhow::anyhow!("{action} failed with HTTP {status}: {message}")
        }
        ureq::Error::Transport(transport) => {
            anyhow::anyhow!("{action} failed: {}", transport)
        }
    }
}

fn extract_error_message(body: &str) -> String {
    if body.trim().is_empty() {
        return "empty response body".to_string();
    }
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(body)
        && let Some(error) = parsed.get("error").and_then(|v| v.as_str())
    {
        return error.to_string();
    }
    body.trim().to_string()
}
