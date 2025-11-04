/// Contract test for Go updater I/O operations
/// 
/// This test validates that the GoUpdater:
/// 1. Creates backup files before modification
/// 2. Actually updates the VERSION file content
/// 3. Follows the same I/O contract as other updaters
use autoversion::updaters::go::GoUpdater;
use autoversion::updaters::traits::VersionUpdater;
use std::fs;
use tempfile::TempDir;

#[test]
fn go_updater_creates_backup_and_updates_version_file() {
    // Arrange: Create a temporary Go project
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    // Create go.mod (required for validation)
    fs::write(
        project_path.join("go.mod"),
        "module github.com/user/project\n\ngo 1.21\n",
    ).unwrap();

    // Create VERSION file with initial version
    fs::write(project_path.join("VERSION"), "v1.0.0\n").unwrap();

    let updater = GoUpdater::new();

    // Act: Update the version
    let result = updater.update_version(project_path, "2.0.0");

    // Assert: Operation succeeded
    assert!(result.is_ok(), "Update should succeed");
    let updated_files = result.unwrap();
    assert_eq!(updated_files.len(), 1, "Should update VERSION file");

    // Assert: Backup was created
    let backup_file = project_path.join("VERSION..autoversion.backup");
    assert!(
        backup_file.exists(),
        "Backup file should exist at {:?}",
        backup_file
    );

    // Assert: Backup contains old version
    let backup_content = fs::read_to_string(&backup_file).unwrap();
    assert_eq!(
        backup_content.trim(),
        "v1.0.0",
        "Backup should contain original version"
    );

    // Assert: VERSION file contains new version
    let version_content = fs::read_to_string(project_path.join("VERSION")).unwrap();
    assert_eq!(
        version_content.trim(),
        "v2.0.0",
        "VERSION file should contain new version"
    );
}

#[test]
fn go_updater_works_without_version_file() {
    // Arrange: Go project without VERSION file (git tags only)
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    fs::write(
        project_path.join("go.mod"),
        "module github.com/user/project\n\ngo 1.21\n",
    ).unwrap();

    let updater = GoUpdater::new();

    // Act: Update should succeed but not create any files
    let result = updater.update_version(project_path, "1.0.0");

    // Assert
    assert!(result.is_ok(), "Update should succeed even without VERSION file");
    let updated_files = result.unwrap();
    assert_eq!(
        updated_files.len(),
        0,
        "Should not update any files when VERSION doesn't exist"
    );

    // No VERSION file should be created
    assert!(
        !project_path.join("VERSION").exists(),
        "VERSION file should not be auto-created"
    );
}

#[test]
fn go_updater_validates_go_mod_presence() {
    // Arrange: Directory without go.mod
    let temp_dir = TempDir::new().unwrap();
    let project_path = temp_dir.path();

    let updater = GoUpdater::new();

    // Act & Assert: Validation should fail
    let result = updater.validate_project(project_path);
    assert!(result.is_err(), "Should fail validation without go.mod");
    assert!(
        result.unwrap_err().to_string().contains("go.mod"),
        "Error should mention go.mod"
    );
}
