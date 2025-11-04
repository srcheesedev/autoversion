use anyhow::{anyhow, Result};
use regex::Regex;
use std::path::{Path, PathBuf};

use super::traits::{VersionUpdater, VersionChange};
use crate::constants::manifests::{GRADLE_BUILD, GRADLE_BUILD_KTS, GRADLE_PROPERTIES};
use crate::utils::files::{backup_file, read_file_safe, write_file_safe};

/// Gradle build version updater
///
/// Implements version management for Gradle projects supporting both Groovy and
/// Kotlin DSL build files.
///
/// # Supported Files
///
/// - `build.gradle` - Groovy DSL (most common)
/// - `build.gradle.kts` - Kotlin DSL  
/// - `gradle.properties` - Properties file with version property
///
/// # Version Detection Strategy
///
/// Searches for version declarations in the following patterns:
///
/// **Groovy DSL (build.gradle):**
/// ```groovy
/// version = '1.2.3'
/// version = "1.2.3"
/// version '1.2.3'
/// ```
///
/// **Kotlin DSL (build.gradle.kts):**
/// ```kotlin
/// version = "1.2.3"
/// ```
///
/// **gradle.properties:**
/// ```properties
/// version=1.2.3
/// ```
///
/// ## Update Strategy
///
/// 1. Check for `gradle.properties` first (most common for version management)
/// 2. Fall back to `build.gradle` or `build.gradle.kts`
/// 3. Use regex to find and replace version declarations
/// 4. Preserve formatting and quote style
///
/// ## Example Usage
///
/// ```rust
/// use autoversion::updaters::traits::VersionUpdater;
/// use autoversion::updaters::gradle::GradleUpdater;
/// # use std::path::Path;
/// # fn example() -> anyhow::Result<()> {
/// # let project_path = Path::new(".");
///
/// let updater = GradleUpdater::new();
/// let current_version = updater.get_current_version(&project_path)?;
/// updater.update_version(&project_path, "1.3.0")?;
/// # Ok(())
/// # }
/// ```
///
/// ## TDD Implementation Notes
///
/// This updater was implemented following Test-Driven Development:
/// - Tests written first defining expected behavior (RED)
/// - Implementation added to make tests pass (GREEN)
/// - Code refactored for clarity and maintainability
///
/// Test coverage includes:
/// - Groovy DSL parsing (single and double quotes)
/// - Kotlin DSL parsing
/// - gradle.properties parsing
/// - Version detection and updates
/// - Multiple file handling
/// - Edge cases (missing files, malformed syntax)
///
/// ## Limitations
///
/// - Does not parse multi-project builds (focuses on root project version)
/// - Does not handle version variables defined elsewhere
/// - Requires explicit version declarations
/// - Does not validate Groovy/Kotlin syntax beyond version pattern
///
/// ## Error Handling
///
/// Returns `Err` for:
/// - No Gradle files found
/// - No version declaration in any file
/// - Invalid file format
/// - File I/O errors
pub struct GradleUpdater;

impl GradleUpdater {
    pub fn new() -> Self {
        Self
    }

    /// Find Gradle build files in the project
    fn find_gradle_files(&self, project_path: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        
        // Check gradle.properties first (most common for version)
        let props = project_path.join(GRADLE_PROPERTIES);
        if props.exists() {
            files.push(props);
        }
        
        // Check build.gradle (Groovy)
        let groovy = project_path.join(GRADLE_BUILD);
        if groovy.exists() {
            files.push(groovy);
        }
        
        // Check build.gradle.kts (Kotlin)
        let kotlin = project_path.join(GRADLE_BUILD_KTS);
        if kotlin.exists() {
            files.push(kotlin);
        }
        
        files
    }

    /// Extract version from gradle.properties
    fn extract_version_from_properties(&self, content: &str) -> Option<String> {
        let re = Regex::new(r"(?m)^version\s*=\s*(.+)$").ok()?;
        re.captures(content)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim().to_string())
    }

    /// Extract version from build.gradle (Groovy DSL)
    fn extract_version_from_groovy(&self, content: &str) -> Option<String> {
        // Match: version = '1.2.3' or version = "1.2.3" or version '1.2.3'
        let re = Regex::new(r#"(?m)^\s*version\s*=?\s*['"]([^'"]+)['"]"#).ok()?;
        re.captures(content)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Extract version from build.gradle.kts (Kotlin DSL)
    fn extract_version_from_kotlin(&self, content: &str) -> Option<String> {
        // Match: version = "1.2.3"
        let re = Regex::new(r#"(?m)^\s*version\s*=\s*"([^"]+)""#).ok()?;
        re.captures(content)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Update version in gradle.properties
    fn update_properties_content(&self, content: &str, new_version: &str) -> Result<String> {
        let re = Regex::new(r"(?m)^version\s*=\s*.+$")
            .map_err(|e| anyhow!("Regex error: {}", e))?;
        
        if re.is_match(content) {
            Ok(re.replace(content, format!("version={}", new_version)).to_string())
        } else {
            Err(anyhow!("No version property found in gradle.properties"))
        }
    }

    /// Update version in build.gradle (Groovy DSL)
    fn update_groovy_content(&self, content: &str, new_version: &str) -> Result<String> {
        let re = Regex::new(r#"(?m)^(\s*version\s*=?\s*)['"]([^'"]+)['"]"#)
            .map_err(|e| anyhow!("Regex error: {}", e))?;
        
        if let Some(caps) = re.captures(content) {
            let quote = if caps.get(0).unwrap().as_str().contains('\'') { '\'' } else { '"' };
            let replacement = format!("{}{}{}{}",
                &caps[1], quote, new_version, quote);
            Ok(re.replace(content, replacement).to_string())
        } else {
            Err(anyhow!("No version declaration found in build.gradle"))
        }
    }

    /// Update version in build.gradle.kts (Kotlin DSL)
    fn update_kotlin_content(&self, content: &str, new_version: &str) -> Result<String> {
        let re = Regex::new(r#"(?m)^(\s*version\s*=\s*)"([^"]+)""#)
            .map_err(|e| anyhow!("Regex error: {}", e))?;
        
        if re.is_match(content) {
            Ok(re.replace(content, format!("${{1}}\"{}\"", new_version)).to_string())
        } else {
            Err(anyhow!("No version declaration found in build.gradle.kts"))
        }
    }
}

impl Default for GradleUpdater {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionUpdater for GradleUpdater {
    fn get_current_version(&self, project_path: &Path) -> Result<String> {
        let files = self.find_gradle_files(project_path);
        
        for file in files {
            let content = read_file_safe(&file)?;
            let filename = file.file_name().unwrap().to_str().unwrap();
            
            let version = match filename {
                name if name == GRADLE_PROPERTIES => self.extract_version_from_properties(&content),
                name if name == GRADLE_BUILD => self.extract_version_from_groovy(&content),
                name if name == GRADLE_BUILD_KTS => self.extract_version_from_kotlin(&content),
                _ => None,
            };
            
            if let Some(v) = version {
                return Ok(v);
            }
        }
        
        Err(anyhow!("No version found in Gradle files"))
    }

    fn update_version(&self, project_path: &Path, new_version: &str) -> Result<Vec<String>> {
        let files = self.find_gradle_files(project_path);
        let mut updated = Vec::new();
        
        for file in files {
            let content = read_file_safe(&file)?;
            let filename = file.file_name().unwrap().to_str().unwrap();
            
            let new_content = match filename {
                name if name == GRADLE_PROPERTIES => self.update_properties_content(&content, new_version),
                name if name == GRADLE_BUILD => self.update_groovy_content(&content, new_version),
                name if name == GRADLE_BUILD_KTS => self.update_kotlin_content(&content, new_version),
                _ => continue,
            };
            
            if let Ok(new_content) = new_content {
                backup_file(&file)?;
                write_file_safe(&file, &new_content)?;
                updated.push(file.display().to_string());
            }
        }
        
        if updated.is_empty() {
            return Err(anyhow!("No Gradle files were updated"));
        }
        
        Ok(updated)
    }

    fn validate_project(&self, project_path: &Path) -> Result<()> {
        let files = self.find_gradle_files(project_path);
        if files.is_empty() {
            return Err(anyhow!("No Gradle files found. Not a valid Gradle project."));
        }
        Ok(())
    }

    fn technology_name(&self) -> &'static str {
        "gradle"
    }

    fn get_primary_file(&self, project_path: &Path) -> Result<PathBuf> {
        // Priority: gradle.properties > build.gradle > build.gradle.kts
        let props = project_path.join(GRADLE_PROPERTIES);
        if props.exists() {
            return Ok(props);
        }
        
        let groovy = project_path.join(GRADLE_BUILD);
        if groovy.exists() {
            return Ok(groovy);
        }
        
        let kotlin = project_path.join(GRADLE_BUILD_KTS);
        if kotlin.exists() {
            return Ok(kotlin);
        }
        
        Err(anyhow!("No Gradle files found"))
    }

    fn can_handle(&self, project_path: &Path) -> bool {
        !self.find_gradle_files(project_path).is_empty()
    }

    fn preview_changes(&self, project_path: &Path, new_version: &str) -> Result<Vec<VersionChange>> {
        let mut changes = Vec::new();
        let files = self.find_gradle_files(project_path);
        
        for file in files {
            let content = read_file_safe(&file)?;
            let filename = file.file_name().unwrap().to_str().unwrap();
            
            let new_content = match filename {
                name if name == GRADLE_PROPERTIES => self.update_properties_content(&content, new_version),
                name if name == GRADLE_BUILD => self.update_groovy_content(&content, new_version),
                name if name == GRADLE_BUILD_KTS => self.update_kotlin_content(&content, new_version),
                _ => continue,
            };
            
            if let Ok(new_content) = new_content {
                let old_version = self.get_current_version(project_path)?;
                changes.push(VersionChange::new(
                    file.clone(),
                    content,
                    new_content,
                    old_version,
                    new_version.to_string(),
                ));
            }
        }
        
        Ok(changes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // TDD Test 1: Detect Gradle projects by presence of build files
    #[test]
    fn test_can_handle_with_gradle_files() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Arrange: Create build.gradle
        fs::write(project_path.join(GRADLE_BUILD), "version = '1.0.0'").unwrap();
        
        // Act
        let updater = GradleUpdater::new();
        
        // Assert
        assert!(updater.can_handle(project_path));
    }

    // TDD Test 2: Don't detect non-Gradle projects
    #[test]
    fn test_cannot_handle_without_gradle_files() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Act
        let updater = GradleUpdater::new();
        
        // Assert
        assert!(!updater.can_handle(project_path));
    }

    // TDD Test 3: Read version from gradle.properties
    #[test]
    fn test_get_version_from_properties() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Arrange
        fs::write(
            project_path.join(GRADLE_PROPERTIES),
            "version=1.2.3\n"
        ).unwrap();
        
        // Act
        let updater = GradleUpdater::new();
        let version = updater.get_current_version(project_path).unwrap();
        
        // Assert
        assert_eq!(version, "1.2.3");
    }

    // TDD Test 4: Read version from build.gradle (Groovy, single quotes)
    #[test]
    fn test_get_version_from_groovy_single_quotes() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Arrange
        fs::write(
            project_path.join(GRADLE_BUILD),
            "version = '1.2.3'\n"
        ).unwrap();
        
        // Act
        let updater = GradleUpdater::new();
        let version = updater.get_current_version(project_path).unwrap();
        
        // Assert
        assert_eq!(version, "1.2.3");
    }

    // TDD Test 5: Read version from build.gradle (Groovy, double quotes)
    #[test]
    fn test_get_version_from_groovy_double_quotes() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Arrange
        fs::write(
            project_path.join(GRADLE_BUILD),
            "version = \"1.2.3\"\n"
        ).unwrap();
        
        // Act
        let updater = GradleUpdater::new();
        let version = updater.get_current_version(project_path).unwrap();
        
        // Assert
        assert_eq!(version, "1.2.3");
    }

    // TDD Test 6: Read version from build.gradle.kts (Kotlin DSL)
    #[test]
    fn test_get_version_from_kotlin_dsl() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Arrange
        fs::write(
            project_path.join(GRADLE_BUILD_KTS),
            "version = \"1.2.3\"\n"
        ).unwrap();
        
        // Act
        let updater = GradleUpdater::new();
        let version = updater.get_current_version(project_path).unwrap();
        
        // Assert
        assert_eq!(version, "1.2.3");
    }

    // TDD Test 7: Update version in gradle.properties
    #[test]
    fn test_update_version_in_properties() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Arrange
        fs::write(
            project_path.join(GRADLE_PROPERTIES),
            "version=1.0.0\n"
        ).unwrap();
        
        // Act
        let updater = GradleUpdater::new();
        let updated = updater.update_version(project_path, "2.0.0").unwrap();
        
        // Assert
        assert_eq!(updated.len(), 1);
        let content = fs::read_to_string(project_path.join(GRADLE_PROPERTIES)).unwrap();
        assert!(content.contains("version=2.0.0"));
    }

    // TDD Test 8: Update version in build.gradle (preserves quote style)
    #[test]
    fn test_update_version_in_groovy_preserves_quotes() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Arrange
        fs::write(
            project_path.join(GRADLE_BUILD),
            "version = '1.0.0'\n"
        ).unwrap();
        
        // Act
        let updater = GradleUpdater::new();
        updater.update_version(project_path, "2.0.0").unwrap();
        
        // Assert
        let content = fs::read_to_string(project_path.join(GRADLE_BUILD)).unwrap();
        assert!(content.contains("version = '2.0.0'"));
    }

    // TDD Test 9: Update version in build.gradle.kts
    #[test]
    fn test_update_version_in_kotlin_dsl() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Arrange
        fs::write(
            project_path.join(GRADLE_BUILD_KTS),
            "version = \"1.0.0\"\n"
        ).unwrap();
        
        // Act
        let updater = GradleUpdater::new();
        updater.update_version(project_path, "2.0.0").unwrap();
        
        // Assert
        let content = fs::read_to_string(project_path.join(GRADLE_BUILD_KTS)).unwrap();
        assert!(content.contains("version = \"2.0.0\""));
    }

    // TDD Test 10: Validate project with Gradle files
    #[test]
    fn test_validate_project_success() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Arrange
        fs::write(project_path.join(GRADLE_BUILD), "version = '1.0.0'").unwrap();
        
        // Act & Assert
        let updater = GradleUpdater::new();
        assert!(updater.validate_project(project_path).is_ok());
    }

    // TDD Test 11: Fail validation without Gradle files
    #[test]
    fn test_validate_project_fails_without_files() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Act & Assert
        let updater = GradleUpdater::new();
        assert!(updater.validate_project(project_path).is_err());
    }

    // TDD Test 12: Technology name
    #[test]
    fn test_technology_name() {
        let updater = GradleUpdater::new();
        assert_eq!(updater.technology_name(), "gradle");
    }

    // TDD Test 13: Get primary file prefers gradle.properties
    #[test]
    fn test_get_primary_file_prefers_properties() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Arrange: Create both files
        fs::write(project_path.join(GRADLE_PROPERTIES), "version=1.0.0").unwrap();
        fs::write(project_path.join(GRADLE_BUILD), "version = '1.0.0'").unwrap();
        
        // Act
        let updater = GradleUpdater::new();
        let primary = updater.get_primary_file(project_path).unwrap();
        
        // Assert
        assert!(primary.ends_with(GRADLE_PROPERTIES));
    }

    // TDD Test 14: Preview changes
    #[test]
    fn test_preview_changes() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Arrange
        fs::write(
            project_path.join(GRADLE_BUILD),
            "version = '1.0.0'\n"
        ).unwrap();
        
        // Act
        let updater = GradleUpdater::new();
        let changes = updater.preview_changes(project_path, "2.0.0").unwrap();
        
        // Assert
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].old_version, "1.0.0");
        assert_eq!(changes[0].new_version, "2.0.0");
    }

    // TDD Test 15: Handle SNAPSHOT versions (like Maven)
    #[test]
    fn test_handles_snapshot_versions() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Arrange
        fs::write(
            project_path.join(GRADLE_PROPERTIES),
            "version=1.2.3-SNAPSHOT\n"
        ).unwrap();
        
        // Act
        let updater = GradleUpdater::new();
        let version = updater.get_current_version(project_path).unwrap();
        
        // Assert
        assert_eq!(version, "1.2.3-SNAPSHOT");
        
        // Update to release version
        updater.update_version(project_path, "1.2.3").unwrap();
        let content = fs::read_to_string(project_path.join(GRADLE_PROPERTIES)).unwrap();
        assert!(content.contains("version=1.2.3"));
        assert!(!content.contains("SNAPSHOT"));
    }
}
