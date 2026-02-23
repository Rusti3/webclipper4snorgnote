use uuid::Uuid;

use notebooklm_runner::deeplink::parse_new_clip_deeplink;

#[test]
fn parses_snorgnote_new_with_clip_id_and_source() {
    let clip_id = Uuid::new_v4();
    let uri = format!("snorgnote://new?clipId={clip_id}&source=web-clipper");
    let parsed = parse_new_clip_deeplink(&uri).expect("parse deep-link");

    assert_eq!(parsed.clip_id, clip_id);
    assert_eq!(parsed.source.as_deref(), Some("web-clipper"));
}

#[test]
fn rejects_when_clip_id_is_missing() {
    let uri = "snorgnote://new?source=web-clipper";
    let err = parse_new_clip_deeplink(uri).expect_err("must reject missing clipId");
    let msg = format!("{err:#}");
    assert!(msg.contains("clipId"));
}

#[test]
fn rejects_wrong_scheme() {
    let uri = format!("https://new?clipId={}", Uuid::new_v4());
    let err = parse_new_clip_deeplink(&uri).expect_err("must reject wrong scheme");
    let msg = format!("{err:#}");
    assert!(msg.contains("snorgnote"));
}

#[test]
fn rejects_wrong_target() {
    let uri = format!("snorgnote://clip?clipId={}", Uuid::new_v4());
    let err = parse_new_clip_deeplink(&uri).expect_err("must reject wrong target");
    let msg = format!("{err:#}");
    assert!(msg.contains("new"));
}

#[test]
fn rejects_invalid_uuid() {
    let err = parse_new_clip_deeplink("snorgnote://new?clipId=not-uuid")
        .expect_err("must reject invalid UUID");
    let msg = format!("{err:#}");
    assert!(msg.contains("UUID"));
}
