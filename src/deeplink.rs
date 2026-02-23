use anyhow::{Context, Result, bail};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewClipDeepLink {
    pub clip_id: Uuid,
    pub source: Option<String>,
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
            "Unsupported deep-link target `{target}`. Expected `new`, like snorgnote://new?clipId=..."
        );
    }

    let mut clip_id_raw = None::<String>;
    let mut source = None::<String>;
    for (key, value) in parsed.query_pairs() {
        match key.as_ref() {
            "clipId" => clip_id_raw = Some(value.into_owned()),
            "source" => source = Some(value.into_owned()),
            _ => {}
        }
    }

    let clip_id_raw =
        clip_id_raw.ok_or_else(|| anyhow::anyhow!("Missing required query parameter: clipId"))?;
    let clip_id = Uuid::parse_str(clip_id_raw.trim())
        .with_context(|| format!("clipId must be a valid UUID, got: {}", clip_id_raw.trim()))?;
    let source = source
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    Ok(NewClipDeepLink { clip_id, source })
}
