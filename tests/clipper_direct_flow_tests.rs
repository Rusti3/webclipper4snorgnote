use std::path::PathBuf;

use notebooklm_runner::app::{AppConfig, run_from_new_deeplink};
use notebooklm_runner::deeplink::{DirectClipPayload, DirectClipType, encode_payload_to_deeplink};

#[test]
fn deeplink_data_flow_saves_note_without_helper() {
    let payload = DirectClipPayload {
        clip_type: DirectClipType::Selection,
        title: "Captured Selection".to_string(),
        url: "https://example.com/article".to_string(),
        content_markdown: "important text".to_string(),
        created_at: "2026-02-24T00:00:00.000Z".to_string(),
        source: Some("web-clipper".to_string()),
    };
    let uri = encode_payload_to_deeplink("snorgnote", &payload).expect("encode");

    let temp_dir = tempfile::tempdir().expect("tempdir");
    let notes_dir = PathBuf::from(temp_dir.path()).join("notes");

    let result = run_from_new_deeplink(
        &uri,
        &AppConfig {
            notes_dir: notes_dir.clone(),
            timeout_sec: 10,
        },
    )
    .expect("run direct deeplink flow");

    assert!(result.note_path.exists(), "note file should be created");
    let note_content = std::fs::read_to_string(&result.note_path).expect("read note");
    assert!(note_content.contains("Captured Selection"));
    assert!(note_content.contains("important text"));
    assert!(note_content.contains("- Source: web-clipper"));
}

#[test]
fn deeplink_data_flow_clips_large_content() {
    let large = "A".repeat(200_000);
    let payload = DirectClipPayload {
        clip_type: DirectClipType::FullPage,
        title: "Very long".to_string(),
        url: "https://example.com/long".to_string(),
        content_markdown: large,
        created_at: "2026-02-24T00:00:00.000Z".to_string(),
        source: Some("web-clipper".to_string()),
    };
    let uri = encode_payload_to_deeplink("snorgnote", &payload).expect("encode");

    let temp_dir = tempfile::tempdir().expect("tempdir");
    let notes_dir = temp_dir.path().join("notes");
    let result = run_from_new_deeplink(
        &uri,
        &AppConfig {
            notes_dir,
            timeout_sec: 10,
        },
    )
    .expect("run direct flow");

    let note = std::fs::read_to_string(result.note_path).expect("read note");
    assert!(
        note.contains("[CLIPPED:"),
        "clipping marker should be present for oversized payload"
    );
}
