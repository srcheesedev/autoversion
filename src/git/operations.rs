use anyhow::{anyhow, Result};
use git2::{Repository, Signature};
use std::path::Path;

use crate::constants::git::{DEFAULT_COMMIT_MESSAGE_PREFIX, DEFAULT_TAG_MESSAGE_PREFIX};

/// Create a git tag for the given version
pub fn create_tag(tag_name: &str, version: &str) -> Result<()> {
    // Backwards-compatible wrapper that creates tag in current directory
    create_tag_in(std::path::Path::new("."), tag_name, version)
}

/// Create a git tag in a specific project path
pub fn create_tag_in(project_path: &std::path::Path, tag_name: &str, version: &str) -> Result<()> {
    let repo = Repository::open(project_path).map_err(|e| {
        anyhow!(
            "Failed to open git repository at {}: {}",
            project_path.display(),
            e
        )
    })?;

    // Get the current HEAD commit
    let head = repo.head()?;
    let commit = head.peel_to_commit()?;

    // Create signature (use configured git user or fallback)
    let signature = get_git_signature(&repo)?;

    // Create annotated tag
    let message = format!("{} {}", DEFAULT_TAG_MESSAGE_PREFIX, version);
    repo.tag(tag_name, &commit.into_object(), &signature, &message, false)?;

    println!("✅ Created tag: {}", tag_name);
    Ok(())
}

/// Commit changes with a version bump message
pub fn commit_version_changes(files: &[String], version: &str) -> Result<()> {
    commit_version_changes_in(std::path::Path::new("."), files, version, None)
}

/// Commit version changes in a specific project path with optional custom message
pub fn commit_version_changes_in(
    project_path: &std::path::Path,
    files: &[String],
    version: &str,
    custom_message: Option<&str>,
) -> Result<()> {
    let repo = Repository::open(project_path).map_err(|e| {
        anyhow!(
            "Failed to open git repository at {}: {}",
            project_path.display(),
            e
        )
    })?;

    let mut index = repo.index()?;

    // Add specified files to the index
    for file_path in files {
        let path = Path::new(file_path);
        // If path is absolute, make it relative to project_path
        let relative_path = if path.is_absolute() {
            path.strip_prefix(project_path).unwrap_or(path)
        } else {
            path
        };
        index.add_path(relative_path)?;
    }
    index.write()?;

    // Create tree from index
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // Get parent commit
    let parent_commit = repo.head()?.peel_to_commit()?;

    // Create signature
    let signature = get_git_signature(&repo)?;

    // Commit message
    let message = if let Some(custom) = custom_message {
        custom.replace("{version}", version)
    } else {
        format!("chore: {} {}", DEFAULT_COMMIT_MESSAGE_PREFIX, version)
    };

    // Create commit
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &message,
        &tree,
        &[&parent_commit],
    )?;

    println!("✅ Committed version changes: {}", message);
    Ok(())
}

/// Check if the repository is clean (no uncommitted changes)
pub fn is_repository_clean() -> Result<bool> {
    is_repository_clean_in(std::path::Path::new("."))
}

/// Check if the repository at project_path is clean
pub fn is_repository_clean_in(project_path: &std::path::Path) -> Result<bool> {
    let repo = Repository::open(project_path).map_err(|e| {
        anyhow!(
            "Failed to open git repository at {}: {}",
            project_path.display(),
            e
        )
    })?;

    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(true);
    let statuses = repo.statuses(Some(&mut opts))?;
    Ok(statuses.is_empty())
}

/// Get git signature from configuration or create a fallback
fn get_git_signature(repo: &Repository) -> Result<Signature<'_>> {
    let config = repo.config()?;

    let name = config
        .get_string("user.name")
        .unwrap_or_else(|_| "Autoversion Action".to_string());

    let email = config
        .get_string("user.email")
        .unwrap_or_else(|_| "autoversion@github.actions".to_string());

    Signature::now(&name, &email).map_err(|e| anyhow!("Failed to create git signature: {}", e))
}

/// Check if a tag already exists
pub fn tag_exists(tag_name: &str) -> Result<bool> {
    tag_exists_in(std::path::Path::new("."), tag_name)
}

/// Check if a tag exists in the repository at project_path
pub fn tag_exists_in(project_path: &std::path::Path, tag_name: &str) -> Result<bool> {
    let repo = Repository::open(project_path).map_err(|e| {
        anyhow!(
            "Failed to open git repository at {}: {}",
            project_path.display(),
            e
        )
    })?;

    let result = match repo.find_reference(&format!("refs/tags/{}", tag_name)) {
        Ok(_) => Ok(true),
        Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(false),
        Err(e) => Err(anyhow!("Failed to check tag existence: {}", e)),
    };

    result
}

/// Get the latest tag in the repository
pub fn get_latest_tag() -> Result<Option<String>> {
    get_latest_tag_in(std::path::Path::new("."))
}

/// Get latest tag in repository at project_path
pub fn get_latest_tag_in(project_path: &std::path::Path) -> Result<Option<String>> {
    let repo = Repository::open(project_path).map_err(|e| {
        anyhow!(
            "Failed to open git repository at {}: {}",
            project_path.display(),
            e
        )
    })?;

    let tag_names = repo.tag_names(None)?;
    let mut tags = Vec::new();

    for tag_name in tag_names.iter().flatten() {
        tags.push(tag_name.to_string());
    }

    // Sort tags and return the latest (this is a simple sort, more sophisticated
    // version comparison could be implemented)
    tags.sort();
    Ok(tags.last().cloned())
}

/// Delete a git tag
pub fn delete_tag(tag_name: &str) -> Result<()> {
    delete_tag_in(std::path::Path::new("."), tag_name)
}

/// Delete a git tag in a specific project path
pub fn delete_tag_in(project_path: &std::path::Path, tag_name: &str) -> Result<()> {
    let repo = Repository::open(project_path).map_err(|e| {
        anyhow!(
            "Failed to open git repository at {}: {}",
            project_path.display(),
            e
        )
    })?;

    // Check if tag exists
    if !tag_exists_in(project_path, tag_name)? {
        return Err(anyhow!("Tag '{}' does not exist", tag_name));
    }

    // Delete the tag
    repo.tag_delete(tag_name)?;

    println!("🗑️  Deleted tag: {}", tag_name);
    Ok(())
}

/// Revert the last N commits
pub fn revert_last_commit() -> Result<()> {
    revert_last_commit_in(std::path::Path::new("."))
}

/// Revert the last commit in a specific project path
pub fn revert_last_commit_in(project_path: &std::path::Path) -> Result<()> {
    let repo = Repository::open(project_path).map_err(|e| {
        anyhow!(
            "Failed to open git repository at {}: {}",
            project_path.display(),
            e
        )
    })?;

    // Get HEAD commit
    let head = repo.head()?;
    let head_commit = head.peel_to_commit()?;

    // Check if there is a parent (can't revert initial commit)
    if head_commit.parent_count() == 0 {
        return Err(anyhow!("Cannot revert initial commit"));
    }

    // Get parent commit
    let parent_commit = head_commit.parent(0)?;

    // Reset HEAD to parent (soft reset - keeps working directory changes)
    repo.reset(parent_commit.as_object(), git2::ResetType::Soft, None)?;

    println!("⏮️  Reverted last commit: {}", head_commit.id());
    Ok(())
}

/// Initialize a new git repository (for testing)
#[cfg(test)]
pub fn init_test_repo(path: &Path) -> Result<Repository> {
    Repository::init(path).map_err(|e| anyhow!("Failed to initialize test repository: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    // std::fs is not used in these tests; remove to avoid unused import warning

    #[test]
    fn test_tag_operations() -> Result<()> {
        // Skip if not in a git repository or if repository operations would interfere
        if Repository::open(".").is_err() {
            println!("Skipping test: not in a git repository");
            return Ok(());
        }

        // Simple test that verifies the functions don't crash
        // We can't easily test tag creation without affecting the actual repository
        assert!(tag_exists("non-existent-tag-12345").is_ok());

        Ok(())
    }

    #[test]
    fn test_repository_clean_check() -> Result<()> {
        // Skip if not in a git repository
        if Repository::open(".").is_err() {
            println!("Skipping test: not in a git repository");
            return Ok(());
        }

        // Simple test that verifies the function doesn't crash
        assert!(is_repository_clean().is_ok());

        Ok(())
    }
}
