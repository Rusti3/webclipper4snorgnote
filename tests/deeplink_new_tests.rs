use notebooklm_runner::deeplink::{
    DirectClipPayload, DirectClipType, encode_payload_to_deeplink, parse_new_clip_deeplink,
};

#[test]
fn parses_snorgnote_new_with_data_payload() {
    let payload = DirectClipPayload {
        clip_type: DirectClipType::FullPage,
        title: "Captured page".to_string(),
        url: "https://example.com/a".to_string(),
        content_markdown: "# Title\n\nBody".to_string(),
        created_at: "2026-02-24T00:00:00.000Z".to_string(),
        source: Some("web-clipper".to_string()),
    };
    let uri = encode_payload_to_deeplink("snorgnote", &payload).expect("encode");
    let parsed = parse_new_clip_deeplink(&uri).expect("parse");

    assert_eq!(parsed.payload, payload);
}

#[test]
fn rejects_when_data_is_missing() {
    let err = parse_new_clip_deeplink("snorgnote://new?source=web-clipper")
        .expect_err("missing data must fail");
    let msg = format!("{err:#}");
    assert!(msg.contains("data"));
}

#[test]
fn rejects_invalid_base64_data() {
    let err =
        parse_new_clip_deeplink("snorgnote://new?data=%%%").expect_err("invalid base64 must fail");
    let msg = format!("{err:#}");
    assert!(msg.contains("base64"));
}

#[test]
fn rejects_invalid_json_payload() {
    let invalid_json = "bm90LWpzb24"; // "not-json" as base64url
    let uri = format!("snorgnote://new?data={invalid_json}");
    let err = parse_new_clip_deeplink(&uri).expect_err("invalid json must fail");
    let msg = format!("{err:#}");
    assert!(msg.contains("JSON"));
}

#[test]
fn rejects_payload_with_missing_required_fields() {
    let raw = r#"{"type":"full_page","title":"x","url":"https://e.com","createdAt":"2026-02-24T00:00:00.000Z"}"#;
    let data = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        raw.as_bytes(),
    );
    let uri = format!("snorgnote://new?data={data}");
    let err = parse_new_clip_deeplink(&uri).expect_err("missing field must fail");
    let msg = format!("{err:#}");
    assert!(msg.contains("contentMarkdown"));
}

#[test]
fn rejects_wrong_target() {
    let err = parse_new_clip_deeplink("snorgnote://clip?data=abc").expect_err("wrong target");
    let msg = format!("{err:#}");
    assert!(msg.contains("Expected `new`"));
}
