use std::path::Path;

use anyhow::{Context, Result, bail};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartInput {
    pub prompt: String,
    pub urls: Vec<String>,
}

pub fn parse_start_file(path: &Path) -> Result<StartInput> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read input file: {}", path.display()))?;
    parse_start_content(&content)
}

pub fn parse_start_content(content: &str) -> Result<StartInput> {
    let lines: Vec<(usize, &str)> = content
        .lines()
        .enumerate()
        .map(|(idx, line)| (idx + 1, line.trim().trim_start_matches('\u{feff}')))
        .filter(|(_, line)| !line.is_empty() && !line.starts_with('#'))
        .collect();

    let Some((_, first_line)) = lines.first() else {
        bail!("Missing PROMPT= line in start.txt");
    };

    if !first_line.starts_with("PROMPT=") {
        bail!("PROMPT= must be the first meaningful line in start.txt");
    }

    let prompt = first_line["PROMPT=".len()..].trim().to_string();
    if prompt.is_empty() {
        bail!("PROMPT= value must not be empty");
    }

    let mut urls = Vec::new();
    for (line_number, line) in lines.iter().skip(1).copied() {
        if line.starts_with("PROMPT=") {
            bail!("PROMPT= can only appear once at line 1, duplicate at line {line_number}");
        }

        validate_url(line, line_number)?;
        urls.push(line.to_string());
    }

    if urls.is_empty() {
        bail!("start.txt must contain at least one URL after PROMPT=");
    }

    Ok(StartInput { prompt, urls })
}

fn validate_url(value: &str, line_number: usize) -> Result<()> {
    let parsed =
        Url::parse(value).with_context(|| format!("Invalid URL at line {line_number}: {value}"))?;
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        _ => bail!("Invalid URL scheme at line {line_number}: {value}"),
    }
}
