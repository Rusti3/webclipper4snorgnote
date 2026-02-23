mod support;

use std::time::Duration;

use uuid::Uuid;

use notebooklm_runner::helper_client::{ClipType, HelperClient};
use support::mock_http::{ExpectedRequest, MockHttpServer};

#[test]
fn fetch_clip_successfully() {
    let clip_id = Uuid::new_v4();
    let response_body = format!(
        r##"{{"clipId":"{clip_id}","payload":{{"type":"full_page","title":"Example","url":"https://example.com","contentMarkdown":"# Title","createdAt":"2026-02-24T00:00:00.000Z"}},"expiresAt":"2026-02-24T01:00:00.000Z"}}"##
    );
    let server = MockHttpServer::start(vec![ExpectedRequest::json(
        "GET",
        format!("/clips/{clip_id}"),
        200,
        response_body,
    )])
    .expect("start mock server");

    let client =
        HelperClient::new(server.base_url().to_string(), Duration::from_secs(2)).expect("client");
    let clip = client.fetch_clip(clip_id).expect("fetch clip");
    assert_eq!(clip.clip_id, clip_id);
    assert_eq!(clip.payload.clip_type, ClipType::FullPage);
    assert_eq!(clip.payload.title, "Example");
    assert_eq!(clip.payload.url, "https://example.com");

    let received = server.finish().expect("finish server");
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].method, "GET");
}

#[test]
fn fetch_clip_returns_not_found_error() {
    let clip_id = Uuid::new_v4();
    let server = MockHttpServer::start(vec![ExpectedRequest::json(
        "GET",
        format!("/clips/{clip_id}"),
        404,
        r#"{"error":"Clip not found or expired."}"#,
    )])
    .expect("start mock server");

    let client =
        HelperClient::new(server.base_url().to_string(), Duration::from_secs(2)).expect("client");
    let err = client.fetch_clip(clip_id).expect_err("expected 404 error");
    let msg = format!("{err:#}");
    assert!(msg.contains("not found") || msg.contains("expired"));

    let _ = server.finish();
}

#[test]
fn delete_clip_sends_delete_request() {
    let clip_id = Uuid::new_v4();
    let server = MockHttpServer::start(vec![ExpectedRequest::empty(
        "DELETE",
        format!("/clips/{clip_id}"),
        204,
    )])
    .expect("start mock server");

    let client =
        HelperClient::new(server.base_url().to_string(), Duration::from_secs(2)).expect("client");
    client.delete_clip(clip_id).expect("delete clip");

    let received = server.finish().expect("finish server");
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].method, "DELETE");
    assert_eq!(received[0].path, format!("/clips/{clip_id}"));
}
