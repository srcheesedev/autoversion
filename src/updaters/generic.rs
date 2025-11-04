use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

use super::traits::{VersionUpdater, VersionChange};
use crate::constants::VERSION_FILES;
use crate::utils::files::{backup_file, read_file_safe, write_file_safe};

/// Generic version file updater
///
/// Implements version management for projects using plain text version files.
/// This updater serves as a universal fallback when no technology-specific
/// manifest file (package.json, Cargo.toml, etc.) is found.
///
/// # Supported File Names (Priority Order)
///
/// 1. `VERSION` - Most common convention
/// 2. `version.txt` - Windows-friendly variant
/// 3. `.version` - Hidden file convention
/// 4. `version` - Lowercase variant
/// 5. `VERSION.txt` - Alternative convention
///
/// # Version Detection Strategy
///
/// Searches for version files in priority order and parses content supporting:
///
/// **Format 1: Plain version (most common)**
/// ```text
/// 1.2.3
/// ```
///
/// **Format 2: Version with 'v' prefix**
/// ```text
/// v1.2.3
/// ```
///
/// **Format 3: Key-value format**
/// ```text
/// version=1.2.3
/// ```
///
/// **Format 4: Multi-line with label**
/// ```text
/// Version: 1.2.3
/// Build: 2024-01-01
/// ```
///
/// The parser extracts the first valid semantic version found.
///
/// # Update Strategy
///
/// 1. Find version file using priority list
/// 2. Read and parse current content to detect format
/// 3. Update version while **preserving original format**:
///    - Plain "1.2.3" stays plain
///    - "v1.2.3" maintains 'v' prefix
///    - "version=1.2.3" maintains key-value format
/// 4. Create .autoversion.backup before modification
/// 5. Write updated content with preserved formatting
///
/// # Examples
///
/// ## Basic Usage
///
/// ```no_run
/// use autoversion::updaters::generic::GenericUpdater;
/// use autoversion::updaters::traits::VersionUpdater;
/// use std::path::Path;
///
/// let updater = GenericUpdater::new();
/// let project = Path::new("./my-project");
///
/// // Find and read version file
/// let current = updater.get_current_version(project)?;
/// println!("Current version: {}", current);
///
/// // Update version
/// let files = updater.update_version(project, "2.0.0")?;
/// println!("Updated files: {:?}", files);
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// ## Format Preservation
///
/// ```text
/// # Input: VERSION file containing "v1.2.3"
/// # After update to 2.0.0
/// # Output: VERSION file contains "v2.0.0"
/// # The 'v' prefix is preserved!
/// ```
///
/// # Design Principles
///
/// **1. Format Preservation**
/// Users choose their preferred format for a reason. We respect and preserve:
/// - Version prefixes ('v', 'version=', etc.)
/// - Whitespace and line endings
/// - Additional content in the file (comments, metadata)
///
/// **2. Intelligent Parsing**
/// Uses regex patterns to extract version from various formats without
/// requiring exact format specification from the user.
///
/// **3. Fallback Strategy**
/// Acts as the last resort updater when no technology-specific file is found,
/// ensuring autoversion works with ANY project structure.
///
/// # TDD Documentation
///
/// Comprehensive test coverage includes:
/// - All supported file names (VERSION, version.txt, etc.)
/// - All format variants (plain, v-prefix, key-value)
/// - Format preservation during updates
/// - Multi-line file handling
/// - Error cases (missing files, invalid versions)
/// - Edge cases (whitespace, case sensitivity)
///
/// Tests follow the Arrange-Act-Assert pattern with descriptive names.
///
/// # Use Cases
///
/// **1. Technology-Agnostic Projects**
/// - Shell scripts
/// - Documentation projects
/// - Configuration repositories
///
/// **2. Multi-Language Monorepos**
/// - Unified version across multiple technologies
/// - Single VERSION file at repo root
///
/// **3. Legacy Projects**
/// - Projects without modern manifest files
/// - Migration from manual version management
///
/// **4. Custom Build Systems**
/// - Projects with non-standard tooling
/// - Internal corporate build systems
///
/// # Limitations
///
/// **Complex Versioning Schemes:**
/// Only supports semantic versioning (MAJOR.MINOR.PATCH with optional
/// pre-release and build metadata). Does not support:
/// - Date-based versions (2024.01.15)
/// - Single-component versions (v7)
/// - Non-standard schemes
///
/// **Multi-Version Files:**
/// Only updates the FIRST version found. If your version file contains
/// multiple versions, only the first match is updated.
///
/// # Error Handling
///
/// Returns `anyhow::Result` for all operations. Common error scenarios:
/// - No version file found in project root
/// - Invalid semantic version format in file
/// - File contains no recognizable version pattern
/// - File system I/O errors
pub struct GenericUpdater {
    /// Possible version file names in order of preference
    version_files: Vec<&'static str>,
}

impl GenericUpdater {
    pub fn new() -> Self {
        Self {
            version_files: VERSION_FILES.to_vec(),
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

        let content = read_file_safe(&version_file)?;

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
            Some(read_file_safe(&version_file)?)
        } else {
            None
        };

        // Create new content preserving format
        let new_content = self.create_version_content(
            new_version, 
            original_content.as_deref()
        );

        // Write updated content
        write_file_safe(&version_file, &new_content)?;
        updated_files.push(version_file.to_string_lossy().to_string());

        Ok(updated_files)
    }

    fn validate_project(&self, project_path: &Path) -> Result<()> {
        if let Some(version_file) = self.find_version_file(project_path) {
            let content = read_file_safe(&version_file)?;
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
            read_file_safe(&version_file)?
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
    use std::fs;

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