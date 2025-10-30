use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

use super::traits::{VersionUpdater, VersionChange};

/// Python pyproject.toml/setup.py version updater (stub implementation for MVP)
pub struct PythonUpdater;

impl PythonUpdater {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PythonUpdater {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionUpdater for PythonUpdater {
    fn get_current_version(&self, _project_path: &Path) -> Result<String> {
        Err(anyhow!("Python updater not yet implemented - coming in Phase 2"))
    }

    fn update_version(&self, _project_path: &Path, _new_version: &str) -> Result<Vec<String>> {
        Err(anyhow!("Python updater not yet implemented - coming in Phase 2"))
    }

    fn validate_project(&self, project_path: &Path) -> Result<()> {
        if project_path.join("pyproject.toml").exists() || project_path.join("setup.py").exists() {
            Err(anyhow!("Python updater not yet implemented - coming in Phase 2"))
        } else {
            Err(anyhow!("No Python project files found (pyproject.toml, setup.py)"))
        }
    }

    fn technology_name(&self) -> &'static str {
        "python"
    }

    fn get_primary_file(&self, project_path: &Path) -> Result<PathBuf> {
        let pyproject = project_path.join("pyproject.toml");
        if pyproject.exists() {
            Ok(pyproject)
        } else {
            let setup_py = project_path.join("setup.py");
            if setup_py.exists() {
                Ok(setup_py)
            } else {
                Err(anyhow!("No Python project files found"))
            }
        }
    }

    fn can_handle(&self, project_path: &Path) -> bool {
        project_path.join("pyproject.toml").exists() || project_path.join("setup.py").exists()
    }

    fn preview_changes(&self, _project_path: &Path, _new_version: &str) -> Result<Vec<VersionChange>> {
        Err(anyhow!("Python updater not yet implemented - coming in Phase 2"))
    }
}