use anyhow::Result;
use std::path::Path;

/// Common interface for all version updaters
pub trait VersionUpdater {
    /// Get the current version from the project
    fn get_current_version(&self, project_path: &Path) -> Result<String>;

    /// Update the version in the project files
    fn update_version(&self, project_path: &Path, new_version: &str) -> Result<Vec<String>>;

    /// Validate that the project structure is compatible with this updater
    fn validate_project(&self, project_path: &Path) -> Result<()>;

    /// Get the name of the technology this updater handles
    fn technology_name(&self) -> &'static str;

    /// Get the primary file that contains the version
    fn get_primary_file(&self, project_path: &Path) -> Result<std::path::PathBuf>;

    /// Check if this updater can handle the project
    fn can_handle(&self, project_path: &Path) -> bool;

    /// Preview the changes that would be made (for dry-run mode)
    fn preview_changes(&self, project_path: &Path, new_version: &str)
        -> Result<Vec<VersionChange>>;
}

/// Represents a change that will be made to a file
#[derive(Debug, Clone)]
pub struct VersionChange {
    pub file_path: std::path::PathBuf,
    pub old_content: String,
    pub new_content: String,
    pub old_version: String,
    pub new_version: String,
}

impl VersionChange {
    pub fn new(
        file_path: std::path::PathBuf,
        old_content: String,
        new_content: String,
        old_version: String,
        new_version: String,
    ) -> Self {
        Self {
            file_path,
            old_content,
            new_content,
            old_version,
            new_version,
        }
    }
}
