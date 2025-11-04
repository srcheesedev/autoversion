use autoversion::updaters::python::PythonUpdater;
use autoversion::updaters::traits::VersionUpdater;
use std::fs;
use tempfile::TempDir;

#[test]
fn python_updater_creates_backups_and_updates_files() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let path = tmp.path();

    // Create pyproject.toml and setup.py
    fs::write(
        path.join("pyproject.toml"),
        r#"[project]
name = "contract-test"
version = "0.1.0"
"#,
    )?;

    fs::write(
        path.join("setup.py"),
        r#"from setuptools import setup
setup(name="contract-test", version="0.1.0")
"#,
    )?;

    let updater = PythonUpdater::new();

    let updated = updater.update_version(path, "0.2.0")?;

    // Backups should exist for files that were updated
    assert!(path.join("pyproject.toml.autoversion.backup").exists() || path.join("setup.py.autoversion.backup").exists());

    // Ensure updated files list contains at least one of the two
    assert!(updated.iter().any(|f| f.ends_with("pyproject.toml") || f.ends_with("setup.py")));

    // Verify at least one file contains the new version
    let py = fs::read_to_string(path.join("pyproject.toml"))?;
    let su = fs::read_to_string(path.join("setup.py"))?;
    assert!(py.contains("version = \"0.2.0\"") || su.contains("version=\"0.2.0\""));

    Ok(())
}
