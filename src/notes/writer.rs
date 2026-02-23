use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Local;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteData {
    pub clip_id: Uuid,
    pub source: Option<String>,
    pub clip_type: String,
    pub title: String,
    pub url: String,
    pub content_markdown: String,
    pub created_at: String,
}

pub fn write_markdown_note(notes_dir: &Path, data: &NoteData) -> Result<PathBuf> {
    std::fs::create_dir_all(notes_dir)
        .with_context(|| format!("failed to create notes dir: {}", notes_dir.display()))?;

    let imported_at = Local::now().to_rfc3339();
    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let slug = slugify_title(&data.title);
    let short_clip = &data.clip_id.to_string()[..8];
    let file_name = format!("{timestamp}-{slug}-{short_clip}.md");
    let path = notes_dir.join(file_name);

    let source = data.source.as_deref().unwrap_or("unknown");
    let mut body = String::new();
    body.push_str(&format!("# {}\n\n", data.title.trim()));
    body.push_str(&format!("- Source: {source}\n"));
    body.push_str(&format!("- Type: {}\n", data.clip_type.trim()));
    body.push_str(&format!("- URL: {}\n", data.url.trim()));
    body.push_str(&format!("- CreatedAt: {}\n", data.created_at.trim()));
    body.push_str(&format!("- ImportedAt: {imported_at}\n\n"));
    body.push_str(data.content_markdown.trim());
    body.push('\n');

    std::fs::write(&path, body)
        .with_context(|| format!("failed to write note file: {}", path.display()))?;
    Ok(path)
}

fn slugify_title(title: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;

    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
            continue;
        }
        if ch.is_whitespace() || ch == '-' || ch == '_' {
            if !last_dash && !out.is_empty() {
                out.push('-');
                last_dash = true;
            }
            continue;
        }
    }

    let slug = out.trim_matches('-');
    if slug.is_empty() {
        "note".to_string()
    } else {
        slug.to_string()
    }
}
