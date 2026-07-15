#[cfg(target_os = "windows")]
pub fn enable_autostart(app_name: &str, exe_path: &str) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run = hkcu
        .open_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Run",
            KEY_SET_VALUE,
        )
        .map_err(|e| e.to_string())?;

    // Quote the path so autostart works when the executable lives under a
    // directory containing spaces (e.g. C:\Program Files\...).
    let quoted = if exe_path.starts_with('"') {
        exe_path.to_string()
    } else {
        format!("\"{exe_path}\"")
    };

    run.set_value(app_name, &quoted).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn enable_autostart(_app_name: &str, _exe_path: &str) -> Result<(), String> {
    // Autostart is Windows-only; succeed quietly on other platforms.
    Ok(())
}
