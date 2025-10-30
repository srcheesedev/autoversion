use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

use super::traits::{VersionUpdater, VersionChange};

/// Maven pom.xml version updater (stub implementation for MVP)
pub struct MavenUpdater;

impl MavenUpdater {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MavenUpdater {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionUpdater for MavenUpdater {
    fn get_current_version(&self, _project_path: &Path) -> Result<String> {
        Err(anyhow!("Maven updater not yet implemented - coming in Phase 2"))
    }

    fn update_version(&self, _project_path: &Path, _new_version: &str) -> Result<Vec<String>> {
        Err(anyhow!("Maven updater not yet implemented - coming in Phase 2"))
    }

    fn validate_project(&self, project_path: &Path) -> Result<()> {
        if project_path.join("pom.xml").exists() {
            Err(anyhow!("Maven updater not yet implemented - coming in Phase 2"))
        } else {
            Err(anyhow!("pom.xml not found"))
        }
    }

    fn technology_name(&self) -> &'static str {
        "maven"
    }

    fn get_primary_file(&self, project_path: &Path) -> Result<PathBuf> {
        let pom_xml = project_path.join("pom.xml");
        if pom_xml.exists() {
            Ok(pom_xml)
        } else {
            Err(anyhow!("pom.xml not found"))
        }
    }

    fn can_handle(&self, project_path: &Path) -> bool {
        project_path.join("pom.xml").exists()
    }

    fn preview_changes(&self, _project_path: &Path, _new_version: &str) -> Result<Vec<VersionChange>> {
        Err(anyhow!("Maven updater not yet implemented - coming in Phase 2"))
    }
}