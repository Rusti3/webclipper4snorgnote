use std::path::PathBuf;
use std::process::Command;

#[test]
#[ignore = "manual live test: requires sidecar npm install + logged-in Google profile"]
fn manual_live_notebooklm_flow() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let start_file = root.join("start.txt");
    assert!(
        start_file.exists(),
        "manual live test expects start.txt in project root"
    );

    let status = Command::new("cargo")
        .args([
            "run",
            "--",
            "run",
            "--input",
            "start.txt",
            "--output",
            "end.txt",
            "--title",
            "Live Test Notebook",
        ])
        .current_dir(&root)
        .status()
        .expect("failed to run cargo live flow");

    assert!(status.success(), "live run should succeed");
}
