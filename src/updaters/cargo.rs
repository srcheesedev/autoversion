use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

use super::traits::{VersionUpdater, VersionChange};

/// Cargo.toml version updater (stub implementation for MVP)
pub struct CargoUpdater;

impl CargoUpdater {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CargoUpdater {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionUpdater for CargoUpdater {
    fn get_current_version(&self, _project_path: &Path) -> Result<String> {
        Err(anyhow!("Cargo updater not yet implemented - coming in Phase 2"))
    }

    fn update_version(&self, _project_path: &Path, _new_version: &str) -> Result<Vec<String>> {
        Err(anyhow!("Cargo updater not yet implemented - coming in Phase 2"))
    }

    fn validate_project(&self, project_path: &Path) -> Result<()> {
        if project_path.join("Cargo.toml").exists() {
            Err(anyhow!("Cargo updater not yet implemented - coming in Phase 2"))
        } else {
            Err(anyhow!("Cargo.toml not found"))
        }
    }

    fn technology_name(&self) -> &'static str {
        "cargo"
    }

    fn get_primary_file(&self, project_path: &Path) -> Result<PathBuf> {
        let cargo_toml = project_path.join("Cargo.toml");
        if cargo_toml.exists() {
            Ok(cargo_toml)
        } else {
            Err(anyhow!("Cargo.toml not found"))
        }
    }

    fn can_handle(&self, project_path: &Path) -> bool {
        project_path.join("Cargo.toml").exists()
    }

    fn preview_changes(&self, _project_path: &Path, _new_version: &str) -> Result<Vec<VersionChange>> {
        Err(anyhow!("Cargo updater not yet implemented - coming in Phase 2"))
    }
}