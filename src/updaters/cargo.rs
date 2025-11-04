use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

use super::traits::{VersionUpdater, VersionChange};
use crate::constants::manifests::CARGO_TOML;
use crate::utils::files::{backup_file, read_file_safe, write_file_safe};
use toml::Value as TomlValue;

/// CargoUpdater
///
/// Implements logic to detect, read and update the version in a `Cargo.toml`.
/// It targets common Rust project layouts where the version lives in the
/// `[package]` table as `version = "x.y.z"`.
///
/// Documentation and TDD:
/// - Each public behavior (get_current_version, update_version, preview_changes,
///   validate_project, can_handle) is covered by unit tests in this module
///   (see `#[cfg(test)]`).
/// - Tests follow TDD: they define the expected behavior and the implementation
///   verifies those expectations.
///
/// Method contract summary:
/// - get_current_version(project_path) -> String (errors if not found)
/// - update_version(project_path, new_version) -> Vec<String> (updated files)
/// - preview_changes(project_path, new_version) -> Vec<VersionChange>
/// - validate_project(project_path) -> Result<()> (ok when Cargo.toml exists)
/// - can_handle(project_path) -> bool (true when Cargo.toml exists)
///
/// Antipatterns avoided:
/// - Never writing to files without taking a backup (`backup_file` is used).
/// - Avoiding regex-based parsing for TOML structures: we use the `toml` crate.

pub struct CargoUpdater;

impl CargoUpdater {
    pub fn new() -> Self {
        Self
    }

    fn parse_cargo_toml(&self, content: &str) -> Result<TomlValue> {
        content
            .parse::<TomlValue>()
            .map_err(|e| anyhow!("Failed to parse Cargo.toml: {}", e))
    }

    fn update_cargo_toml_content(&self, content: &str, new_version: &str) -> Result<String> {
        let mut value = self.parse_cargo_toml(content)?;

        if let Some(table) = value.get_mut("package") {
            if let Some(pkg_table) = table.as_table_mut() {
                pkg_table.insert("version".to_string(), TomlValue::String(new_version.to_string()));
            }
        } else {
            return Err(anyhow!("Cargo.toml does not contain [package] table with version"));
        }

        // Serialize back to TOML
        let updated = toml::to_string_pretty(&value)?;
        Ok(updated + "\n")
    }
}

impl Default for CargoUpdater {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionUpdater for CargoUpdater {
    fn get_current_version(&self, project_path: &Path) -> Result<String> {
        let cargo_toml = project_path.join(CARGO_TOML);
        let content = read_file_safe(&cargo_toml)?;

        let value = self.parse_cargo_toml(&content)?;
        if let Some(pkg) = value.get("package") {
            if let Some(ver) = pkg.get("version") {
                if let Some(s) = ver.as_str() {
                    return Ok(s.to_string());
                }
            }
        }

        Err(anyhow!("No version field found in Cargo.toml"))
    }

    fn update_version(&self, project_path: &Path, new_version: &str) -> Result<Vec<String>> {
        let cargo_toml = project_path.join(CARGO_TOML);
        backup_file(&cargo_toml)?;

        let old_content = read_file_safe(&cargo_toml)?;
        let updated = self.update_cargo_toml_content(&old_content, new_version)?;
        write_file_safe(&cargo_toml, &updated)?;

        Ok(vec![cargo_toml.to_string_lossy().to_string()])
    }

    fn validate_project(&self, project_path: &Path) -> Result<()> {
        let cargo_toml = project_path.join(CARGO_TOML);
        if cargo_toml.exists() {
            Ok(())
        } else {
            Err(anyhow!("Cargo.toml not found"))
        }
    }

    fn technology_name(&self) -> &'static str {
        "cargo"
    }

    fn get_primary_file(&self, project_path: &Path) -> Result<PathBuf> {
        let cargo_toml = project_path.join(CARGO_TOML);
        if cargo_toml.exists() {
            Ok(cargo_toml)
        } else {
            Err(anyhow!("Cargo.toml not found"))
        }
    }

    fn can_handle(&self, project_path: &Path) -> bool {
        project_path.join(CARGO_TOML).exists()
    }

    fn preview_changes(&self, project_path: &Path, new_version: &str) -> Result<Vec<VersionChange>> {
        let cargo_toml = project_path.join(CARGO_TOML);
        let mut changes = Vec::new();
        if cargo_toml.exists() {
            let old_content = read_file_safe(&cargo_toml)?;
            let new_content = self.update_cargo_toml_content(&old_content, new_version)?;
            let old_version = self.get_current_version(project_path)?;
            changes.push(VersionChange::new(
                cargo_toml,
                old_content,
                new_content,
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
    use tempfile::TempDir;
    use std::fs;

    // Helper used by multiple tests to create a minimal Cargo.toml in a temp dir.
    // TDD note: los tests crean un entorno aislado (TempDir) que simula un proyecto
    // real; así toda la lógica se valida sin tocar el workspace del desarrollador.
    fn create_cargo_toml(dir: &std::path::Path, version: &str) -> anyhow::Result<()> {
        let content = format!(
            "[package]\nname = \"test-package\"\nversion = \"{}\"\n\n[dependencies]\n",
            version
        );
        fs::write(dir.join(CARGO_TOML), content)?;
        Ok(())
    }

    #[test]
    fn test_get_current_version() -> anyhow::Result<()> {
        let tmp = TempDir::new()?;
        let path = tmp.path();
        create_cargo_toml(path, "1.2.3")?;

        let updater = CargoUpdater::new();
        let v = updater.get_current_version(path)?;
        assert_eq!(v, "1.2.3");
        Ok(())
    }

    #[test]
    fn test_update_version() -> anyhow::Result<()> {
        let tmp = TempDir::new()?;
        let path = tmp.path();
        create_cargo_toml(path, "0.1.0")?;

        let updater = CargoUpdater::new();
        let updated = updater.update_version(path, "0.2.0")?;
        assert_eq!(updated.len(), 1);

        let content = fs::read_to_string(path.join(CARGO_TOML))?;
        assert!(content.contains("version = \"0.2.0\""));
        Ok(())
    }

    #[test]
    fn test_validate_and_can_handle() -> anyhow::Result<()> {
        let tmp = TempDir::new()?;
        let path = tmp.path();
        assert!(!CargoUpdater::new().can_handle(path));

        create_cargo_toml(path, "1.0.0")?;
        assert!(CargoUpdater::new().can_handle(path));
        assert!(CargoUpdater::new().validate_project(path).is_ok());
        Ok(())
    }

    #[test]
    fn test_preview_changes() -> anyhow::Result<()> {
        let tmp = TempDir::new()?;
        let path = tmp.path();
        create_cargo_toml(path, "1.0.0")?;

        let changes = CargoUpdater::new().preview_changes(path, "2.0.0")?;
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].old_version, "1.0.0");
        assert_eq!(changes[0].new_version, "2.0.0");
        Ok(())
    }

    #[test]
    fn test_technology_name() {
        assert_eq!(CargoUpdater::new().technology_name(), "cargo");
    }
}