use mod_keeper_lib::utils::version::read_pe_version;
use camino::Utf8PathBuf;

#[cfg(target_os = "windows")]
#[test]
fn test_read_windows_system_dll_version() {
    // Every Windows install has kernel32.dll in System32
    let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    let path = Utf8PathBuf::from(system_root).join("System32\\kernel32.dll");

    let result = read_pe_version(&path);

    assert!(
        result.is_ok(),
        "Should read version from kernel32.dll: {:?}",
        result.err()
    );
    let version = result.unwrap();
    // Version should look like "10.0.xxxxx.xxxx"
    assert!(version.contains('.'));
    assert!(version.chars().next().unwrap().is_ascii_digit());
}

#[cfg(target_os = "windows")]
#[test]
fn test_powershell_availability() {
    let output = std::process::Command::new("powershell")
        .arg("-Command")
        .arg("echo 'hello'")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");
}
