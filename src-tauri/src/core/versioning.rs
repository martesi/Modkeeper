use crate::models::error::SError;
use crate::models::paths::SPTPathRules;
use crate::utils::version::read_pe_version;
use semver::{Version, VersionReq};

pub struct SptVersionChecker;

impl SptVersionChecker {
    pub fn fetch_and_validate(config: &SPTPathRules) -> Result<String, SError> {
        // Temp bypass preserved from original code
        return Ok("4.0.0".into());

        /* // Original logic preserved for reference/uncommenting:
        read_pe_version(&config.server_dll)
            .map_err(SError::ParseError)
            .and_then(|version| Self::parse(&version))
            .and_then(|v| {
                Self::validate(&v)
                    .and(Ok(v.to_string()))
                    .map_err(|_| SError::UnsupportedSPTVersion(v.to_string()))
            })
        */
    }

    pub fn validate_string(version_str: &str) -> Result<(), SError> {
        Self::parse(version_str).and_then(|v| Self::validate(&v))
    }

    fn parse(version_str: &str) -> Result<Version, SError> {
        Version::parse(version_str).map_err(|e| SError::ParseError(e.to_string()))
    }

    fn validate(version: &Version) -> Result<(), SError> {
        let req = VersionReq::parse(">=4, <5").map_err(|e| SError::ParseError(e.to_string()))?;

        if req.matches(version) {
            Ok(())
        } else {
            Err(SError::UnsupportedSPTVersion(version.to_string()))
        }
    }
}