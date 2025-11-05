/// Contract test for Composer updater I/O operations
///
/// This test validates that the ComposerUpdater:
/// 1. Creates backup files before modification
/// 2. Actually updates the composer.json content
/// 3. Optionally updates composer.lock if present
/// 4. Follows the same I/O contract as other updaters
use autoversion::updaters::composer::ComposerUpdater;
use autoversion::updaters::traits::VersionUpdater;
use std::fs;
use tempfile::TempDir;

#[test]
fn composer_updater_creates_backups_and_updates_files() {
    // Arrange: Create a temporary PHP Composer project
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Create composer.json with initial version
    fs::write(
        project_path.join("composer.json"),
        r#"{
    "name": "vendor/test-package",
    "version": "1.0.0",
    "require": {
        "php": "^8.0"
    }
}"#,
    )
    .unwrap();

    // Create composer.lock
    fs::write(
        project_path.join("composer.lock"),
        r#"{
    "packages": [],
    "version": "1.0.0"
}"#,
    )
    .unwrap();

    let updater = ComposerUpdater::new();

    // Act: Update the version
    let result = updater.update_version(project_path, "2.0.0");

    // Assert: Operation succeeded
    assert!(result.is_ok(), "Update should succeed");
    let updated_files = result.unwrap();
    assert_eq!(
        updated_files.len(),
        2,
        "Should update both composer.json and composer.lock"
    );

    // Assert: Backups were created
    let json_backup = project_path.join("composer.json.autoversion.backup");
    let lock_backup = project_path.join("composer.lock.autoversion.backup");

    assert!(json_backup.exists(), "composer.json backup should exist");
    assert!(lock_backup.exists(), "composer.lock backup should exist");

    // Assert: Backups contain old version
    let json_backup_content = fs::read_to_string(&json_backup).unwrap();
    assert!(
        json_backup_content.contains("1.0.0"),
        "Backup should contain original version"
    );

    // Assert: composer.json contains new version
    let json_content = fs::read_to_string(project_path.join("composer.json")).unwrap();
    assert!(
        json_content.contains("2.0.0"),
        "composer.json should contain new version"
    );
    assert!(
        json_content.contains("vendor/test-package"),
        "composer.json should preserve other fields"
    );

    // Assert: composer.lock contains new version
    let lock_content = fs::read_to_string(project_path.join("composer.lock")).unwrap();
    assert!(
        lock_content.contains("2.0.0"),
        "composer.lock should contain new version"
    );
}

#[test]
fn composer_updater_works_without_composer_lock() {
    // Arrange: PHP project with only composer.json
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    fs::write(
        project_path.join("composer.json"),
        r#"{"name": "vendor/package", "version": "1.0.0"}"#,
    )
    .unwrap();

    let updater = ComposerUpdater::new();

    // Act: Update should succeed
    let result = updater.update_version(project_path, "2.0.0");

    // Assert
    assert!(
        result.is_ok(),
        "Update should succeed without composer.lock"
    );
    let updated_files = result.unwrap();
    assert_eq!(
        updated_files.len(),
        1,
        "Should only update composer.json when lock doesn't exist"
    );

    // Verify content
    let content = fs::read_to_string(project_path.join("composer.json")).unwrap();
    assert!(content.contains("2.0.0"), "Should contain new version");
}

#[test]
fn composer_updater_validates_composer_json_presence() {
    // Arrange: Directory without composer.json
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    let updater = ComposerUpdater::new();

    // Act & Assert: Validation should fail
    let result = updater.validate_project(project_path);
    assert!(
        result.is_err(),
        "Should fail validation without composer.json"
    );
    assert!(
        result.unwrap_err().to_string().contains("composer.json"),
        "Error should mention composer.json"
    );
}
