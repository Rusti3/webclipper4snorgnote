use std::path::Path;

use notebooklm_runner::protocol::protocol_command_value;

#[test]
fn protocol_command_value_wraps_exe_and_placeholder() {
    let exe = Path::new(r"D:\Snorgnote App\notebooklm_runner.exe");
    let command = protocol_command_value(exe);
    assert_eq!(
        command,
        r#""D:\Snorgnote App\notebooklm_runner.exe" deeplink "%1""#
    );
}
