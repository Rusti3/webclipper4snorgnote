use notebooklm_runner::io::start_parser::{parse_start_content, parse_start_file};

#[test]
fn parses_prompt_and_urls_ignoring_comments_and_empty_lines() {
    let content = r#"
# comment
PROMPT=Summarize key points

https://example.com/a
https://youtu.be/abc123
"#;

    let parsed = parse_start_content(content).expect("parser should parse valid input");
    assert_eq!(parsed.prompt, "Summarize key points");
    assert_eq!(
        parsed.urls,
        vec![
            "https://example.com/a".to_string(),
            "https://youtu.be/abc123".to_string()
        ]
    );
}

#[test]
fn fails_if_prompt_is_missing() {
    let content = r#"
https://example.com/a
https://example.com/b
"#;

    let err = parse_start_content(content).expect_err("missing prompt must fail");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("PROMPT="),
        "expected error to mention PROMPT=, got: {msg}"
    );
}

#[test]
fn fails_if_prompt_is_not_the_first_meaningful_line() {
    let content = r#"
https://example.com/a
PROMPT=hello
"#;

    let err = parse_start_content(content).expect_err("prompt must be first meaningful line");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("first meaningful line"),
        "expected specific ordering error, got: {msg}"
    );
}

#[test]
fn fails_if_no_urls_after_prompt() {
    let content = r#"
PROMPT=Only prompt, no links
"#;

    let err = parse_start_content(content).expect_err("empty url list must fail");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("at least one URL"),
        "expected URL count message, got: {msg}"
    );
}

#[test]
fn fails_if_url_is_invalid() {
    let content = r#"
PROMPT=Test
notaurl
"#;

    let err = parse_start_content(content).expect_err("invalid URL must fail");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("Invalid URL"),
        "expected URL validation error, got: {msg}"
    );
}

#[test]
fn parse_start_file_reads_from_disk() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("start.txt");
    std::fs::write(
        &path,
        "PROMPT=Disk parse\nhttps://example.com/doc\nhttps://example.org",
    )
    .expect("write test file");

    let parsed = parse_start_file(&path).expect("parse_start_file");
    assert_eq!(parsed.prompt, "Disk parse");
    assert_eq!(parsed.urls.len(), 2);
}

#[test]
fn parses_when_file_starts_with_utf8_bom() {
    let content = "\u{feff}PROMPT=With BOM\nhttps://example.com/a";
    let parsed = parse_start_content(content).expect("parser should handle UTF-8 BOM");
    assert_eq!(parsed.prompt, "With BOM");
    assert_eq!(parsed.urls, vec!["https://example.com/a"]);
}
