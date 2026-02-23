use std::path::PathBuf;

use notebooklm_runner::bridge::process::{BridgeClient, BridgeProcessConfig};
use notebooklm_runner::logging::AppLogger;
use serde_json::json;

#[test]
fn bridge_client_roundtrip_with_mock_sidecar() {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mock_script = project_root
        .join("tests")
        .join("fixtures")
        .join("mock_bridge.js");
    assert!(
        mock_script.exists(),
        "missing test fixture: {}",
        mock_script.display()
    );

    let mut client = BridgeClient::spawn(
        BridgeProcessConfig {
            node_path: PathBuf::from("node"),
            sidecar_script: mock_script,
            profile_dir: None,
            browser_path: None,
            timeout_sec: 10,
        },
        AppLogger::new_for_tests(),
    )
    .expect("spawn mock bridge");

    let connect = client
        .send_command("connect", json!({}))
        .expect("connect response");
    assert_eq!(connect["status"], "connected");

    let import = client
        .send_command(
            "import_urls",
            json!({"urls": ["https://example.com/a", "https://example.com/b"]}),
        )
        .expect("import response");

    assert_eq!(import["imported"], 2);
    assert_eq!(import["failed"].as_array().expect("failed arr").len(), 0);

    let answer = client
        .send_command("ask", json!({"prompt": "test prompt"}))
        .expect("ask response");
    assert!(
        answer["answer"]
            .as_str()
            .expect("answer string")
            .contains("test prompt")
    );

    client.close().expect("close");
}
