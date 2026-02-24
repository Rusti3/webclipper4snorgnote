use std::path::Path;

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolRegistrationStatus {
    AlreadyRegistered,
    Updated,
    Skipped,
}

pub fn protocol_command_value(exe_path: &Path) -> String {
    format!("\"{}\" deeplink \"%1\"", exe_path.display())
}

#[cfg(windows)]
pub fn ensure_protocol_registered(
    scheme: &str,
    exe_path: &Path,
) -> Result<ProtocolRegistrationStatus> {
    use anyhow::Context;
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    let scheme = scheme.trim().to_ascii_lowercase();
    if scheme.is_empty() {
        anyhow::bail!("scheme must not be empty");
    }

    let classes_path = format!("Software\\Classes\\{scheme}");
    let command_path = format!("{classes_path}\\shell\\open\\command");
    let expected_command = protocol_command_value(exe_path);

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    let existing_command = hkcu
        .open_subkey(&command_path)
        .ok()
        .and_then(|key| key.get_value::<String, _>("").ok());

    let already_registered = existing_command.as_deref() == Some(expected_command.as_str());
    if already_registered {
        return Ok(ProtocolRegistrationStatus::AlreadyRegistered);
    }

    let (scheme_key, _) = hkcu
        .create_subkey(&classes_path)
        .with_context(|| format!("failed to create/open registry key: {classes_path}"))?;
    scheme_key
        .set_value("", &format!("URL:{scheme} Protocol"))
        .context("failed to set protocol description")?;
    scheme_key
        .set_value("URL Protocol", &"")
        .context("failed to set URL Protocol value")?;

    let (command_key, _) = hkcu
        .create_subkey(&command_path)
        .with_context(|| format!("failed to create/open registry key: {command_path}"))?;
    command_key
        .set_value("", &expected_command)
        .context("failed to set shell open command")?;

    Ok(ProtocolRegistrationStatus::Updated)
}

#[cfg(not(windows))]
pub fn ensure_protocol_registered(
    _scheme: &str,
    _exe_path: &Path,
) -> Result<ProtocolRegistrationStatus> {
    Ok(ProtocolRegistrationStatus::Skipped)
}
