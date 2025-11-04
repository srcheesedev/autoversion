use autoversion::updaters::npm::NpmUpdater;
use autoversion::updaters::traits::VersionUpdater;
use std::fs;
use tempfile::TempDir;

/// Contract test: NpmUpdater.update_version should create backups and update package.json and package-lock.json
#[test]
fn npm_updater_creates_backups_and_updates_files() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let project_path = temp.path();

    // Create package.json and package-lock.json
    fs::write(
        project_path.join("package.json"),
        r#"{
  "name": "contract-test",
  "version": "0.1.0"
}
"#,
    )?;

    fs::write(
        project_path.join("package-lock.json"),
        r#"{
  "name": "contract-test",
  "version": "0.1.0",
  "lockfileVersion": 2,
  "packages": {
    "": { "name": "contract-test", "version": "0.1.0" }
  }
}
"#,
    )?;

    let updater = NpmUpdater::new();

    // Run update
    let updated_files = updater.update_version(project_path, "0.2.0")?;

    // Backups should exist
    assert!(project_path.join("package.json.autoversion.backup").exists(), "package.json backup missing");
    assert!(project_path.join("package-lock.json.autoversion.backup").exists(), "package-lock.json backup missing");

    // Updated files list should contain both
    assert!(updated_files.iter().any(|f| f.ends_with("package.json")));
    assert!(updated_files.iter().any(|f| f.ends_with("package-lock.json")));

    // Contents should reflect new version
    let pkg = fs::read_to_string(project_path.join("package.json"))?;
    assert!(pkg.contains("\"version\": \"0.2.0\""));

    let lock = fs::read_to_string(project_path.join("package-lock.json"))?;
    assert!(lock.contains("\"version\": \"0.2.0\""));

    Ok(())
}
