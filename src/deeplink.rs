use anyhow::{Context, Result, bail};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DirectClipType {
    #[serde(rename = "full_page")]
    FullPage,
    #[serde(rename = "selection")]
    Selection,
}

impl Display for DirectClipType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FullPage => write!(f, "full_page"),
            Self::Selection => write!(f, "selection"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectClipPayload {
    #[serde(rename = "type")]
    pub clip_type: DirectClipType,
    pub title: String,
    pub url: String,
    #[serde(rename = "contentMarkdown")]
    pub content_markdown: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewClipDeepLink {
    pub payload: DirectClipPayload,
}

pub fn parse_new_clip_deeplink(uri: &str) -> Result<NewClipDeepLink> {
    let parsed = Url::parse(uri).with_context(|| format!("Invalid deep-link URI: {uri}"))?;
    if parsed.scheme() != "snorgnote" {
        bail!(
            "Unsupported URI scheme `{}`. Expected `snorgnote`",
            parsed.scheme()
        );
    }

    let target = parsed
        .host_str()
        .map(str::to_string)
        .or_else(|| {
            parsed
                .path_segments()
                .and_then(|mut segments| segments.next().map(str::to_string))
        })
        .unwrap_or_default();
    if !target.eq_ignore_ascii_case("new") {
        bail!(
            "Unsupported deep-link target `{target}`. Expected `new`, like snorgnote://new?data=..."
        );
    }

    let data_param = parsed
        .query_pairs()
        .find_map(|(key, value)| (key == "data").then(|| value.into_owned()))
        .ok_or_else(|| anyhow::anyhow!("Missing required query parameter: data"))?;

    let mut payload = parse_data_payload(&data_param)?;
    normalize_payload(&mut payload)?;
    Ok(NewClipDeepLink { payload })
}

pub fn encode_payload_to_deeplink(scheme: &str, payload: &DirectClipPayload) -> Result<String> {
    if scheme.trim().is_empty() {
        bail!("scheme must not be empty");
    }
    let mut normalized = payload.clone();
    normalize_payload(&mut normalized)?;
    let bytes = serde_json::to_vec(&normalized)?;
    let data = URL_SAFE_NO_PAD.encode(bytes);
    Ok(format!("{scheme}://new?data={data}"))
}

fn parse_data_payload(data: &str) -> Result<DirectClipPayload> {
    let bytes = URL_SAFE_NO_PAD
        .decode(data)
        .with_context(|| "Failed to decode data payload as base64url")?;
    let payload: DirectClipPayload =
        serde_json::from_slice(&bytes).with_context(|| "Failed to parse data payload JSON")?;
    Ok(payload)
}

fn normalize_payload(payload: &mut DirectClipPayload) -> Result<()> {
    payload.title = payload.title.trim().to_string();
    payload.url = payload.url.trim().to_string();
    payload.content_markdown = payload.content_markdown.trim().to_string();
    payload.created_at = payload.created_at.trim().to_string();
    payload.source = payload
        .source
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    if payload.title.is_empty() {
        bail!("Missing or empty required field: title");
    }
    if payload.url.is_empty() {
        bail!("Missing or empty required field: url");
    }
    if payload.content_markdown.is_empty() {
        bail!("Missing or empty required field: contentMarkdown");
    }
    if payload.created_at.is_empty() {
        bail!("Missing or empty required field: createdAt");
    }

    validate_url(&payload.url)?;
    validate_created_at(&payload.created_at)?;
    Ok(())
}

fn validate_url(value: &str) -> Result<()> {
    let parsed = Url::parse(value).with_context(|| format!("Invalid URL in payload: {value}"))?;
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        _ => bail!("Invalid URL scheme in payload: {value}"),
    }
}

fn validate_created_at(value: &str) -> Result<()> {
    DateTime::parse_from_rfc3339(value)
        .with_context(|| format!("Invalid createdAt. Must be RFC3339: {value}"))?;
    Ok(())
}
