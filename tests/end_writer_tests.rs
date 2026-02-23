use notebooklm_runner::io::end_writer::{EndReport, ImportFailure, render_end_report};

#[test]
fn render_end_report_contains_all_sections() {
    let report = EndReport {
        status: "ok".to_string(),
        notebook_title: Some("Auto Notebook".to_string()),
        notebook_url: Some("https://notebooklm.google.com/notebook/123".to_string()),
        prompt: "Summarize".to_string(),
        answer: Some("Here is a concise summary.".to_string()),
        imported: 2,
        failed: vec![ImportFailure {
            url: "https://bad.example".to_string(),
            reason: "timeout".to_string(),
        }],
        errors: vec!["minor warning".to_string()],
        duration_ms: 3100,
    };

    let rendered = render_end_report(&report);
    assert!(rendered.contains("Status"));
    assert!(rendered.contains("Notebook"));
    assert!(rendered.contains("Imported"));
    assert!(rendered.contains("Answer"));
    assert!(rendered.contains("Errors"));
    assert!(rendered.contains("Timing"));
    assert!(rendered.contains("Auto Notebook"));
    assert!(rendered.contains("Here is a concise summary."));
    assert!(rendered.contains("https://bad.example"));
}

#[test]
fn render_end_report_handles_empty_optional_fields() {
    let report = EndReport {
        status: "failed".to_string(),
        notebook_title: None,
        notebook_url: None,
        prompt: "Prompt".to_string(),
        answer: None,
        imported: 0,
        failed: vec![],
        errors: vec!["fatal: login timeout".to_string()],
        duration_ms: 7000,
    };

    let rendered = render_end_report(&report);
    assert!(rendered.contains("failed"));
    assert!(rendered.contains("login timeout"));
    assert!(rendered.contains("No answer received"));
}
