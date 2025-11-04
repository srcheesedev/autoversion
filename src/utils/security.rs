//! Security utilities for path validation and sanitization
//!
//! This module provides security-focused utilities to prevent common vulnerabilities:
//! - Path traversal attacks
//! - Symbolic link exploitation
//! - Invalid file path injection
//! - Directory traversal via ".." sequences
//!
//! ## Design Principles
//!
//! Following defense-in-depth security:
//! 1. Validate all user-provided paths
//! 2. Canonicalize paths to resolve symlinks and ".." sequences
//! 3. Ensure paths stay within expected boundaries
//! 4. Fail securely with clear error messages
//!
//! ## Usage
//!
//! ```rust
//! use autoversion::utils::security::validate_project_path;
//! # use std::path::Path;
//!
//! # fn example() -> anyhow::Result<()> {
//! let user_path = Path::new("../../../etc/passwd");
//! 
//! // This will fail with appropriate error
//! let safe_path = validate_project_path(user_path)?;
//! # Ok(())
//! # }
//! ```

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

/// Validates that a project path is safe to use
///
/// Performs the following checks:
/// 1. Path must exist
/// 2. Path must be a directory (not a file)
/// 3. Path must be canonical (no symlinks, no "..")
/// 4. Path must be absolute
///
/// # Security
///
/// This function prevents path traversal attacks by ensuring the path
/// doesn't contain ".." sequences and is fully resolved.
///
/// # Arguments
///
/// * `path` - The project path to validate
///
/// # Returns
///
/// * `Ok(PathBuf)` - Canonicalized, validated path
/// * `Err(_)` - If path is invalid or unsafe
///
/// # Examples
///
/// ```rust
/// use autoversion::utils::security::validate_project_path;
/// use std::path::Path;
/// # use std::fs;
/// # use tempfile::TempDir;
///
/// # fn example() -> anyhow::Result<()> {
/// # let temp = TempDir::new()?;
/// # let project_path = temp.path();
/// let safe_path = validate_project_path(project_path)?;
/// assert!(safe_path.is_absolute());
/// # Ok(())
/// # }
/// ```
pub fn validate_project_path(path: &Path) -> Result<PathBuf> {
    // Check if path exists
    if !path.exists() {
        return Err(anyhow!(
            "Project path does not exist: {}",
            path.display()
        ));
    }

    // Check if path is a directory
    if !path.is_dir() {
        return Err(anyhow!(
            "Project path must be a directory, not a file: {}",
            path.display()
        ));
    }

    // Canonicalize to resolve symlinks and ".." sequences
    let canonical_path = path.canonicalize().map_err(|e| {
        anyhow!(
            "Failed to canonicalize path {}: {}",
            path.display(),
            e
        )
    })?;

    // Ensure path is absolute (canonicalize should guarantee this, but double-check)
    if !canonical_path.is_absolute() {
        return Err(anyhow!(
            "Path must be absolute: {}",
            canonical_path.display()
        ));
    }

    Ok(canonical_path)
}

/// Validates that a file path is safe and within the project directory
///
/// Ensures the file path:
/// 1. Is within the specified project directory
/// 2. Doesn't contain path traversal sequences
/// 3. Has a valid filename
///
/// # Security
///
/// This prevents writing files outside the project directory through
/// malicious relative paths like "../../../etc/passwd".
///
/// # Arguments
///
/// * `file_path` - The file path to validate
/// * `project_path` - The project root directory
///
/// # Returns
///
/// * `Ok(PathBuf)` - Validated file path
/// * `Err(_)` - If file path is outside project or invalid
pub fn validate_file_path(file_path: &Path, project_path: &Path) -> Result<PathBuf> {
    // Always canonicalize project path to handle symlinks (e.g., macOS /var vs /private/var)
    let canonical_project = project_path.canonicalize().map_err(|e| {
        anyhow!("Failed to canonicalize project path {}: {}", project_path.display(), e)
    })?;

    let abs_file = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        canonical_project.join(file_path)
    };

    // Canonicalize to resolve any ".." or symlinks
    let canonical_file = if abs_file.exists() {
        abs_file.canonicalize()?
    } else {
        // If file doesn't exist yet, validate the parent directory
        if let Some(parent) = abs_file.parent() {
            if parent.exists() {
                let canonical_parent = parent.canonicalize()?;
                if let Some(filename) = abs_file.file_name() {
                    canonical_parent.join(filename)
                } else {
                    return Err(anyhow!("Invalid file path: missing filename"));
                }
            } else {
                return Err(anyhow!(
                    "Parent directory does not exist: {}",
                    parent.display()
                ));
            }
        } else {
            return Err(anyhow!("Invalid file path: no parent directory"));
        }
    };

    // Ensure the file is within the project directory
    if !canonical_file.starts_with(&canonical_project) {
        return Err(anyhow!(
            "File path {} is outside project directory {}",
            canonical_file.display(),
            canonical_project.display()
        ));
    }

    Ok(canonical_file)
}

/// Sanitizes a filename to prevent injection attacks
///
/// Removes or replaces characters that could be used for:
/// - Command injection
/// - Path traversal
/// - Shell metacharacters
///
/// # Security
///
/// This is a defense-in-depth measure. Proper path validation
/// should be the primary security control.
///
/// # Arguments
///
/// * `filename` - The filename to sanitize
///
/// # Returns
///
/// Sanitized filename safe for filesystem operations
pub fn sanitize_filename(filename: &str) -> String {
    let sanitized: String = filename
        .chars()
        .filter(|c| {
            // Allow alphanumeric, dash, underscore, dot
            c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.'
        })
        .collect();
    
    // Remove leading dots to prevent hidden files and path traversal
    sanitized.trim_start_matches('.').to_string()
}

/// Checks if a path contains any suspicious patterns
///
/// Detects common attack patterns:
/// - Path traversal attempts ("../", "..")
/// - Null bytes
/// - Control characters
///
/// # Arguments
///
/// * `path` - The path to check
///
/// # Returns
///
/// * `true` if path contains suspicious patterns
/// * `false` if path appears safe
pub fn contains_suspicious_patterns(path: &Path) -> bool {
    let path_str = path.to_string_lossy();

    // Check for path traversal
    if path_str.contains("..") {
        return true;
    }

    // Check for null bytes (can terminate strings in C-based APIs)
    if path_str.contains('\0') {
        return true;
    }

    // Check for control characters
    if path_str.chars().any(|c| c.is_control() && c != '\n' && c != '\r') {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_project_path_success() -> Result<()> {
        let temp = TempDir::new()?;
        let path = temp.path();

        let validated = validate_project_path(path)?;
        assert!(validated.is_absolute());
        assert!(validated.is_dir());

        Ok(())
    }

    #[test]
    fn test_validate_project_path_nonexistent() {
        let result = validate_project_path(Path::new("/nonexistent/path"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_validate_project_path_file_not_dir() -> Result<()> {
        let temp = TempDir::new()?;
        let file_path = temp.path().join("test.txt");
        fs::write(&file_path, "test")?;

        let result = validate_project_path(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be a directory"));

        Ok(())
    }

    #[test]
    fn test_validate_file_path_success() -> Result<()> {
        let temp = TempDir::new()?;
        let project = temp.path();
        let file = project.join("test.txt");
        fs::write(&file, "test")?;

        let validated = validate_file_path(&file, project)?;
        
        // Canonicalize both paths for comparison (handles macOS /var vs /private/var)
        let canonical_project = project.canonicalize()?;
        assert!(validated.starts_with(&canonical_project));

        Ok(())
    }

    #[test]
    fn test_validate_file_path_outside_project() -> Result<()> {
        let temp1 = TempDir::new()?;
        let temp2 = TempDir::new()?;
        
        let project = temp1.path();
        let outside_file = temp2.path().join("evil.txt");
        fs::write(&outside_file, "evil")?;

        let result = validate_file_path(&outside_file, project);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("outside project"));

        Ok(())
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("normal.txt"), "normal.txt");
        assert_eq!(sanitize_filename("file-name_123.tar.gz"), "file-name_123.tar.gz");
        assert_eq!(sanitize_filename("../../../etc/passwd"), "etcpasswd");
        assert_eq!(sanitize_filename("file; rm -rf /"), "filerm-rf");  // hyphen is allowed
        assert_eq!(sanitize_filename("file\0name"), "filename");
        assert_eq!(sanitize_filename("...hidden"), "hidden");  // removes leading dots
    }

    #[test]
    fn test_contains_suspicious_patterns() {
        assert!(!contains_suspicious_patterns(Path::new("normal/path")));
        assert!(!contains_suspicious_patterns(Path::new("file.txt")));
        
        assert!(contains_suspicious_patterns(Path::new("../etc/passwd")));
        assert!(contains_suspicious_patterns(Path::new("path/../other")));
        assert!(contains_suspicious_patterns(Path::new("file\0name")));
    }
}
