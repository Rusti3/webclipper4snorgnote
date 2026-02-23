use std::path::PathBuf;

use notebooklm_runner::app::{RunnerConfig, run_from_deeplink};
use notebooklm_runner::deeplink::{ClipPayload, encode_payload_to_deeplink};

#[test]
fn run_from_deeplink_writes_start_and_end_files() {
    let payload = ClipPayload {
        prompt: "Deep-link prompt".to_string(),
        urls: vec![
            "https://example.com/alpha".to_string(),
            "https://example.com/beta".to_string(),
        ],
        source: Some("web-clipper".to_string()),
        title: Some("Notebook from clipper".to_string()),
    };
    let uri = encode_payload_to_deeplink("snorgnote", &payload).expect("encode");

    let dir = tempfile::tempdir().expect("tempdir");
    let start_path = dir.path().join("deeplink-start.txt");
    let end_path = dir.path().join("deeplink-end.txt");

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let sidecar = root.join("tests").join("fixtures").join("mock_bridge.js");

    let result = run_from_deeplink(
        &uri,
        &RunnerConfig {
            input: start_path.clone(),
            output: end_path.clone(),
            title: "Default title".to_string(),
            sidecar_script: sidecar,
            node_path: PathBuf::from("node"),
            profile_dir: None,
            browser_path: None,
            timeout_sec: 10,
        },
    );
    assert!(
        result.is_ok(),
        "run_from_deeplink should succeed: {result:?}"
    );

    let start_text = std::fs::read_to_string(start_path).expect("start file");
    assert!(start_text.contains("PROMPT=Deep-link prompt"));
    assert!(start_text.contains("https://example.com/alpha"));

    let end_text = std::fs::read_to_string(end_path).expect("end file");
    assert!(end_text.contains("Status"));
    assert!(end_text.contains("ok"));
    assert!(end_text.contains("Notebook from clipper"));
}
