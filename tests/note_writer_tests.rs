use notebooklm_runner::notes::writer::{NoteData, write_markdown_note};
use uuid::Uuid;

#[test]
fn writes_markdown_note_with_metadata_and_content() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let notes_dir = temp_dir.path().join("notes");
    let clip_id = Uuid::new_v4();

    let note_path = write_markdown_note(
        &notes_dir,
        &NoteData {
            clip_id,
            source: Some("web-clipper".to_string()),
            clip_type: "selection".to_string(),
            title: "Title with / invalid : chars".to_string(),
            url: "https://example.com/article".to_string(),
            content_markdown: "Selected paragraph\n\n[Source](https://example.com/article)"
                .to_string(),
            created_at: "2026-02-24T00:00:00.000Z".to_string(),
        },
    )
    .expect("write note");

    assert!(note_path.exists());
    assert_eq!(note_path.extension().and_then(|e| e.to_str()), Some("md"));

    let file_name = note_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    assert!(
        !file_name.contains('/'),
        "sanitized filename should not contain slash: {file_name}"
    );
    assert!(
        file_name.contains(&clip_id.to_string()[..8]),
        "filename should include short clip id"
    );

    let content = std::fs::read_to_string(note_path).expect("read note");
    assert!(content.contains("# Title with / invalid : chars"));
    assert!(content.contains("- Source: web-clipper"));
    assert!(content.contains("- Type: selection"));
    assert!(content.contains("- URL: https://example.com/article"));
    assert!(content.contains("Selected paragraph"));
}
