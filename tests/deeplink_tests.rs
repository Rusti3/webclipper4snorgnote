use std::path::PathBuf;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use notebooklm_runner::deeplink::{
    ClipPayload, encode_payload_to_deeplink, parse_clip_uri, write_start_file_from_payload,
};

#[test]
fn parses_base64_data_uri_payload() {
    let payload = ClipPayload {
        prompt: "Summarize in 5 bullets".to_string(),
        urls: vec![
            "https://example.com/a".to_string(),
            "https://example.com/b".to_string(),
        ],
        source: Some("web-clipper".to_string()),
        title: Some("Deep Link Notebook".to_string()),
    };

    let uri = encode_payload_to_deeplink("snorgnote", &payload).expect("encode deeplink");
    let parsed = parse_clip_uri(&uri).expect("parse deeplink");
    assert_eq!(parsed, payload);
}

#[test]
fn parses_query_fallback_without_data_param() {
    let uri = "snorgnote://clip?prompt=Hello%20World&url=https%3A%2F%2Fexample.com%2Fa&url=https%3A%2F%2Fexample.com%2Fb";
    let parsed = parse_clip_uri(uri).expect("parse query fallback");
    assert_eq!(parsed.prompt, "Hello World");
    assert_eq!(
        parsed.urls,
        vec![
            "https://example.com/a".to_string(),
            "https://example.com/b".to_string()
        ]
    );
    assert_eq!(parsed.source, None);
}

#[test]
fn rejects_wrong_scheme() {
    let err = parse_clip_uri("wrong://clip?prompt=hi&url=https://example.com")
        .expect_err("wrong scheme must fail");
    let msg = format!("{err:#}");
    assert!(msg.contains("snorgnote"));
}

#[test]
fn rejects_payload_with_empty_urls() {
    let raw_payload = r#"{"prompt":"hello","urls":[]}"#;
    let data = URL_SAFE_NO_PAD.encode(raw_payload.as_bytes());
    let uri = format!("snorgnote://clip?data={data}");
    let err = parse_clip_uri(&uri).expect_err("empty url list must fail");
    let msg = format!("{err:#}");
    assert!(msg.contains("at least one URL"));
}

#[test]
fn writes_start_txt_from_payload() {
    let payload = ClipPayload {
        prompt: "My Prompt".to_string(),
        urls: vec!["https://example.com/a".to_string()],
        source: Some("web-clipper".to_string()),
        title: None,
    };
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("start.txt");
    write_start_file_from_payload(&path, &payload).expect("write start from payload");
    let text = std::fs::read_to_string(path).expect("read start");
    assert!(text.contains("PROMPT=My Prompt"));
    assert!(text.contains("https://example.com/a"));
}

#[test]
fn deeplink_encoder_is_url_safe() {
    let payload = ClipPayload {
        prompt: "Привет + symbols / ? &".to_string(),
        urls: vec!["https://example.com/a?x=1&y=2".to_string()],
        source: None,
        title: None,
    };
    let uri = encode_payload_to_deeplink("snorgnote", &payload).expect("encode");
    assert!(uri.starts_with("snorgnote://clip?data="));
    assert!(
        !uri.contains('+'),
        "base64url payload in query should not include plus sign"
    );
}

#[test]
fn deeplink_start_file_path_can_be_customized() {
    let payload = ClipPayload {
        prompt: "Prompt".to_string(),
        urls: vec!["https://example.com/1".to_string()],
        source: None,
        title: None,
    };
    let dir = tempfile::tempdir().expect("tempdir");
    let custom = PathBuf::from(dir.path()).join("incoming").join("clip.txt");
    write_start_file_from_payload(&custom, &payload).expect("write custom");
    assert!(custom.exists());
}
