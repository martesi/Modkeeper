#[cfg(target_os = "windows")]
pub fn read_pe_version(path: &camino::Utf8PathBuf) -> Result<String, String> {
    let p = path.as_str();
    // Using double quotes inside the PS expression and escaping them is often safer
    let ps_expr = format!(
        r#"([System.Diagnostics.FileVersionInfo]::GetVersionInfo("{}")).FileVersion"#,
        p
    );

    let output = std::process::Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(ps_expr)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        // IMPORTANT: Capture stderr to see what actually went wrong
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("PS Error: {}", err.trim()));
    }

    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        Err("No version found in file".into())
    } else {
        Ok(s)
    }
}

#[cfg(target_os = "macos")]
pub fn read_pe_version(path: &Utf8PathBuf) -> Result<String, String> {
    // macOS: Try using `strings` + grep to extract version from PE binary
    let output = Command::new("strings")
        .arg(path.as_str())
        .output()
        .map_err(|e| format!("Failed to run strings: {}", e))?;

    if !output.status.success() {
        return Err("strings command failed".into());
    }

    let content = String::from_utf8_lossy(&output.stdout);
    // Look for FileVersion pattern (e.g., "4.0.0.123")
    for line in content.lines() {
        if let Some(caps) = regex::Regex::new(r"(\d+\.\d+(?:\.\d+)?)")
            .ok()
            .and_then(|re| re.find(line))
        {
            let version = caps.as_str().to_string();
            if version.starts_with('4') || version.starts_with('3') {
                return Ok(version);
            }
        }
    }
    Err("No version pattern found in PE binary".into())
}

#[cfg(target_os = "linux")]
pub fn read_pe_version(path: &Utf8PathBuf) -> Result<String, String> {
    // Linux: Try using `file` command to identify PE binary, then extract version
    let output = Command::new("file")
        .arg("-b")
        .arg(path.as_str())
        .output()
        .map_err(|e| format!("Failed to run file: {}", e))?;

    let file_info = String::from_utf8_lossy(&output.stdout);
    if !file_info.contains("PE32") && !file_info.contains("PE64") {
        return Err("File is not a PE binary".into());
    }

    // Try strings to extract version info
    let output = Command::new("strings")
        .arg(path.as_str())
        .output()
        .map_err(|e| format!("Failed to run strings: {}", e))?;

    let content = String::from_utf8_lossy(&output.stdout);
    for line in content.lines() {
        if let Some(caps) = regex::Regex::new(r"(\d+\.\d+(?:\.\d+)?)")
            .ok()
            .and_then(|re| re.find(line))
        {
            let version = caps.as_str().to_string();
            if version.starts_with('4') || version.starts_with('3') {
                return Ok(version);
            }
        }
    }
    Err("No version pattern found in PE binary".into())
}

