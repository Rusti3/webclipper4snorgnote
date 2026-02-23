use std::path::Path;

use anyhow::{Context, Result, bail};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClipPayload {
    pub prompt: String,
    pub urls: Vec<String>,
    pub source: Option<String>,
    pub title: Option<String>,
}

pub fn parse_clip_uri(uri: &str) -> Result<ClipPayload> {
    let parsed = Url::parse(uri).with_context(|| format!("Invalid deep-link URI: {uri}"))?;
    if parsed.scheme() != "snorgnote" {
        bail!(
            "Unsupported URI scheme `{}`. Expected `snorgnote`",
            parsed.scheme()
        );
    }

    let target = parsed
        .host_str()
        .map(str::to_owned)
        .or_else(|| {
            parsed
                .path_segments()
                .and_then(|mut segments| segments.next().map(str::to_owned))
        })
        .unwrap_or_default();
    if !target.eq_ignore_ascii_case("clip") {
        bail!(
            "Unsupported deep-link target `{target}`. Expected `clip`, like snorgnote://clip?data=..."
        );
    }

    let mut data_param = None::<String>;
    let mut prompt_param = None::<String>;
    let mut url_params = Vec::new();
    let mut source_param = None::<String>;
    let mut title_param = None::<String>;

    for (key, value) in parsed.query_pairs() {
        match key.as_ref() {
            "data" => data_param = Some(value.into_owned()),
            "prompt" => prompt_param = Some(value.into_owned()),
            "url" => url_params.push(value.into_owned()),
            "source" => source_param = Some(value.into_owned()),
            "title" => title_param = Some(value.into_owned()),
            _ => {}
        }
    }

    let mut payload = if let Some(encoded) = data_param {
        parse_payload_data_param(&encoded)?
    } else {
        ClipPayload {
            prompt: prompt_param.unwrap_or_default(),
            urls: url_params,
            source: source_param,
            title: title_param,
        }
    };

    normalize_payload(&mut payload)?;
    Ok(payload)
}

pub fn write_start_file_from_payload(path: &Path, payload: &ClipPayload) -> Result<()> {
    let mut payload = payload.clone();
    normalize_payload(&mut payload)?;

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create parent dir: {}", parent.display()))?;
        }
    }

    let mut lines = Vec::new();
    if let Some(source) = &payload.source {
        lines.push(format!("# source={source}"));
    }
    lines.push(format!("PROMPT={}", payload.prompt));
    lines.extend(payload.urls);

    let text = format!("{}\n", lines.join("\n"));
    std::fs::write(path, text)
        .with_context(|| format!("failed to write start file: {}", path.display()))?;
    Ok(())
}

pub fn encode_payload_to_deeplink(scheme: &str, payload: &ClipPayload) -> Result<String> {
    if scheme.trim().is_empty() {
        bail!("scheme must not be empty");
    }
    let mut payload = payload.clone();
    normalize_payload(&mut payload)?;
    let json = serde_json::to_vec(&payload)?;
    let encoded = URL_SAFE_NO_PAD.encode(json);
    Ok(format!("{scheme}://clip?data={encoded}"))
}

fn parse_payload_data_param(encoded: &str) -> Result<ClipPayload> {
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .with_context(|| "Failed to decode data param as base64url")?;
    let payload: ClipPayload = serde_json::from_slice(&bytes)
        .with_context(|| "Failed to parse data param JSON payload")?;
    Ok(payload)
}

fn normalize_payload(payload: &mut ClipPayload) -> Result<()> {
    payload.prompt = normalize_prompt(&payload.prompt);
    if payload.prompt.is_empty() {
        bail!("Payload must include non-empty prompt");
    }

    payload.urls = payload
        .urls
        .iter()
        .map(|u| u.trim().to_string())
        .filter(|u| !u.is_empty())
        .collect();
    if payload.urls.is_empty() {
        bail!("Payload must include at least one URL");
    }
    for url in &payload.urls {
        validate_http_url(url)?;
    }

    payload.source = payload
        .source
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    payload.title = payload
        .title
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    Ok(())
}

fn normalize_prompt(prompt: &str) -> String {
    prompt
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn validate_http_url(value: &str) -> Result<()> {
    let parsed = Url::parse(value).with_context(|| format!("Invalid URL in payload: {value}"))?;
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        _ => bail!("Invalid URL scheme in payload: {value}"),
    }
}
