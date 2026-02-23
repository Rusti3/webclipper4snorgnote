use std::path::PathBuf;

use notebooklm_runner::app::{RunnerConfig, run_once};

#[test]
fn run_once_writes_successful_end_file_with_mock_sidecar() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let input = temp_dir.path().join("start.txt");
    let output = temp_dir.path().join("end.txt");

    std::fs::write(
        &input,
        "PROMPT=Summarize this\nhttps://example.com/a\nhttps://example.com/b",
    )
    .expect("write start file");

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let sidecar = root.join("tests").join("fixtures").join("mock_bridge.js");

    let result = run_once(&RunnerConfig {
        input,
        output: output.clone(),
        title: "Mock Notebook".to_string(),
        sidecar_script: sidecar,
        node_path: PathBuf::from("node"),
        profile_dir: None,
        browser_path: None,
        timeout_sec: 10,
    });
    assert!(result.is_ok(), "run_once should succeed: {result:?}");

    let rendered = std::fs::read_to_string(output).expect("read end file");
    assert!(rendered.contains("Status"));
    assert!(rendered.contains("ok"));
    assert!(rendered.contains("Mock Notebook"));
    assert!(rendered.contains("mock answer for: Summarize this"));
}
