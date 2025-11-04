use autoversion::updaters::cargo::CargoUpdater;
use autoversion::updaters::traits::VersionUpdater;
use std::fs;
use tempfile::TempDir;

#[test]
fn cargo_updater_creates_backup_and_updates_cargo_toml() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let path = tmp.path();

    // Create initial Cargo.toml
    fs::write(
        path.join("Cargo.toml"),
        r#"[package]
name = "contract-test"
version = "0.1.0"

[dependencies]
"#,
    )?;

    let updater = CargoUpdater::new();
    let updated = updater.update_version(path, "0.2.0")?;
    assert_eq!(updated.len(), 1);

    // Backup should exist
    assert!(path.join("Cargo.toml.autoversion.backup").exists());

    // Cargo.toml should be updated
    let content = fs::read_to_string(path.join("Cargo.toml"))?;
    assert!(content.contains("version = \"0.2.0\""));

    Ok(())
}
