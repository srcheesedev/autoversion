use anyhow::{anyhow, Result};
use semver::Version;
use std::path::Path;

use crate::git::history::CommitAnalyzer;

/// Version bump types
#[derive(Debug, Clone, PartialEq)]
pub enum BumpType {
    Major,
    Minor,
    Patch,
}

impl std::fmt::Display for BumpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BumpType::Major => write!(f, "major"),
            BumpType::Minor => write!(f, "minor"),
            BumpType::Patch => write!(f, "patch"),
        }
    }
}

impl std::str::FromStr for BumpType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "major" => Ok(BumpType::Major),
            "minor" => Ok(BumpType::Minor),
            "patch" => Ok(BumpType::Patch),
            _ => Err(anyhow!("Invalid bump type: {}. Valid options: major, minor, patch", s)),
        }
    }
}

/// Core version bumping logic
pub struct VersionBumper {
    commit_analyzer: CommitAnalyzer,
}

impl VersionBumper {
    pub fn new() -> Self {
        Self {
            commit_analyzer: CommitAnalyzer::new(),
        }
    }

    /// Parse a version string into a semver Version
    pub fn parse_version(&self, version_str: &str) -> Result<Version> {
        // Remove common prefixes like 'v' or '='
        let clean_version = version_str
            .trim_start_matches('v')
            .trim_start_matches('=')
            .trim();

        Version::parse(clean_version)
            .map_err(|e| anyhow!("Invalid version format '{}': {}", version_str, e))
    }

    /// Validate that a version string is valid semver
    pub fn validate_version(&self, version_str: &str) -> Result<()> {
        self.parse_version(version_str)?;
        Ok(())
    }

    /// Automatically determine bump type based on commit history
    pub fn auto_bump(&self, current_version: &str, repo_path: &Path) -> Result<(String, String)> {
        let version = self.parse_version(current_version)?;
        
        // Analyze commits since last version tag
        let bump_type = self.commit_analyzer.analyze_commits_since_version(repo_path, &version)?;
        
        let new_version = self.apply_bump(&version, &bump_type)?;
        
        Ok((new_version.to_string(), bump_type.to_string()))
    }

    /// Manually bump version by specified type
    pub fn manual_bump(&self, current_version: &str, bump_type_str: &str) -> Result<(String, String)> {
        let version = self.parse_version(current_version)?;
        let bump_type: BumpType = bump_type_str.parse()?;
        
        let new_version = self.apply_bump(&version, &bump_type)?;
        
        Ok((new_version.to_string(), bump_type.to_string()))
    }

    /// Apply a bump type to a version
    fn apply_bump(&self, version: &Version, bump_type: &BumpType) -> Result<Version> {
        let mut new_version = version.clone();
        
        match bump_type {
            BumpType::Major => {
                new_version.major += 1;
                new_version.minor = 0;
                new_version.patch = 0;
            }
            BumpType::Minor => {
                new_version.minor += 1;
                new_version.patch = 0;
            }
            BumpType::Patch => {
                new_version.patch += 1;
            }
        }

        // Clear pre-release and build metadata for clean releases
        new_version.pre = semver::Prerelease::EMPTY;
        new_version.build = semver::BuildMetadata::EMPTY;
        
        Ok(new_version)
    }

    /// Check if version A is greater than version B
    pub fn is_greater(&self, version_a: &str, version_b: &str) -> Result<bool> {
        let a = self.parse_version(version_a)?;
        let b = self.parse_version(version_b)?;
        Ok(a > b)
    }

    /// Get the next version for a given bump type
    pub fn preview_bump(&self, current_version: &str, bump_type_str: &str) -> Result<String> {
        let version = self.parse_version(current_version)?;
        let bump_type: BumpType = bump_type_str.parse()?;
        let new_version = self.apply_bump(&version, &bump_type)?;
        Ok(new_version.to_string())
    }
}

impl Default for VersionBumper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_clean() {
        let bumper = VersionBumper::new();
        
        assert_eq!(bumper.parse_version("1.2.3").unwrap().to_string(), "1.2.3");
        assert_eq!(bumper.parse_version("v1.2.3").unwrap().to_string(), "1.2.3");
        assert_eq!(bumper.parse_version("=1.2.3").unwrap().to_string(), "1.2.3");
        assert_eq!(bumper.parse_version(" 1.2.3 ").unwrap().to_string(), "1.2.3");
    }

    #[test]
    fn test_parse_version_invalid() {
        let bumper = VersionBumper::new();
        
        assert!(bumper.parse_version("1.2").is_err());
        assert!(bumper.parse_version("1.2.3.4").is_err());
        assert!(bumper.parse_version("not-a-version").is_err());
    }

    #[test]
    fn test_manual_bump_patch() {
        let bumper = VersionBumper::new();
        let (new_version, bump_type) = bumper.manual_bump("1.2.3", "patch").unwrap();
        
        assert_eq!(new_version, "1.2.4");
        assert_eq!(bump_type, "patch");
    }

    #[test]
    fn test_manual_bump_minor() {
        let bumper = VersionBumper::new();
        let (new_version, bump_type) = bumper.manual_bump("1.2.3", "minor").unwrap();
        
        assert_eq!(new_version, "1.3.0");
        assert_eq!(bump_type, "minor");
    }

    #[test]
    fn test_manual_bump_major() {
        let bumper = VersionBumper::new();
        let (new_version, bump_type) = bumper.manual_bump("1.2.3", "major").unwrap();
        
        assert_eq!(new_version, "2.0.0");
        assert_eq!(bump_type, "major");
    }

    #[test]
    fn test_is_greater() {
        let bumper = VersionBumper::new();
        
        assert!(bumper.is_greater("1.2.4", "1.2.3").unwrap());
        assert!(bumper.is_greater("1.3.0", "1.2.3").unwrap());
        assert!(bumper.is_greater("2.0.0", "1.2.3").unwrap());
        assert!(!bumper.is_greater("1.2.3", "1.2.4").unwrap());
    }

    #[test]
    fn test_preview_bump() {
        let bumper = VersionBumper::new();
        
        assert_eq!(bumper.preview_bump("1.2.3", "patch").unwrap(), "1.2.4");
        assert_eq!(bumper.preview_bump("1.2.3", "minor").unwrap(), "1.3.0");
        assert_eq!(bumper.preview_bump("1.2.3", "major").unwrap(), "2.0.0");
    }

    #[test]
    fn test_bump_type_parsing() {
        assert_eq!("major".parse::<BumpType>().unwrap(), BumpType::Major);
        assert_eq!("minor".parse::<BumpType>().unwrap(), BumpType::Minor);
        assert_eq!("patch".parse::<BumpType>().unwrap(), BumpType::Patch);
        assert_eq!("MAJOR".parse::<BumpType>().unwrap(), BumpType::Major);
        
        assert!("invalid".parse::<BumpType>().is_err());
    }

    #[test]
    fn test_prerelease_clearing() {
        let bumper = VersionBumper::new();
        let (new_version, _) = bumper.manual_bump("1.2.3-alpha.1", "patch").unwrap();
        
        assert_eq!(new_version, "1.2.4");
    }

    #[test]
    fn test_maven_snapshot_clearing() {
        let bumper = VersionBumper::new();
        
        // Maven SNAPSHOT versions should be parsed and cleared
        let (new_version, _) = bumper.manual_bump("1.2.3-SNAPSHOT", "patch").unwrap();
        assert_eq!(new_version, "1.2.4");
        
        let (new_version, _) = bumper.manual_bump("1.2.3-SNAPSHOT", "minor").unwrap();
        assert_eq!(new_version, "1.3.0");
        
        let (new_version, _) = bumper.manual_bump("1.2.3-SNAPSHOT", "major").unwrap();
        assert_eq!(new_version, "2.0.0");
    }
}