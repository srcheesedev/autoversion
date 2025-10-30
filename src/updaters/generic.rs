use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};

use super::traits::{VersionUpdater, VersionChange};
use crate::utils::files::backup_file;

/// Generic version file updater for VERSION, version.txt, .version files
pub struct GenericUpdater {
    /// Possible version file names in order of preference
    version_files: Vec<&'static str>,
}

impl GenericUpdater {
    pub fn new() -> Self {
        Self {
            version_files: vec![
                "VERSION",
                "version.txt", 
                ".version",
                "version",
                "VERSION.txt",
            ],
        }
    }

    /// Find the version file in the project
    fn find_version_file(&self, project_path: &Path) -> Option<PathBuf> {
        for &filename in &self.version_files {
            let file_path = project_path.join(filename);
            if file_path.exists() && file_path.is_file() {
                return Some(file_path);
            }
        }
        None
    }

    /// Parse version from file content
    fn parse_version_content(&self, content: &str) -> Result<String> {
        let content = content.trim();
        
        if content.is_empty() {
            return Err(anyhow!("Version file is empty"));
        }

        // Handle different formats:
        // 1. Plain version: "1.2.3"
        // 2. With prefix: "v1.2.3" or "version=1.2.3"
        // 3. Multi-line files - take first line that looks like version
        
        for line in content.lines() {
            let line = line.trim();
            
            if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                continue; // Skip empty lines and comments
            }
            
            // Try to extract version from various formats
            let version = if line.starts_with("version=") {
                line.strip_prefix("version=").unwrap()
            } else if line.starts_with("VERSION=") {
                line.strip_prefix("VERSION=").unwrap()
            } else if line.starts_with('v') && line.len() > 1 {
                line.strip_prefix('v').unwrap()
            } else {
                line
            };
            
            // Basic validation that this looks like a version
            if self.looks_like_version(version) {
                return Ok(version.to_string());
            }
        }
        
        Err(anyhow!("No valid version found in file content"))
    }

    /// Check if a string looks like a semantic version
    fn looks_like_version(&self, s: &str) -> bool {
        use regex::Regex;
        
        // Basic semver pattern: X.Y.Z with optional pre-release and build metadata
        let version_pattern = Regex::new(r"^\d+\.\d+\.\d+").unwrap();
        version_pattern.is_match(s)
    }

    /// Create version file content from version string
    fn create_version_content(&self, version: &str, original_content: Option<&str>) -> String {
        if let Some(original) = original_content {
            // Try to preserve the original format
            if original.contains("version=") {
                format!("version={}\n", version)
            } else if original.contains("VERSION=") {
                format!("VERSION={}\n", version)
            } else if original.trim_start().starts_with('v') {
                format!("v{}\n", version)
            } else {
                format!("{}\n", version)
            }
        } else {
            // Default format for new files
            format!("{}\n", version)
        }
    }

    /// Get the preferred version file name for new files
    fn get_preferred_filename(&self) -> &'static str {
        self.version_files[0] // "VERSION"
    }
}

impl Default for GenericUpdater {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionUpdater for GenericUpdater {
    fn get_current_version(&self, project_path: &Path) -> Result<String> {
        let version_file = self.find_version_file(project_path)
            .ok_or_else(|| anyhow!("No version file found. Looked for: {:?}", self.version_files))?;

        let content = fs::read_to_string(&version_file)
            .map_err(|e| anyhow!("Failed to read version file {}: {}", version_file.display(), e))?;

        self.parse_version_content(&content)
    }

    fn update_version(&self, project_path: &Path, new_version: &str) -> Result<Vec<String>> {
        let version_file = if let Some(existing) = self.find_version_file(project_path) {
            existing
        } else {
            // Create new version file with preferred name
            project_path.join(self.get_preferred_filename())
        };

        let mut updated_files = Vec::new();

        // Read original content if file exists
        let original_content = if version_file.exists() {
            backup_file(&version_file)?;
            Some(fs::read_to_string(&version_file)?)
        } else {
            None
        };

        // Create new content preserving format
        let new_content = self.create_version_content(
            new_version, 
            original_content.as_deref()
        );

        // Write updated content
        fs::write(&version_file, new_content)?;
        updated_files.push(version_file.to_string_lossy().to_string());

        Ok(updated_files)
    }

    fn validate_project(&self, project_path: &Path) -> Result<()> {
        if let Some(version_file) = self.find_version_file(project_path) {
            let content = fs::read_to_string(&version_file)?;
            self.parse_version_content(&content)?;
            Ok(())
        } else {
            // For generic updater, it's okay to not have a version file
            // We can create one if needed
            Ok(())
        }
    }

    fn technology_name(&self) -> &'static str {
        "generic"
    }

    fn get_primary_file(&self, project_path: &Path) -> Result<PathBuf> {
        self.find_version_file(project_path)
            .ok_or_else(|| anyhow!("No version file found"))
            .or_else(|_| {
                // Return the preferred filename even if it doesn't exist yet
                Ok(project_path.join(self.get_preferred_filename()))
            })
    }

    fn can_handle(&self, project_path: &Path) -> bool {
        // Generic updater can always handle a project (as a fallback)
        // But it's more useful if there's already a version file
        self.find_version_file(project_path).is_some()
    }

    fn preview_changes(&self, project_path: &Path, new_version: &str) -> Result<Vec<VersionChange>> {
        let mut changes = Vec::new();
        
        let version_file = if let Some(existing) = self.find_version_file(project_path) {
            existing
        } else {
            project_path.join(self.get_preferred_filename())
        };

        let old_content = if version_file.exists() {
            fs::read_to_string(&version_file)?
        } else {
            String::new()
        };

        let old_version = if version_file.exists() {
            self.get_current_version(project_path).unwrap_or_else(|_| "0.0.0".to_string())
        } else {
            "0.0.0".to_string()
        };

        let new_content = self.create_version_content(
            new_version,
            if old_content.is_empty() { None } else { Some(&old_content) }
        );

        changes.push(VersionChange::new(
            version_file,
            old_content,
            new_content,
            old_version,
            new_version.to_string(),
        ));

        Ok(changes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_version_plain() -> Result<()> {
        let updater = GenericUpdater::new();
        
        assert_eq!(updater.parse_version_content("1.2.3")?, "1.2.3");
        assert_eq!(updater.parse_version_content("1.2.3\n")?, "1.2.3");
        assert_eq!(updater.parse_version_content("  1.2.3  \n")?, "1.2.3");
        
        Ok(())
    }

    #[test]
    fn test_parse_version_with_prefix() -> Result<()> {
        let updater = GenericUpdater::new();
        
        assert_eq!(updater.parse_version_content("v1.2.3")?, "1.2.3");
        assert_eq!(updater.parse_version_content("version=1.2.3")?, "1.2.3");
        assert_eq!(updater.parse_version_content("VERSION=1.2.3")?, "1.2.3");
        
        Ok(())
    }

    #[test]
    fn test_parse_version_multiline() -> Result<()> {
        let updater = GenericUpdater::new();
        
        let content = r#"# Version file
# Generated automatically
version=1.2.3
build=123"#;
        
        assert_eq!(updater.parse_version_content(content)?, "1.2.3");
        
        Ok(())
    }

    #[test]
    fn test_parse_version_empty() {
        let updater = GenericUpdater::new();
        
        assert!(updater.parse_version_content("").is_err());
        assert!(updater.parse_version_content("   \n\n  ").is_err());
    }

    #[test]
    fn test_parse_version_invalid() {
        let updater = GenericUpdater::new();
        
        assert!(updater.parse_version_content("not-a-version").is_err());
        assert!(updater.parse_version_content("1.2").is_err());
    }

    #[test]
    fn test_looks_like_version() {
        let updater = GenericUpdater::new();
        
        assert!(updater.looks_like_version("1.2.3"));
        assert!(updater.looks_like_version("0.1.0"));
        assert!(updater.looks_like_version("10.20.30"));
        assert!(updater.looks_like_version("1.2.3-alpha.1"));
        assert!(updater.looks_like_version("1.2.3+build.1"));
        
        assert!(!updater.looks_like_version("1.2"));
        assert!(!updater.looks_like_version("not-a-version"));
        assert!(!updater.looks_like_version(""));
    }

    #[test]
    fn test_create_version_content() {
        let updater = GenericUpdater::new();
        
        // New file (no original content)
        assert_eq!(updater.create_version_content("1.2.3", None), "1.2.3\n");
        
        // Preserve format
        assert_eq!(updater.create_version_content("1.2.3", Some("v1.0.0")), "v1.2.3\n");
        assert_eq!(updater.create_version_content("1.2.3", Some("version=1.0.0")), "version=1.2.3\n");
        assert_eq!(updater.create_version_content("1.2.3", Some("VERSION=1.0.0")), "VERSION=1.2.3\n");
        assert_eq!(updater.create_version_content("1.2.3", Some("1.0.0")), "1.2.3\n");
    }

    #[test]
    fn test_find_version_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();
        
        let updater = GenericUpdater::new();
        
        // No version file found
        assert!(updater.find_version_file(project_path).is_none());
        
        // Create VERSION file
        fs::write(project_path.join("VERSION"), "1.0.0")?;
        let found = updater.find_version_file(project_path);
        assert!(found.is_some());
        assert_eq!(found.unwrap().file_name().unwrap(), "VERSION");
        
        Ok(())
    }

    #[test]
    fn test_get_current_version() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();
        
        fs::write(project_path.join("VERSION"), "1.2.3")?;
        
        let updater = GenericUpdater::new();
        let version = updater.get_current_version(project_path)?;
        
        assert_eq!(version, "1.2.3");
        Ok(())
    }

    #[test]
    fn test_update_version_existing_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();
        
        fs::write(project_path.join("VERSION"), "1.0.0")?;
        
        let updater = GenericUpdater::new();
        let updated_files = updater.update_version(project_path, "2.0.0")?;
        
        assert_eq!(updated_files.len(), 1);
        
        let new_version = updater.get_current_version(project_path)?;
        assert_eq!(new_version, "2.0.0");
        
        Ok(())
    }

    #[test]
    fn test_update_version_new_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();
        
        let updater = GenericUpdater::new();
        let updated_files = updater.update_version(project_path, "1.0.0")?;
        
        assert_eq!(updated_files.len(), 1);
        assert!(project_path.join("VERSION").exists());
        
        let version = updater.get_current_version(project_path)?;
        assert_eq!(version, "1.0.0");
        
        Ok(())
    }

    #[test]
    fn test_can_handle() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();
        
        let updater = GenericUpdater::new();
        
        // Should not handle empty directory
        assert!(!updater.can_handle(project_path));
        
        // Should handle with version file
        fs::write(project_path.join("VERSION"), "1.0.0")?;
        assert!(updater.can_handle(project_path));
        
        Ok(())
    }

    #[test]
    fn test_preview_changes() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();
        
        fs::write(project_path.join("VERSION"), "1.0.0")?;
        
        let updater = GenericUpdater::new();
        let changes = updater.preview_changes(project_path, "2.0.0")?;
        
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].old_version, "1.0.0");
        assert_eq!(changes[0].new_version, "2.0.0");
        assert_eq!(changes[0].new_content, "2.0.0\n");
        
        Ok(())
    }

    #[test]
    fn test_technology_name() {
        let updater = GenericUpdater::new();
        assert_eq!(updater.technology_name(), "generic");
    }
}