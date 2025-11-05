use anyhow::Result;
use std::sync::Arc;

use super::cargo::CargoUpdater;
use super::composer::ComposerUpdater;
use super::generic::GenericUpdater;
use super::go::GoUpdater;
use super::gradle::GradleUpdater;
use super::maven::MavenUpdater;
use super::npm::NpmUpdater;
use super::python::PythonUpdater;
use super::traits::VersionUpdater;
use crate::core::detector::Technology;

/// Factory for creating appropriate version updaters
pub struct UpdaterFactory;

impl UpdaterFactory {
    /// Create an updater for the specified technology
    pub fn create(technology: &str) -> Result<Arc<dyn VersionUpdater>> {
        let tech: Technology = technology.parse()?;

        let updater: Arc<dyn VersionUpdater> = match tech {
            Technology::Npm => Arc::new(NpmUpdater::new()),
            Technology::Cargo => Arc::new(CargoUpdater::new()),
            Technology::Maven => Arc::new(MavenUpdater::new()),
            Technology::Python => Arc::new(PythonUpdater::new()),
            Technology::Go => Arc::new(GoUpdater::new()),
            Technology::Composer => Arc::new(ComposerUpdater::new()),
            Technology::Gradle => Arc::new(GradleUpdater::new()),
            Technology::Generic => Arc::new(GenericUpdater::new()),
        };

        Ok(updater)
    }

    /// Create an updater by auto-detecting the technology
    pub fn create_auto(project_path: &std::path::Path) -> Result<Arc<dyn VersionUpdater>> {
        use crate::core::detector::TechnologyDetector;

        let detector = TechnologyDetector::new();
        let technology = detector.detect(project_path)?;
        Self::create(&technology)
    }

    /// Get all available updaters for multi-technology projects
    pub fn get_all_updaters() -> Vec<Arc<dyn VersionUpdater>> {
        vec![
            Arc::new(NpmUpdater::new()),
            Arc::new(CargoUpdater::new()),
            Arc::new(MavenUpdater::new()),
            Arc::new(PythonUpdater::new()),
            Arc::new(GoUpdater::new()),
            Arc::new(ComposerUpdater::new()),
            Arc::new(GenericUpdater::new()),
        ]
    }

    /// Find all compatible updaters for a project
    pub fn find_compatible_updaters(
        project_path: &std::path::Path,
    ) -> Vec<Arc<dyn VersionUpdater>> {
        Self::get_all_updaters()
            .into_iter()
            .filter(|updater| updater.can_handle(project_path))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_create_npm_updater() -> Result<()> {
        let updater = UpdaterFactory::create("npm")?;
        assert_eq!(updater.technology_name(), "npm");
        Ok(())
    }

    #[test]
    fn test_create_cargo_updater() -> Result<()> {
        let updater = UpdaterFactory::create("cargo")?;
        assert_eq!(updater.technology_name(), "cargo");
        Ok(())
    }

    #[test]
    fn test_create_invalid_technology() {
        let result = UpdaterFactory::create("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_auto_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();

        // Create package.json
        fs::write(
            project_path.join("package.json"),
            r#"{"name": "test", "version": "1.0.0"}"#,
        )?;

        let updater = UpdaterFactory::create_auto(project_path)?;
        assert_eq!(updater.technology_name(), "npm");

        Ok(())
    }

    #[test]
    fn test_find_compatible_updaters() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();

        // Create both package.json and VERSION file
        fs::write(
            project_path.join("package.json"),
            r#"{"name": "test", "version": "1.0.0"}"#,
        )?;
        fs::write(project_path.join("VERSION"), "1.0.0")?;

        let updaters = UpdaterFactory::find_compatible_updaters(project_path);

        // Should find both npm and generic updaters
        assert!(updaters.len() >= 2);

        let technologies: Vec<&str> = updaters.iter().map(|u| u.technology_name()).collect();

        assert!(technologies.contains(&"npm"));
        assert!(technologies.contains(&"generic"));

        Ok(())
    }
}
