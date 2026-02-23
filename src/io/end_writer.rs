use std::path::Path;

use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportFailure {
    pub url: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndReport {
    pub status: String,
    pub notebook_title: Option<String>,
    pub notebook_url: Option<String>,
    pub prompt: String,
    pub answer: Option<String>,
    pub imported: usize,
    pub failed: Vec<ImportFailure>,
    pub errors: Vec<String>,
    pub duration_ms: u128,
}

pub fn render_end_report(report: &EndReport) -> String {
    let mut out = String::new();
    out.push_str("Status\n");
    out.push_str("------\n");
    out.push_str(&format!("{}\n\n", report.status));

    out.push_str("Notebook\n");
    out.push_str("--------\n");
    out.push_str(&format!(
        "Title: {}\n",
        report.notebook_title.as_deref().unwrap_or("N/A")
    ));
    out.push_str(&format!(
        "URL: {}\n\n",
        report.notebook_url.as_deref().unwrap_or("N/A")
    ));

    out.push_str("Prompt\n");
    out.push_str("------\n");
    out.push_str(&format!("{}\n\n", report.prompt));

    out.push_str("Imported\n");
    out.push_str("--------\n");
    out.push_str(&format!("Succeeded: {}\n", report.imported));
    out.push_str(&format!("Failed: {}\n", report.failed.len()));
    for item in &report.failed {
        out.push_str(&format!("- {} :: {}\n", item.url, item.reason));
    }
    out.push('\n');

    out.push_str("Answer\n");
    out.push_str("------\n");
    match &report.answer {
        Some(answer) if !answer.trim().is_empty() => {
            out.push_str(answer.trim());
            out.push_str("\n\n");
        }
        _ => out.push_str("No answer received\n\n"),
    }

    out.push_str("Errors\n");
    out.push_str("------\n");
    if report.errors.is_empty() {
        out.push_str("None\n\n");
    } else {
        for error in &report.errors {
            out.push_str(&format!("- {}\n", error));
        }
        out.push('\n');
    }

    out.push_str("Timing\n");
    out.push_str("------\n");
    out.push_str(&format!("DurationMs: {}\n", report.duration_ms));
    out
}

pub fn write_end_file(path: &Path, report: &EndReport) -> Result<()> {
    let text = render_end_report(report);
    std::fs::write(path, text)?;
    Ok(())
}
