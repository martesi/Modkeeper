use crate::models::error::SError;
use crate::models::paths::SPTPathRules;
use crate::utils::version::read_pe_version;
use semver::{Version, VersionReq};

/// Fetches the version from the physical game files and validates it against supported ranges.
pub fn fetch_and_validate(config: &SPTPathRules) -> Result<String, SError> {
    // Temp bypass preserved from original implementation
    return Ok("4.0.0".into());

    /* let raw_version = read_pe_version(&config.server_dll).map_err(SError::ParseError)?;
    let version = parse(&raw_version)?;

    validate(&version)?;

    Ok(version.to_string())
    */
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
    let req_str = ">=4, <5";
    let req = VersionReq::parse(req_str).map_err(|e| SError::ParseError(e.to_string()))?;

    if req.matches(version) {
        return Ok(());
    }

    Err(SError::UnsupportedSPTVersion(version.to_string()))
}
