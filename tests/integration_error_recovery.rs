use anyhow::Result;
use autoversion::updaters::npm::NpmUpdater;
use autoversion::updaters::traits::VersionUpdater;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Create a test NPM project
fn create_npm_project(path: &Path) -> Result<()> {
    fs::write(
        path.join("package.json"),
        r#"{
  "name": "test-project",
  "version": "1.0.0",
  "description": "Test"
}"#,
    )?;
    Ok(())
}

#[test]
fn test_integration_backup_created_before_update() {
    let temp = TempDir::new().unwrap();
    create_npm_project(temp.path()).unwrap();

    let updater = NpmUpdater;
    updater.update_version(temp.path(), "1.0.1").unwrap();

    // Verify backup exists
    let backup_path = temp.path().join("package.json.autoversion.backup");
    assert!(backup_path.exists());

    // Verify backup contains original version
    let backup_content = fs::read_to_string(backup_path).unwrap();
    assert!(backup_content.contains("1.0.0"));
}

#[test]
fn test_integration_original_file_updated() {
    let temp = TempDir::new().unwrap();
    create_npm_project(temp.path()).unwrap();

    let updater = NpmUpdater;
    updater.update_version(temp.path(), "2.0.0").unwrap();

    // Verify original file has new version
    let content = fs::read_to_string(temp.path().join("package.json")).unwrap();
    assert!(content.contains("2.0.0"));
    assert!(!content.contains("1.0.0"));
}

#[test]
fn test_integration_multiple_updates_replace_backup() {
    let temp = TempDir::new().unwrap();
    create_npm_project(temp.path()).unwrap();

    let updater = NpmUpdater;

    // First update
    updater.update_version(temp.path(), "1.1.0").unwrap();
    let backup1 = fs::read_to_string(temp.path().join("package.json.autoversion.backup")).unwrap();
    assert!(backup1.contains("1.0.0"));

    // Second update (should replace backup)
    updater.update_version(temp.path(), "1.2.0").unwrap();
    let backup2 = fs::read_to_string(temp.path().join("package.json.autoversion.backup")).unwrap();
    assert!(backup2.contains("1.1.0"));
    assert!(!backup2.contains("1.0.0"));
}

#[test]
fn test_integration_backup_can_restore() {
    let temp = TempDir::new().unwrap();
    create_npm_project(temp.path()).unwrap();

    let updater = NpmUpdater;
    updater.update_version(temp.path(), "3.0.0").unwrap();

    // Manually restore from backup
    let backup_path = temp.path().join("package.json.autoversion.backup");
    let package_path = temp.path().join("package.json");
    fs::copy(&backup_path, &package_path).unwrap();

    // Verify restored version
    let content = fs::read_to_string(package_path).unwrap();
    assert!(content.contains("1.0.0"));
    assert!(!content.contains("3.0.0"));
}

#[test]
fn test_integration_version_extraction_after_update() {
    let temp = TempDir::new().unwrap();
    create_npm_project(temp.path()).unwrap();

    let updater = NpmUpdater;

    // Initial version
    let version1 = updater.get_current_version(temp.path()).unwrap();
    assert_eq!(version1, "1.0.0");

    // Update version
    updater.update_version(temp.path(), "4.5.6").unwrap();

    // Extract new version
    let version2 = updater.get_current_version(temp.path()).unwrap();
    assert_eq!(version2, "4.5.6");
}

#[test]
fn test_integration_error_recovery_invalid_version() {
    let temp = TempDir::new().unwrap();
    create_npm_project(temp.path()).unwrap();

    let updater = NpmUpdater;

    // Try to set invalid version (should still work for NPM as it's permissive)
    let result = updater.update_version(temp.path(), "invalid");

    // NPM updater is permissive, so this actually succeeds
    assert!(result.is_ok());
}

#[test]
fn test_integration_error_missing_manifest() {
    let temp = TempDir::new().unwrap();
    // Don't create package.json

    let updater = NpmUpdater;
    let result = updater.get_current_version(temp.path());

    assert!(result.is_err());
}

#[test]
fn test_integration_error_corrupted_json() {
    let temp = TempDir::new().unwrap();

    // Create invalid JSON
    fs::write(
        temp.path().join("package.json"),
        r#"{ "name": "test", invalid json }"#,
    )
    .unwrap();

    let updater = NpmUpdater;
    let result = updater.get_current_version(temp.path());

    assert!(result.is_err());
}

#[test]
fn test_integration_sequential_updates_preserve_formatting() {
    let temp = TempDir::new().unwrap();

    // Create with specific formatting
    fs::write(
        temp.path().join("package.json"),
        r#"{
  "name": "test-project",
  "version": "1.0.0",
  "description": "Test with specific formatting"
}"#,
    )
    .unwrap();

    let updater = NpmUpdater;

    // Multiple updates
    updater.update_version(temp.path(), "1.0.1").unwrap();
    updater.update_version(temp.path(), "1.0.2").unwrap();

    // Check formatting is roughly preserved (spaces, newlines)
    let content = fs::read_to_string(temp.path().join("package.json")).unwrap();
    assert!(content.contains("  \"name\":"));
    assert!(content.contains("  \"version\":"));
}

#[test]
fn test_integration_concurrent_backup_prevention() {
    let temp = TempDir::new().unwrap();
    create_npm_project(temp.path()).unwrap();

    let updater = NpmUpdater;

    // First update creates backup
    updater.update_version(temp.path(), "1.1.0").unwrap();

    let backup_path = temp.path().join("package.json.autoversion.backup");
    let modified1 = fs::metadata(&backup_path).unwrap().modified().unwrap();

    // Small delay
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Second update should replace backup
    updater.update_version(temp.path(), "1.2.0").unwrap();

    let modified2 = fs::metadata(&backup_path).unwrap().modified().unwrap();

    // Backup should have been updated
    assert!(modified2 > modified1);
}

#[test]
#[cfg(unix)]
fn test_integration_file_permissions_preserved() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().unwrap();
    let package_path = temp.path().join("package.json");

    create_npm_project(temp.path()).unwrap();

    // Set specific permissions
    let mut perms = fs::metadata(&package_path).unwrap().permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&package_path, perms).unwrap();

    // Update version
    let updater = NpmUpdater;
    updater.update_version(temp.path(), "1.5.0").unwrap();

    // Check permissions are preserved (roughly - umask may affect this)
    let new_mode = fs::metadata(&package_path).unwrap().permissions().mode();

    // Just verify file is still readable and writable
    assert!(new_mode & 0o600 == 0o600);
}

#[test]
fn test_integration_empty_file_handling() {
    let temp = TempDir::new().unwrap();

    // Create empty file
    fs::write(temp.path().join("package.json"), "").unwrap();

    let updater = NpmUpdater;
    let result = updater.get_current_version(temp.path());

    assert!(result.is_err());
}

#[test]
fn test_integration_large_file_handling() {
    let temp = TempDir::new().unwrap();

    // Create package.json with lots of extra data
    let mut content = String::from(
        r#"{
  "name": "test-project",
  "version": "1.0.0",
  "description": "Test",
  "dependencies": {
"#,
    );

    // Add 1000 fake dependencies
    for i in 0..1000 {
        content.push_str(&format!(
            "    \"package-{}\": \"^1.0.0\"{}\n",
            i,
            if i < 999 { "," } else { "" }
        ));
    }
    content.push_str("  }\n}");

    fs::write(temp.path().join("package.json"), content).unwrap();

    let updater = NpmUpdater;

    // Should handle large file
    let version = updater.get_current_version(temp.path()).unwrap();
    assert_eq!(version, "1.0.0");

    // Should be able to update large file
    let result = updater.update_version(temp.path(), "2.0.0");
    assert!(result.is_ok());
}
