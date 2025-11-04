/// Rollback module for reverting version bump operations
/// 
/// Provides functionality to safely revert version changes by:
/// - Restoring files from backup (.autoversion.backup)
/// - Deleting git tags
/// - Optionally reverting git commits
/// 
/// # Safety
/// - Verifies backups exist before attempting restore
/// - Atomic operations where possible
/// - Clear error messages for each step

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use crate::constants::BACKUP_FILE_EXTENSION;
use crate::git::operations::{delete_tag_in, revert_last_commit_in, tag_exists_in};

/// Options for rollback operation
#[derive(Debug, Clone)]
pub struct RollbackOptions {
    /// Restore files from backups
    pub restore_files: bool,
    /// Delete git tags
    pub delete_tags: bool,
    /// Revert git commits
    pub revert_commits: bool,
    /// Specific version to rollback (if None, rollback last)
    pub version: Option<String>,
}

impl Default for RollbackOptions {
    fn default() -> Self {
        Self {
            restore_files: true,
            delete_tags: true,
            revert_commits: false, // Conservative default
            version: None,
        }
    }
}

/// Find all backup files in a directory
pub fn find_backup_files(project_path: &Path) -> Result<Vec<PathBuf>> {
    let mut backups = Vec::new();
    
    for entry in fs::read_dir(project_path)
        .context("Failed to read project directory")?
    {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(BACKUP_FILE_EXTENSION) {
                    backups.push(path);
                }
            }
        }
    }
    
    Ok(backups)
}

/// Restore a single file from backup
pub fn restore_from_backup(backup_path: &Path) -> Result<PathBuf> {
    if !backup_path.exists() {
        anyhow::bail!("Backup file does not exist: {}", backup_path.display());
    }
    
    // Determine original file path by removing backup extension
    let original_path = backup_path.to_str()
        .and_then(|s| s.strip_suffix(BACKUP_FILE_EXTENSION))
        .map(PathBuf::from)
        .context("Failed to determine original file path")?;
    
    // Copy backup to original location
    fs::copy(backup_path, &original_path)
        .context(format!("Failed to restore {} from backup", original_path.display()))?;
    
    println!("✅ Restored: {}", original_path.display());
    
    Ok(original_path)
}

/// Delete backup file after successful restore
pub fn delete_backup(backup_path: &Path) -> Result<()> {
    fs::remove_file(backup_path)
        .context(format!("Failed to delete backup: {}", backup_path.display()))?;
    
    println!("🗑️  Deleted backup: {}", backup_path.display());
    
    Ok(())
}

/// Rollback version changes
pub fn rollback(project_path: &Path, options: &RollbackOptions) -> Result<Vec<PathBuf>> {
    let mut restored_files = Vec::new();
    
    // Find backup files
    let backups = find_backup_files(project_path)?;
    
    if backups.is_empty() {
        anyhow::bail!("No backup files found in {}", project_path.display());
    }
    
    println!("📦 Found {} backup file(s)", backups.len());
    
    // Restore files if requested
    if options.restore_files {
        for backup in &backups {
            let restored = restore_from_backup(backup)?;
            restored_files.push(restored);
            
            // Delete backup after successful restore
            delete_backup(backup)?;
        }
    }
    
    // Handle git operations (tags, commits)
    if options.delete_tags {
        if let Some(version) = &options.version {
            let tag_name = format!("v{}", version);
            
            // Check if tag exists before trying to delete
            if tag_exists_in(project_path, &tag_name)? {
                delete_tag_in(project_path, &tag_name)
                    .context("Failed to delete git tag")?;
            } else {
                println!("ℹ️  Tag {} does not exist, skipping deletion", tag_name);
            }
        } else {
            println!("⚠️  No version specified, skipping tag deletion");
        }
    }
    
    if options.revert_commits {
        revert_last_commit_in(project_path)
            .context("Failed to revert last commit")?;
    }
    
    Ok(restored_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file(path: &Path, content: &str) {
        fs::write(path, content).unwrap();
    }

    #[test]
    fn test_find_backup_files_empty_directory() {
        let temp = TempDir::new().unwrap();
        let backups = find_backup_files(temp.path()).unwrap();
        assert_eq!(backups.len(), 0);
    }

    #[test]
    fn test_find_backup_files_single_backup() {
        let temp = TempDir::new().unwrap();
        create_test_file(
            &temp.path().join("package.json.autoversion.backup"),
            r#"{"version": "1.0.0"}"#,
        );

        let backups = find_backup_files(temp.path()).unwrap();
        assert_eq!(backups.len(), 1);
        assert!(backups[0].to_str().unwrap().ends_with(".autoversion.backup"));
    }

    #[test]
    fn test_find_backup_files_multiple_backups() {
        let temp = TempDir::new().unwrap();
        create_test_file(
            &temp.path().join("package.json.autoversion.backup"),
            r#"{"version": "1.0.0"}"#,
        );
        create_test_file(
            &temp.path().join("Cargo.toml.autoversion.backup"),
            "[package]\nversion = \"1.0.0\"",
        );

        let backups = find_backup_files(temp.path()).unwrap();
        assert_eq!(backups.len(), 2);
    }

    #[test]
    fn test_find_backup_files_ignores_regular_files() {
        let temp = TempDir::new().unwrap();
        create_test_file(&temp.path().join("package.json"), r#"{"version": "2.0.0"}"#);
        create_test_file(
            &temp.path().join("package.json.autoversion.backup"),
            r#"{"version": "1.0.0"}"#,
        );

        let backups = find_backup_files(temp.path()).unwrap();
        assert_eq!(backups.len(), 1);
    }

    #[test]
    fn test_restore_from_backup_success() {
        let temp = TempDir::new().unwrap();
        let original = temp.path().join("package.json");
        let backup = temp.path().join("package.json.autoversion.backup");

        // Create original file with new version
        create_test_file(&original, r#"{"version": "2.0.0"}"#);
        // Create backup with old version
        create_test_file(&backup, r#"{"version": "1.0.0"}"#);

        // Restore from backup
        let restored = restore_from_backup(&backup).unwrap();
        
        assert_eq!(restored, original);
        
        // Verify content was restored
        let content = fs::read_to_string(&original).unwrap();
        assert!(content.contains("1.0.0"));
        assert!(!content.contains("2.0.0"));
    }

    #[test]
    fn test_restore_from_backup_nonexistent() {
        let temp = TempDir::new().unwrap();
        let backup = temp.path().join("nonexistent.autoversion.backup");

        let result = restore_from_backup(&backup);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_delete_backup_success() {
        let temp = TempDir::new().unwrap();
        let backup = temp.path().join("package.json.autoversion.backup");
        create_test_file(&backup, "test content");

        assert!(backup.exists());
        
        delete_backup(&backup).unwrap();
        
        assert!(!backup.exists());
    }

    #[test]
    fn test_delete_backup_nonexistent() {
        let temp = TempDir::new().unwrap();
        let backup = temp.path().join("nonexistent.autoversion.backup");

        let result = delete_backup(&backup);
        assert!(result.is_err());
    }

    #[test]
    fn test_rollback_default_options() {
        let temp = TempDir::new().unwrap();
        let original = temp.path().join("VERSION");
        let backup = temp.path().join("VERSION.autoversion.backup");

        create_test_file(&original, "2.0.0");
        create_test_file(&backup, "1.0.0");

        let options = RollbackOptions::default();
        let restored = rollback(temp.path(), &options).unwrap();

        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0], original);
        
        // Backup should be deleted
        assert!(!backup.exists());
        
        // Original should have old content
        let content = fs::read_to_string(&original).unwrap();
        assert!(content.contains("1.0.0"));
    }

    #[test]
    fn test_rollback_no_backups() {
        let temp = TempDir::new().unwrap();
        let options = RollbackOptions::default();

        let result = rollback(temp.path(), &options);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No backup files found"));
    }

    #[test]
    fn test_rollback_multiple_files() {
        let temp = TempDir::new().unwrap();
        
        // Create multiple original files
        create_test_file(&temp.path().join("package.json"), r#"{"version": "2.0.0"}"#);
        create_test_file(&temp.path().join("package-lock.json"), r#"{"version": "2.0.0"}"#);
        
        // Create backups
        create_test_file(
            &temp.path().join("package.json.autoversion.backup"),
            r#"{"version": "1.0.0"}"#,
        );
        create_test_file(
            &temp.path().join("package-lock.json.autoversion.backup"),
            r#"{"version": "1.0.0"}"#,
        );

        let options = RollbackOptions::default();
        let restored = rollback(temp.path(), &options).unwrap();

        assert_eq!(restored.len(), 2);
        
        // Both should be restored
        let content1 = fs::read_to_string(temp.path().join("package.json")).unwrap();
        let content2 = fs::read_to_string(temp.path().join("package-lock.json")).unwrap();
        
        assert!(content1.contains("1.0.0"));
        assert!(content2.contains("1.0.0"));
    }

    #[test]
    fn test_rollback_options_files_only() {
        let temp = TempDir::new().unwrap();
        let original = temp.path().join("VERSION");
        let backup = temp.path().join("VERSION.autoversion.backup");

        create_test_file(&original, "2.0.0");
        create_test_file(&backup, "1.0.0");

        let options = RollbackOptions {
            restore_files: true,
            delete_tags: false,
            revert_commits: false,
            version: None,
        };

        let restored = rollback(temp.path(), &options).unwrap();
        assert_eq!(restored.len(), 1);
    }

    #[test]
    fn test_rollback_with_git_tag_deletion() {
        use crate::git::operations::{init_test_repo, create_tag_in};
        use git2::Signature;
        
        let temp = TempDir::new().unwrap();
        
        // Initialize git repo and create initial commit
        let repo = init_test_repo(temp.path()).unwrap();
        let sig = Signature::now("Test", "test@example.com").unwrap();
        
        // Create initial file and commit
        create_test_file(&temp.path().join("VERSION"), "1.0.0");
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("VERSION")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();
        
        // Update file and create backup
        create_test_file(&temp.path().join("VERSION"), "2.0.0");
        create_test_file(&temp.path().join("VERSION.autoversion.backup"), "1.0.0");
        
        // Create a tag
        create_tag_in(temp.path(), "v2.0.0", "2.0.0").unwrap();
        
        // Rollback with tag deletion
        let options = RollbackOptions {
            restore_files: true,
            delete_tags: true,
            revert_commits: false,
            version: Some("2.0.0".to_string()),
        };
        
        let restored = rollback(temp.path(), &options).unwrap();
        assert_eq!(restored.len(), 1);
        
        // Verify tag was deleted
        assert!(!tag_exists_in(temp.path(), "v2.0.0").unwrap());
    }

    #[test]
    fn test_rollback_with_commit_reversion() {
        use crate::git::operations::{init_test_repo, commit_version_changes_in};
        use git2::Signature;
        
        let temp = TempDir::new().unwrap();
        
        // Initialize git repo and create initial commit
        let repo = init_test_repo(temp.path()).unwrap();
        let sig = Signature::now("Test", "test@example.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();
        
        // Create a file for version bump
        create_test_file(&temp.path().join("VERSION"), "1.0.0");
        create_test_file(&temp.path().join("VERSION.autoversion.backup"), "1.0.0");
        
        // Make a version bump commit
        let version_file = temp.path().join("VERSION").to_string_lossy().to_string();
        commit_version_changes_in(temp.path(), &[version_file], "2.0.0", None).unwrap();
        
        // Now rollback with commit reversion
        let options = RollbackOptions {
            restore_files: false,
            delete_tags: false,
            revert_commits: true,
            version: None,
        };
        
        let result = rollback(temp.path(), &options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rollback_without_version_skips_tag_deletion() {
        let temp = TempDir::new().unwrap();
        
        // Create backup
        create_test_file(&temp.path().join("VERSION"), "2.0.0");
        create_test_file(&temp.path().join("VERSION.autoversion.backup"), "1.0.0");
        
        // Rollback with delete_tags but no version specified
        let options = RollbackOptions {
            restore_files: true,
            delete_tags: true,  // Requested but will be skipped
            revert_commits: false,
            version: None,  // No version specified
        };
        
        let restored = rollback(temp.path(), &options).unwrap();
        assert_eq!(restored.len(), 1);
        // Test passes if no panic occurs (tag deletion skipped gracefully)
    }
}
