use std::path::PathBuf;

use anyhow::Result;
use uuid::Uuid;

use crate::deeplink::parse_new_clip_deeplink;
use crate::logging::AppLogger;
use crate::notes::writer::{NoteData, write_markdown_note};

pub const MAX_CONTENT_MARKDOWN_CHARS: usize = 120_000;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub notes_dir: PathBuf,
    pub timeout_sec: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipRunResult {
    pub clip_id: Uuid,
    pub note_path: PathBuf,
    pub clipped: bool,
}

pub fn run_from_new_deeplink(uri: &str, config: &AppConfig) -> Result<ClipRunResult> {
    let logger = AppLogger::new()?;
    logger.info("clipper direct run started");
    logger.info(&format!("configured timeout_sec={}", config.timeout_sec));

    let deep_link = parse_new_clip_deeplink(uri)?;
    logger.info("deep-link payload parsed");

    let clip_id = Uuid::new_v4();
    let (content_markdown, clipped) = clip_content_markdown(
        &deep_link.payload.content_markdown,
        MAX_CONTENT_MARKDOWN_CHARS,
    );
    if clipped {
        logger.warn("content was clipped due to size limit");
    }

    let note_path = write_markdown_note(
        &config.notes_dir,
        &NoteData {
            clip_id,
            source: deep_link.payload.source,
            clip_type: deep_link.payload.clip_type.to_string(),
            title: deep_link.payload.title,
            url: deep_link.payload.url,
            content_markdown,
            created_at: deep_link.payload.created_at,
        },
    )?;
    logger.info(&format!("note saved: {}", note_path.display()));

    Ok(ClipRunResult {
        clip_id,
        note_path,
        clipped,
    })
}

fn clip_content_markdown(content: &str, max_chars: usize) -> (String, bool) {
    if content.chars().count() <= max_chars {
        return (content.to_string(), false);
    }
    let clipped: String = content.chars().take(max_chars).collect();
    let marker = format!(
        "\n\n[CLIPPED: original_length={} chars, limit={} chars]",
        content.chars().count(),
        max_chars
    );
    (format!("{clipped}{marker}"), true)
}
