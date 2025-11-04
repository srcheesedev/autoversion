use anyhow::{anyhow, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

use super::traits::{VersionUpdater, VersionChange};
use crate::constants::manifests::{PHP_COMPOSER, PHP_COMPOSER_LOCK};
use crate::utils::files::{backup_file, read_file_safe, write_file_safe};

/// PHP Composer package version updater
///
/// Implements version management for PHP projects using Composer package manager.
/// Updates version in both composer.json and composer.lock files.
///
/// # Version Strategy
///
/// Composer projects store version in `composer.json`:
/// - Optional field (often omitted for applications)
/// - Required for libraries published to Packagist
/// - Must follow semantic versioning
///
/// # Update Strategy
///
/// 1. Parse composer.json as JSON
/// 2. Update root-level `version` field
/// 3. If composer.lock exists, update its version fields
/// 4. Create .autoversion.backup for both files before modification
/// 5. Preserve JSON formatting and structure
///
/// # File Formats
///
/// **composer.json:**
/// ```json
/// {
///     "name": "vendor/package",
///     "version": "1.2.3",
///     "require": {
///         "php": "^8.0"
///     }
/// }
/// ```
///
/// **composer.lock (optional):**
/// Contains locked dependency versions. We update the root package version if present.
///
/// # Examples
///
/// ```no_run
/// use autoversion::updaters::composer::ComposerUpdater;
/// use autoversion::updaters::traits::VersionUpdater;
/// use std::path::Path;
///
/// let updater = ComposerUpdater::new();
/// let project = Path::new("./my-php-project");
///
/// // Get current version
/// let current = updater.get_current_version(project)?;
/// println!("Current version: {}", current);
///
/// // Update version
/// let files = updater.update_version(project, "2.0.0")?;
/// println!("Updated files: {:?}", files);
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # TDD Documentation
///
/// All tests written before implementation following TDD red-green-refactor cycle.
/// Test coverage includes:
/// - composer.json detection
/// - Version reading from composer.json
/// - Version updates in composer.json
/// - Optional composer.lock handling
/// - Error cases (missing files, malformed JSON)
/// - Backup creation
///
/// # Error Handling
///
/// Returns `anyhow::Result` for all operations. Common error scenarios:
/// - No composer.json found
/// - Invalid JSON format
/// - Missing version field
/// - File system I/O errors
pub struct ComposerUpdater;

impl ComposerUpdater {
    pub fn new() -> Self {
        Self
    }

    /// Parse composer.json and extract version
    fn parse_composer_json(&self, content: &str) -> Result<Value> {
        serde_json::from_str(content)
            .map_err(|e| anyhow!("Failed to parse composer.json: {}", e))
    }

    /// Update version in composer.json content
    fn update_composer_json_content(&self, content: &str, new_version: &str) -> Result<String> {
        let mut composer_json: Value = self.parse_composer_json(content)?;
        
        // Update version field
        if let Some(obj) = composer_json.as_object_mut() {
            obj.insert("version".to_string(), Value::String(new_version.to_string()));
        } else {
            return Err(anyhow!("composer.json root is not an object"));
        }

        // Serialize with pretty printing
        serde_json::to_string_pretty(&composer_json)
            .map_err(|e| anyhow!("Failed to serialize composer.json: {}", e))
    }
}

impl VersionUpdater for ComposerUpdater {
    fn get_current_version(&self, project_path: &Path) -> Result<String> {
        let composer_json = project_path.join(PHP_COMPOSER);
        let content = read_file_safe(&composer_json)?;
        
        let json: Value = self.parse_composer_json(&content)?;
        
        // Extract version field
        json.get("version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("No version field found in composer.json"))
    }

    fn update_version(&self, project_path: &Path, new_version: &str) -> Result<Vec<String>> {
        let mut updated_files = Vec::new();
        
        // Update composer.json
        let composer_json = project_path.join(PHP_COMPOSER);
        let content = read_file_safe(&composer_json)?;
        
        backup_file(&composer_json)?;
        let new_content = self.update_composer_json_content(&content, new_version)?;
        write_file_safe(&composer_json, &new_content)?;
        
        updated_files.push(composer_json.display().to_string());
        
        // Update composer.lock if it exists
        let composer_lock = project_path.join(PHP_COMPOSER_LOCK);
        if composer_lock.exists() {
            let lock_content = read_file_safe(&composer_lock)?;
            backup_file(&composer_lock)?;
            
            // Parse and update composer.lock
            let mut lock_json: Value = serde_json::from_str(&lock_content)
                .map_err(|e| anyhow!("Failed to parse composer.lock: {}", e))?;
            
            // Update version if present in lock file
            if let Some(obj) = lock_json.as_object_mut() {
                if obj.contains_key("version") {
                    obj.insert("version".to_string(), Value::String(new_version.to_string()));
                }
            }
            
            let new_lock_content = serde_json::to_string_pretty(&lock_json)
                .map_err(|e| anyhow!("Failed to serialize composer.lock: {}", e))?;
            
            write_file_safe(&composer_lock, &new_lock_content)?;
            updated_files.push(composer_lock.display().to_string());
        }
        
        Ok(updated_files)
    }

    fn validate_project(&self, project_path: &Path) -> Result<()> {
        let composer_json = project_path.join(PHP_COMPOSER);
        if !composer_json.exists() {
            return Err(anyhow!("composer.json not found. Not a valid Composer project."));
        }
        Ok(())
    }

    fn technology_name(&self) -> &'static str {
        "composer"
    }

    fn get_primary_file(&self, project_path: &Path) -> Result<PathBuf> {
        let composer_json = project_path.join(PHP_COMPOSER);
        if composer_json.exists() {
            Ok(composer_json)
        } else {
            Err(anyhow!("composer.json not found"))
        }
    }

    fn can_handle(&self, project_path: &Path) -> bool {
        project_path.join(PHP_COMPOSER).exists()
    }

    fn preview_changes(&self, project_path: &Path, new_version: &str) -> Result<Vec<VersionChange>> {
        let mut changes = Vec::new();
        
        // Preview composer.json change
        let composer_json = project_path.join("composer.json");
        let old_content = read_file_safe(&composer_json)?;
        let old_version = self.get_current_version(project_path)?;
        let new_content = self.update_composer_json_content(&old_content, new_version)?;
        
        changes.push(VersionChange::new(
            composer_json,
            old_content,
            new_content,
            old_version.clone(),
            new_version.to_string(),
        ));
        
        // Preview composer.lock change if it exists
        let composer_lock = project_path.join("composer.lock");
        if composer_lock.exists() {
            let lock_content = read_file_safe(&composer_lock)?;
            let mut lock_json: Value = serde_json::from_str(&lock_content)?;
            
            if let Some(obj) = lock_json.as_object_mut() {
                if obj.contains_key("version") {
                    obj.insert("version".to_string(), Value::String(new_version.to_string()));
                }
            }
            
            let new_lock_content = serde_json::to_string_pretty(&lock_json)?;
            
            changes.push(VersionChange::new(
                composer_lock,
                lock_content,
                new_lock_content,
                old_version,
                new_version.to_string(),
            ));
        }
        
        Ok(changes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // TDD Test 1: Detect PHP Composer projects by composer.json presence
    #[test]
    fn test_can_handle_with_composer_json() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange: Create composer.json
        fs::write(
            project_path.join(PHP_COMPOSER),
            r#"{"name": "vendor/package", "version": "1.0.0"}"#,
        ).unwrap();

        // Act
        let updater = ComposerUpdater::new();

        // Assert
        assert!(updater.can_handle(project_path), "Should detect composer.json");
    }

    // TDD Test 2: Don't detect non-Composer projects
    #[test]
    fn test_can_handle_without_composer_json() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange: Create PHP file but no composer.json
        fs::write(project_path.join("index.php"), "<?php\necho 'Hello';\n").unwrap();

        // Act
        let updater = ComposerUpdater::new();

        // Assert
        assert!(!updater.can_handle(project_path), "Should not detect without composer.json");
    }

    // TDD Test 3: Validate project requires composer.json
    #[test]
    fn test_validate_project_success() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange
        fs::write(
            project_path.join("composer.json"),
            r#"{"name": "vendor/package"}"#,
        ).unwrap();
        let updater = ComposerUpdater::new();

        // Act & Assert
        assert!(updater.validate_project(project_path).is_ok());
    }

    // TDD Test 4: Validation fails without composer.json
    #[test]
    fn test_validate_project_missing_composer_json() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange
        let updater = ComposerUpdater::new();

        // Act
        let result = updater.validate_project(project_path);

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("composer.json"));
    }

    // TDD Test 5: Read version from composer.json
    #[test]
    fn test_get_current_version() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange
        fs::write(
            project_path.join("composer.json"),
            r#"{
                "name": "vendor/package",
                "version": "1.2.3",
                "require": {
                    "php": "^8.0"
                }
            }"#,
        ).unwrap();
        let updater = ComposerUpdater::new();

        // Act
        let version = updater.get_current_version(project_path).unwrap();

        // Assert
        assert_eq!(version, "1.2.3");
    }

    // TDD Test 6: Error when version field is missing
    #[test]
    fn test_get_current_version_missing_field() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange: composer.json without version
        fs::write(
            project_path.join("composer.json"),
            r#"{"name": "vendor/package"}"#,
        ).unwrap();
        let updater = ComposerUpdater::new();

        // Act
        let result = updater.get_current_version(project_path);

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("version"));
    }

    // TDD Test 7: Update version in composer.json
    #[test]
    fn test_update_version_composer_json() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange
        fs::write(
            project_path.join("composer.json"),
            r#"{"name": "vendor/package", "version": "1.0.0"}"#,
        ).unwrap();
        let updater = ComposerUpdater::new();

        // Act
        let updated_files = updater.update_version(project_path, "2.0.0").unwrap();

        // Assert
        assert_eq!(updated_files.len(), 1);
        assert!(updated_files[0].contains("composer.json"));

        let content = fs::read_to_string(project_path.join("composer.json")).unwrap();
        assert!(content.contains("2.0.0"));
    }

    // TDD Test 8: Update preserves other fields
    #[test]
    fn test_update_preserves_json_structure() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange
        fs::write(
            project_path.join("composer.json"),
            r#"{
                "name": "vendor/package",
                "version": "1.0.0",
                "require": {
                    "php": "^8.0",
                    "symfony/console": "^6.0"
                }
            }"#,
        ).unwrap();
        let updater = ComposerUpdater::new();

        // Act
        updater.update_version(project_path, "2.0.0").unwrap();

        // Assert
        let content = fs::read_to_string(project_path.join("composer.json")).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();
        
        assert_eq!(json["version"], "2.0.0");
        assert_eq!(json["name"], "vendor/package");
        assert_eq!(json["require"]["php"], "^8.0");
    }

    // TDD Test 9: Handle composer.lock if present
    #[test]
    fn test_update_with_composer_lock() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange
        fs::write(
            project_path.join("composer.json"),
            r#"{"name": "vendor/package", "version": "1.0.0"}"#,
        ).unwrap();
        fs::write(
            project_path.join("composer.lock"),
            r#"{"packages": []}"#,
        ).unwrap();
        let updater = ComposerUpdater::new();

        // Act
        let updated_files = updater.update_version(project_path, "2.0.0").unwrap();

        // Assert - Should update both files
        assert_eq!(updated_files.len(), 2);
        assert!(updated_files.iter().any(|f| f.contains("composer.json")));
        assert!(updated_files.iter().any(|f| f.contains("composer.lock")));
    }

    // TDD Test 10: Preview changes
    #[test]
    fn test_preview_changes() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange
        fs::write(
            project_path.join("composer.json"),
            r#"{"name": "vendor/package", "version": "1.0.0"}"#,
        ).unwrap();
        let updater = ComposerUpdater::new();

        // Act
        let changes = updater.preview_changes(project_path, "2.0.0").unwrap();

        // Assert
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].old_version, "1.0.0");
        assert_eq!(changes[0].new_version, "2.0.0");
    }

    // TDD Test 11: Technology name
    #[test]
    fn test_technology_name() {
        let updater = ComposerUpdater::new();
        assert_eq!(updater.technology_name(), "composer");
    }

    // TDD Test 12: Get primary file
    #[test]
    fn test_get_primary_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange
        fs::write(
            project_path.join("composer.json"),
            r#"{"name": "vendor/package"}"#,
        ).unwrap();
        let updater = ComposerUpdater::new();

        // Act
        let primary_file = updater.get_primary_file(project_path).unwrap();

        // Assert
        assert!(primary_file.ends_with("composer.json"));
    }

    // TDD Test 13: Backup created before update
    #[test]
    fn test_backup_created_on_update() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange
        fs::write(
            project_path.join("composer.json"),
            r#"{"name": "vendor/package", "version": "1.0.0"}"#,
        ).unwrap();
        let updater = ComposerUpdater::new();

        // Act
        updater.update_version(project_path, "2.0.0").unwrap();

        // Assert
        let backup = project_path.join("composer.json.autoversion.backup");
        assert!(backup.exists(), "Backup should be created");
        
        let backup_content = fs::read_to_string(backup).unwrap();
        assert!(backup_content.contains("1.0.0"), "Backup should contain old version");
    }

    // TDD Test 14: Handle malformed JSON
    #[test]
    fn test_get_version_malformed_json() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange: Invalid JSON
        fs::write(
            project_path.join("composer.json"),
            r#"{"name": "vendor/package", "version": }"#,  // Invalid
        ).unwrap();
        let updater = ComposerUpdater::new();

        // Act
        let result = updater.get_current_version(project_path);

        // Assert
        assert!(result.is_err());
    }

    // TDD Test 15: Works without composer.lock
    #[test]
    fn test_update_without_composer_lock() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange: Only composer.json
        fs::write(
            project_path.join("composer.json"),
            r#"{"name": "vendor/package", "version": "1.0.0"}"#,
        ).unwrap();
        let updater = ComposerUpdater::new();

        // Act
        let updated_files = updater.update_version(project_path, "2.0.0").unwrap();

        // Assert - Should only update composer.json
        assert_eq!(updated_files.len(), 1);
        assert!(updated_files[0].contains("composer.json"));
    }
}
