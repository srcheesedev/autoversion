use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

/// Create a backup of a file before modifying it
pub fn backup_file(file_path: &Path) -> Result<()> {
    if !file_path.exists() {
        return Ok(()); // Nothing to backup
    }

    let backup_path = file_path.with_extension(format!(
        "{}.autoversion.backup",
        file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
    ));

    fs::copy(file_path, &backup_path)
        .map_err(|e| anyhow!("Failed to create backup of {}: {}", file_path.display(), e))?;

    println!("📋 Created backup: {}", backup_path.display());
    Ok(())
}

/// Restore a file from its backup
pub fn restore_from_backup(file_path: &Path) -> Result<()> {
    let backup_path = file_path.with_extension(format!(
        "{}.autoversion.backup",
        file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
    ));

    if !backup_path.exists() {
        return Err(anyhow!("No backup found for {}", file_path.display()));
    }

    fs::copy(&backup_path, file_path)
        .map_err(|e| anyhow!("Failed to restore from backup: {}", e))?;

    println!("🔄 Restored from backup: {}", backup_path.display());
    Ok(())
}

/// Clean up backup files
pub fn cleanup_backups(project_path: &Path) -> Result<()> {
    let entries = fs::read_dir(project_path)
        .map_err(|e| anyhow!("Failed to read project directory: {}", e))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.contains(".autoversion.backup") {
                fs::remove_file(&path).map_err(|e| {
                    anyhow!("Failed to remove backup file {}: {}", path.display(), e)
                })?;
                println!("🗑️  Removed backup: {}", path.display());
            }
        }
    }

    Ok(())
}

/// Check if a file is writable
pub fn is_writable(file_path: &Path) -> bool {
    if !file_path.exists() {
        // Check if parent directory is writable
        if let Some(parent) = file_path.parent() {
            return parent.exists()
                && fs::metadata(parent)
                    .map(|m| !m.permissions().readonly())
                    .unwrap_or(false);
        }
        return false;
    }

    fs::metadata(file_path)
        .map(|metadata| !metadata.permissions().readonly())
        .unwrap_or(false)
}

/// Ensure a file has proper line endings for the platform
pub fn normalize_line_endings(content: &str) -> String {
    #[cfg(windows)]
    {
        // First normalize to Unix, then convert to Windows to avoid double \r
        content.replace("\r\n", "\n").replace('\n', "\r\n")
    }
    #[cfg(not(windows))]
    {
        content.replace("\r\n", "\n")
    }
}

/// Read file content with error context
pub fn read_file_safe(file_path: &Path) -> Result<String> {
    fs::read_to_string(file_path)
        .map_err(|e| anyhow!("Failed to read {}: {}", file_path.display(), e))
}

/// Write file content with error context and proper permissions
pub fn write_file_safe(file_path: &Path, content: &str) -> Result<()> {
    let normalized_content = normalize_line_endings(content);

    fs::write(file_path, normalized_content)
        .map_err(|e| anyhow!("Failed to write {}: {}", file_path.display(), e))
}

/// Get file size in bytes
pub fn get_file_size(file_path: &Path) -> Result<u64> {
    fs::metadata(file_path)
        .map(|metadata| metadata.len())
        .map_err(|e| anyhow!("Failed to get size of {}: {}", file_path.display(), e))
}

/// Check if file exists and is readable
pub fn is_readable(file_path: &Path) -> bool {
    file_path.exists() && file_path.is_file() && read_file_safe(file_path).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_backup_and_restore() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");
        let original_content = "original content";

        // Create original file
        fs::write(&file_path, original_content)?;

        // Create backup
        backup_file(&file_path)?;

        // Modify original
        fs::write(&file_path, "modified content")?;

        // Restore from backup
        restore_from_backup(&file_path)?;

        // Check content is restored
        let restored_content = fs::read_to_string(&file_path)?;
        assert_eq!(restored_content, original_content);

        Ok(())
    }

    #[test]
    fn test_backup_nonexistent_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("nonexistent.txt");

        // Should not error when backing up non-existent file
        backup_file(&file_path)?;

        Ok(())
    }

    #[test]
    fn test_cleanup_backups() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();

        // Create some files and backups
        fs::write(project_path.join("file1.txt"), "content")?;
        fs::write(project_path.join("file1.txt.autoversion.backup"), "backup")?;
        fs::write(project_path.join("file2.json"), "content")?;
        fs::write(project_path.join("file2.json.autoversion.backup"), "backup")?;
        fs::write(project_path.join("normal_file.txt"), "content")?;

        // Clean up backups
        cleanup_backups(project_path)?;

        // Check that backups are removed but normal files remain
        assert!(project_path.join("file1.txt").exists());
        assert!(project_path.join("file2.json").exists());
        assert!(project_path.join("normal_file.txt").exists());
        assert!(!project_path.join("file1.txt.autoversion.backup").exists());
        assert!(!project_path.join("file2.json.autoversion.backup").exists());

        Ok(())
    }

    #[test]
    fn test_is_writable() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");

        // Non-existent file in writable directory
        assert!(is_writable(&file_path));

        // Create file
        fs::write(&file_path, "content")?;
        assert!(is_writable(&file_path));

        Ok(())
    }

    #[test]
    fn test_normalize_line_endings() {
        let content_unix = "line1\nline2\nline3";
        let content_windows = "line1\r\nline2\r\nline3";

        #[cfg(windows)]
        {
            assert_eq!(normalize_line_endings(content_unix), content_windows);
            assert_eq!(normalize_line_endings(content_windows), content_windows);
        }

        #[cfg(not(windows))]
        {
            assert_eq!(normalize_line_endings(content_windows), content_unix);
            assert_eq!(normalize_line_endings(content_unix), content_unix);
        }
    }

    #[test]
    fn test_read_write_safe() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");
        let content = "test content\nwith newlines";

        write_file_safe(&file_path, content)?;
        let read_content = read_file_safe(&file_path)?;

        // Content should be preserved (with potential line ending normalization)
        assert!(read_content.contains("test content"));
        assert!(read_content.contains("with newlines"));

        Ok(())
    }

    #[test]
    fn test_get_file_size() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");
        let content = "hello world";

        fs::write(&file_path, content)?;
        let size = get_file_size(&file_path)?;

        assert_eq!(size as usize, content.len());

        Ok(())
    }

    #[test]
    fn test_is_readable() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");

        // Non-existent file
        assert!(!is_readable(&file_path));

        // Create readable file
        fs::write(&file_path, "content")?;
        assert!(is_readable(&file_path));

        // Directory is not readable as file
        assert!(!is_readable(temp_dir.path()));

        Ok(())
    }
}
