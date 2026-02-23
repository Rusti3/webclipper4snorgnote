mod support;

use std::path::PathBuf;

use uuid::Uuid;

use notebooklm_runner::app::{AppConfig, run_from_new_deeplink};
use support::mock_http::{ExpectedRequest, MockHttpServer};

#[test]
fn deeplink_flow_fetches_clip_saves_note_and_deletes_clip() {
    let clip_id = Uuid::new_v4();
    let deep_link = format!("snorgnote://new?clipId={clip_id}&source=web-clipper");

    let get_body = format!(
        r##"{{"clipId":"{clip_id}","payload":{{"type":"full_page","title":"Captured Title","url":"https://example.com/page","contentMarkdown":"# Captured\n\nbody","createdAt":"2026-02-24T00:00:00.000Z"}},"expiresAt":"2026-02-24T01:00:00.000Z"}}"##
    );
    let server = MockHttpServer::start(vec![
        ExpectedRequest::json("GET", format!("/clips/{clip_id}"), 200, get_body),
        ExpectedRequest::empty("DELETE", format!("/clips/{clip_id}"), 204),
    ])
    .expect("start mock server");

    let temp_dir = tempfile::tempdir().expect("tempdir");
    let notes_dir = PathBuf::from(temp_dir.path()).join("notes");
    let result = run_from_new_deeplink(
        &deep_link,
        &AppConfig {
            notes_dir: notes_dir.clone(),
            helper_base_url: server.base_url().to_string(),
            timeout_sec: 10,
        },
    )
    .expect("run deeplink flow");

    assert!(result.note_path.exists(), "note file should be created");
    let note_content = std::fs::read_to_string(&result.note_path).expect("read note");
    assert!(note_content.contains("Captured Title"));
    assert!(note_content.contains("# Captured"));

    let received = server.finish().expect("finish server");
    assert_eq!(received.len(), 2);
    assert_eq!(received[0].method, "GET");
    assert_eq!(received[0].path, format!("/clips/{clip_id}"));
    assert_eq!(received[1].method, "DELETE");
    assert_eq!(received[1].path, format!("/clips/{clip_id}"));
}
