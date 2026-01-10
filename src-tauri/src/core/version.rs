use crate::models::error::SError;
use crate::models::paths::SPTPathRules;
use regex;
use semver::{Version, VersionReq};
use serde_json::Value;
use std::fs;

/// Fetches the version from the SPT registry file and validates it against supported ranges.
pub fn fetch_and_validate(config: &SPTPathRules) -> Result<String, SError> {
    // Read the registry.json file
    let registry_path = &config.server_registry;
    let content = fs::read_to_string(registry_path)
        .map_err(|e| SError::IOError(format!("Failed to read registry file: {}", e)))?;

    // Parse JSON
    let json: Value = serde_json::from_str(&content)
        .map_err(|e| SError::ParseError(format!("Failed to parse registry JSON: {}", e)))?;

    // Look for SPT_Version or SPT_Vesion (handling potential typo)
    let version_str = json
        .get("SPT_Version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            SError::ParseError("SPT_Version field not found in registry.json".to_string())
        })?;

    // Extract version number from string like "SPT 4.0.11 - 278e72"
    // Pattern: "SPT " followed by version number, then optional " - ..."
    let version_number = extract_version_number(version_str)?;

    // Parse and validate the version
    let version = parse(&version_number)?;
    validate(&version)?;

    Ok(version.to_string())
}

/// Extracts version number from a string like "SPT 4.0.11 - 278e72" -> "4.0.11"
fn extract_version_number(version_str: &str) -> Result<String, SError> {
    // Use regex to extract version pattern (e.g., "4.0.11")
    let re = regex::Regex::new(r"(\d+\.\d+\.\d+)")
        .map_err(|e| SError::ParseError(format!("Failed to create regex: {}", e)))?;

    if let Some(caps) = re.captures(version_str) {
        if let Some(version_match) = caps.get(1) {
            return Ok(version_match.as_str().to_string());
        }
    }

    Err(SError::ParseError(format!(
        "Could not extract version number from: {}",
        version_str
    )))
}

/// Validates a version string against the supported SPT version requirements.
pub fn validate_string(version_str: &str) -> Result<(), SError> {
    let version = parse(version_str)?;
    validate(&version)
}

/// Parses a string into a SemVer Version, handling errors with early exit.
fn parse(version_str: &str) -> Result<Version, SError> {
    Version::parse(version_str).map_err(|e| SError::ParseError(e.to_string()))
}

/// Checks if the provided version matches the hardcoded requirement range (>=4, <5).
fn validate(version: &Version) -> Result<(), SError> {
    let req_str = "^4";
    let req = VersionReq::parse(req_str).map_err(|e| SError::ParseError(e.to_string()))?;

    if req.matches(version) {
        return Ok(());
    }

    Err(SError::UnsupportedSPTVersion(version.to_string()))
}
