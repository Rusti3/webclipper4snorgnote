use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use uuid::Uuid;

use crate::deeplink::parse_new_clip_deeplink;
use crate::helper_client::{HelperClient, HelperHealth};
use crate::logging::AppLogger;
use crate::notes::writer::{NoteData, write_markdown_note};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub notes_dir: PathBuf,
    pub helper_base_url: String,
    pub timeout_sec: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipRunResult {
    pub clip_id: Uuid,
    pub note_path: PathBuf,
    pub delete_error: Option<String>,
}

pub fn run_from_new_deeplink(uri: &str, config: &AppConfig) -> Result<ClipRunResult> {
    let logger = AppLogger::new()?;
    logger.info("clipper run started");

    let deep_link = parse_new_clip_deeplink(uri)?;
    logger.info(&format!("deep-link parsed: clipId={}", deep_link.clip_id));

    let helper = HelperClient::new(
        config.helper_base_url.clone(),
        Duration::from_secs(config.timeout_sec.max(1)),
    )?;
    let clip = helper.fetch_clip(deep_link.clip_id)?;
    logger.info("clip payload fetched from helper");

    let note_path = write_markdown_note(
        &config.notes_dir,
        &NoteData {
            clip_id: clip.clip_id,
            source: deep_link.source,
            clip_type: clip.payload.clip_type.to_string(),
            title: clip.payload.title,
            url: clip.payload.url,
            content_markdown: clip.payload.content_markdown,
            created_at: clip.payload.created_at,
        },
    )?;
    logger.info(&format!("note saved: {}", note_path.display()));

    let delete_error = match helper.delete_clip(clip.clip_id) {
        Ok(()) => {
            logger.info("clip deleted from helper");
            None
        }
        Err(err) => {
            let message = format!("{err:#}");
            logger.warn(&format!("failed to delete clip from helper: {message}"));
            Some(message)
        }
    };

    Ok(ClipRunResult {
        clip_id: clip.clip_id,
        note_path,
        delete_error,
    })
}

pub fn check_helper_health(config: &AppConfig) -> Result<HelperHealth> {
    let helper = HelperClient::new(
        config.helper_base_url.clone(),
        Duration::from_secs(config.timeout_sec.max(1)),
    )?;
    helper.health()
}
