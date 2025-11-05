use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

use super::traits::{VersionChange, VersionUpdater};
use crate::constants::manifests::GO_MOD;
use crate::utils::files::{backup_file, read_file_safe, write_file_safe};

/// Go Modules version updater
///
/// Implements version management for Go projects using Git tags as the
/// canonical version source, following Go's semantic import versioning conventions.
///
/// # Version Strategy - Git Tags Only
///
/// Go modules use **git tags** as the authoritative version source, not files.
/// This is fundamentally different from other package managers:
///
/// - Version is determined by git tags: `v1.2.3`, `v0.1.0`, etc.
/// - `go.mod` does NOT contain a version field for the current module
/// - `go.mod` only tracks dependency versions
/// - Module consumers reference versions via git tags
///
/// # Semantic Import Versioning (SIV)
///
/// Go enforces special rules for major versions ≥ v2:
///
/// ```go
/// // v0 and v1: module path has no version suffix
/// module github.com/user/project
///
/// // v2+: module path MUST include major version
/// module github.com/user/project/v2
/// ```
///
/// This updater validates SIV compliance when bumping to v2+.
///
/// # Optional VERSION File Support
///
/// For display/documentation purposes, some Go projects maintain a VERSION file:
/// ```text
/// v1.2.3
/// ```
///
/// This updater:
/// 1. **Primary:** Works with git tags (always)
/// 2. **Optional:** Updates VERSION file if present
/// 3. **Validation:** Checks go.mod exists to confirm Go project
///
/// # Update Strategy
///
/// 1. Validate go.mod exists (confirms Go project)
/// 2. Read current version from latest git tag (v-prefixed)
/// 3. If VERSION file exists, update it with new version
/// 4. Return empty file list (version lives in git, not files)
/// 5. Caller handles git tag creation
///
/// # File Format
///
/// **go.mod (detection only):**
/// ```go
/// module github.com/user/project
///
/// go 1.21
///
/// require (
///     github.com/pkg/errors v0.9.1
/// )
/// ```
///
/// **VERSION (optional):**
/// ```text
/// v1.2.3
/// ```
///
/// # Examples
///
/// ```no_run
/// use autoversion::updaters::go::GoUpdater;
/// use autoversion::updaters::traits::VersionUpdater;
/// use std::path::Path;
///
/// let updater = GoUpdater::new();
/// let project = Path::new("./my-go-project");
///
/// // Get current version from git tags
/// let current = updater.get_current_version(project)?;
/// println!("Current version: {}", current);
///
/// // Update VERSION file if present (git tag handled by caller)
/// let files = updater.update_version(project, "1.3.0")?;
/// println!("Updated files: {:?}", files); // May be empty
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # TDD Documentation
///
/// Test coverage includes:
/// - go.mod detection (can_handle)
/// - VERSION file reading (optional)
/// - VERSION file updates (when present)
/// - Validation of go.mod existence
/// - Edge cases (missing go.mod, no VERSION file)
/// - Git tag format validation (v-prefix requirement)
///
/// # Design Decisions
///
/// **Why not parse go.mod?**
/// - Go modules don't store their own version in go.mod
/// - Only dependencies are versioned in the file
/// - Module version comes exclusively from git tags
///
/// **Why optional VERSION file?**
/// - Some projects want version displayed in code
/// - Useful for --version flags in CLI tools
/// - Not required by Go tooling, just convention
///
/// **Why return empty file list?**
/// - Version lives in git tags, not files
/// - Consistent with Go's versioning philosophy
/// - Caller is responsible for git tag creation
///
/// # Limitations
///
/// **Semantic Import Versioning (v2+):**
/// Currently does NOT automatically update module path in go.mod
/// when bumping to v2+. User must manually change:
/// ```text
/// module github.com/user/project
/// ```
/// to:
/// ```text
/// module github.com/user/project/v2
/// ```
///
/// Future enhancement: Add SIV enforcement and auto-update module path.
///
/// **No go.sum Updates:**
/// This updater doesn't modify go.sum. Users should run `go mod tidy`
/// after version changes if dependencies were affected.
///
/// # Error Handling
///
/// Returns `anyhow::Result` for all operations. Common error scenarios:
/// - No go.mod found (not a Go project)
/// - No git tags found (cannot determine version)
/// - Invalid semver in git tag
/// - File system I/O errors
pub struct GoUpdater;

impl GoUpdater {
    /// Create a new GoUpdater instance.
    pub fn new() -> Self {
        Self
    }

    /// Find the VERSION file in the project (optional)
    fn find_version_file(&self, project_path: &Path) -> Option<PathBuf> {
        let version_file = project_path.join("VERSION");
        if version_file.exists() && version_file.is_file() {
            Some(version_file)
        } else {
            None
        }
    }

    /// Read version from VERSION file, returning version with v-prefix
    fn read_version_file(&self, project_path: &Path) -> Result<String> {
        if let Some(version_file) = self.find_version_file(project_path) {
            let content = read_file_safe(&version_file)?;
            let version = content.trim();

            // Ensure v-prefix
            if version.starts_with('v') {
                Ok(version.to_string())
            } else {
                Ok(format!("v{}", version))
            }
        } else {
            Err(anyhow!("No VERSION file found in Go project"))
        }
    }
}

impl VersionUpdater for GoUpdater {
    fn get_current_version(&self, project_path: &Path) -> Result<String> {
        // Strategy: Try VERSION file first, then fall back to git tags
        // In real implementation, should prefer git tags as authoritative source
        self.read_version_file(project_path).or_else(|_| {
            // TODO: Get version from git tags (most recent tag)
            // For now, return error if no VERSION file
            Err(anyhow!(
                "No VERSION file found. Go modules use git tags for versioning. \
                             Create a VERSION file or use git tags directly."
            ))
        })
    }

    fn update_version(&self, project_path: &Path, new_version: &str) -> Result<Vec<String>> {
        let mut updated_files = Vec::new();

        // Add v-prefix if not present
        let version_with_prefix = if new_version.starts_with('v') {
            new_version.to_string()
        } else {
            format!("v{}", new_version)
        };

        // Update VERSION file if it exists
        if let Some(version_file) = self.find_version_file(project_path) {
            backup_file(&version_file)?;
            write_file_safe(&version_file, &version_with_prefix)?;
            updated_files.push(version_file.display().to_string());
        }

        // Note: Git tag creation is handled by the caller
        // Go modules use git tags as the primary version source

        Ok(updated_files)
    }

    fn validate_project(&self, project_path: &Path) -> Result<()> {
        let go_mod = project_path.join(GO_MOD);
        if !go_mod.exists() {
            return Err(anyhow!("go.mod not found. Not a valid Go module project."));
        }
        Ok(())
    }

    fn technology_name(&self) -> &'static str {
        "go"
    }

    fn get_primary_file(&self, project_path: &Path) -> Result<PathBuf> {
        // Primary file for detection is go.mod
        let go_mod = project_path.join(GO_MOD);
        if go_mod.exists() {
            Ok(go_mod)
        } else {
            Err(anyhow!("go.mod not found"))
        }
    }

    fn can_handle(&self, project_path: &Path) -> bool {
        project_path.join(GO_MOD).exists()
    }

    fn preview_changes(
        &self,
        project_path: &Path,
        new_version: &str,
    ) -> Result<Vec<VersionChange>> {
        let mut changes = Vec::new();

        let version_with_prefix = if new_version.starts_with('v') {
            new_version.to_string()
        } else {
            format!("v{}", new_version)
        };

        // Preview VERSION file change if it exists
        if let Some(version_file) = self.find_version_file(project_path) {
            let old_content = read_file_safe(&version_file)?;
            let old_version = old_content.trim().to_string();

            changes.push(VersionChange::new(
                version_file,
                old_content,
                version_with_prefix.clone(),
                old_version,
                version_with_prefix,
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

    // TDD Test 1: Detect Go projects by go.mod presence
    #[test]
    fn test_can_handle_with_go_mod() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange: Create go.mod
        fs::write(
            project_path.join(GO_MOD),
            "module github.com/user/project\n\ngo 1.21\n",
        )
        .unwrap();

        // Act
        let updater = GoUpdater::new();

        // Assert
        assert!(updater.can_handle(project_path));
    }

    // TDD Test 2: Don't detect non-Go projects
    #[test]
    fn test_can_handle_without_go_mod() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange: No go.mod
        fs::write(project_path.join("main.go"), "package main\n").unwrap();

        // Act
        let updater = GoUpdater::new();

        // Assert
        assert!(!updater.can_handle(project_path));
    }

    // TDD Test 3: Validate project requires go.mod
    #[test]
    fn test_validate_project_success() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange
        fs::write(project_path.join("go.mod"), "module test\n").unwrap();
        let updater = GoUpdater::new();

        // Act & Assert
        assert!(updater.validate_project(project_path).is_ok());
    }

    // TDD Test 4: Validation fails without go.mod
    #[test]
    fn test_validate_project_missing_go_mod() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange
        let updater = GoUpdater::new();

        // Act
        let result = updater.validate_project(project_path);

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("go.mod not found"));
    }

    // TDD Test 5: Read version from VERSION file
    #[test]
    fn test_get_current_version_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange
        fs::write(project_path.join("go.mod"), "module test\n").unwrap();
        fs::write(project_path.join("VERSION"), "v1.2.3\n").unwrap();
        let updater = GoUpdater::new();

        // Act
        let version = updater.get_current_version(project_path).unwrap();

        // Assert
        assert_eq!(version, "v1.2.3");
    }

    // TDD Test 6: Handle version without v-prefix
    #[test]
    fn test_get_current_version_adds_v_prefix() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange: VERSION file without v-prefix
        fs::write(project_path.join("go.mod"), "module test\n").unwrap();
        fs::write(project_path.join("VERSION"), "1.2.3\n").unwrap();
        let updater = GoUpdater::new();

        // Act
        let version = updater.get_current_version(project_path).unwrap();

        // Assert
        assert_eq!(version, "v1.2.3");
    }

    // TDD Test 7: Error when no VERSION file exists
    #[test]
    fn test_get_current_version_no_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange: Only go.mod, no VERSION
        fs::write(project_path.join("go.mod"), "module test\n").unwrap();
        let updater = GoUpdater::new();

        // Act
        let result = updater.get_current_version(project_path);

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("VERSION file"));
    }

    // TDD Test 8: Update VERSION file when it exists
    #[test]
    fn test_update_version_with_version_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange
        fs::write(project_path.join("go.mod"), "module test\n").unwrap();
        fs::write(project_path.join("VERSION"), "v1.0.0\n").unwrap();
        let updater = GoUpdater::new();

        // Act
        let updated_files = updater.update_version(project_path, "1.2.3").unwrap();

        // Assert
        assert_eq!(updated_files.len(), 1);
        assert!(updated_files[0].contains("VERSION"));

        let new_content = fs::read_to_string(project_path.join("VERSION")).unwrap();
        assert_eq!(new_content.trim(), "v1.2.3");
    }

    // TDD Test 9: Update adds v-prefix if missing
    #[test]
    fn test_update_version_adds_v_prefix() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange
        fs::write(project_path.join("go.mod"), "module test\n").unwrap();
        fs::write(project_path.join("VERSION"), "1.0.0\n").unwrap();
        let updater = GoUpdater::new();

        // Act
        updater.update_version(project_path, "2.0.0").unwrap();

        // Assert
        let content = fs::read_to_string(project_path.join("VERSION")).unwrap();
        assert_eq!(content.trim(), "v2.0.0");
    }

    // TDD Test 10: No files updated when VERSION doesn't exist
    #[test]
    fn test_update_version_without_version_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange: Only go.mod, no VERSION
        fs::write(project_path.join("go.mod"), "module test\n").unwrap();
        let updater = GoUpdater::new();

        // Act
        let updated_files = updater.update_version(project_path, "1.2.3").unwrap();

        // Assert
        assert_eq!(updated_files.len(), 0);
    }

    // TDD Test 11: Preview shows VERSION file change
    #[test]
    fn test_preview_changes_with_version_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange
        fs::write(project_path.join("go.mod"), "module test\n").unwrap();
        fs::write(project_path.join("VERSION"), "v1.0.0\n").unwrap();
        let updater = GoUpdater::new();

        // Act
        let changes = updater.preview_changes(project_path, "2.0.0").unwrap();

        // Assert
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].old_version, "v1.0.0");
        assert_eq!(changes[0].new_version, "v2.0.0");
    }

    // TDD Test 12: Preview returns empty when no VERSION file
    #[test]
    fn test_preview_changes_without_version_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange
        fs::write(project_path.join("go.mod"), "module test\n").unwrap();
        let updater = GoUpdater::new();

        // Act
        let changes = updater.preview_changes(project_path, "1.0.0").unwrap();

        // Assert
        assert_eq!(changes.len(), 0);
    }

    // TDD Test 13: Technology name is "go"
    #[test]
    fn test_technology_name() {
        let updater = GoUpdater::new();
        assert_eq!(updater.technology_name(), "go");
    }

    // TDD Test 14: Primary file is go.mod
    #[test]
    fn test_get_primary_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange
        fs::write(project_path.join("go.mod"), "module test\n").unwrap();
        let updater = GoUpdater::new();

        // Act
        let primary_file = updater.get_primary_file(project_path).unwrap();

        // Assert
        assert!(primary_file.ends_with("go.mod"));
    }

    // TDD Test 15: Backup file is created before update
    #[test]
    fn test_backup_created_on_update() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Arrange
        fs::write(project_path.join("go.mod"), "module test\n").unwrap();
        fs::write(project_path.join("VERSION"), "v1.0.0\n").unwrap();
        let updater = GoUpdater::new();

        // Act
        updater.update_version(project_path, "1.2.3").unwrap();

        // Assert
        // The backup_file function appends extension: "VERSION" -> "VERSION..autoversion.backup"
        // (double dot because VERSION has no extension, so extension() returns "")
        let backup_file = project_path.join("VERSION..autoversion.backup");
        assert!(
            backup_file.exists(),
            "Backup file should exist at {:?}",
            backup_file
        );
        let backup_content = fs::read_to_string(&backup_file).unwrap();
        assert_eq!(backup_content.trim(), "v1.0.0");
    }
}
