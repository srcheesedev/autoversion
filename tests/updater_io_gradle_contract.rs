use autoversion::updaters::gradle::GradleUpdater;
use autoversion::updaters::traits::VersionUpdater;
use std::fs;
use tempfile::TempDir;

/// Contract test: GradleUpdater should create backup and update gradle.properties
#[test]
fn gradle_updater_creates_backup_and_updates_gradle_properties() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let project_path = temp.path();

    // Create gradle.properties
    fs::write(
        project_path.join("gradle.properties"),
        "# Project properties\nversion=1.0.0\ngroup=com.example\n",
    )?;

    let updater = GradleUpdater::new();

    // Run update
    let updated_files = updater.update_version(project_path, "1.1.0")?;

    // Backup should exist
    assert!(
        project_path
            .join("gradle.properties.autoversion.backup")
            .exists(),
        "gradle.properties backup missing"
    );

    // Updated files list should contain gradle.properties
    assert!(updated_files
        .iter()
        .any(|f| f.ends_with("gradle.properties")));

    // Contents should reflect new version
    let content = fs::read_to_string(project_path.join("gradle.properties"))?;
    assert!(
        content.contains("version=1.1.0"),
        "Version not updated in gradle.properties"
    );

    Ok(())
}

/// Contract test: GradleUpdater should create backup and update build.gradle
#[test]
fn gradle_updater_creates_backup_and_updates_build_gradle() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let project_path = temp.path();

    // Create build.gradle
    fs::write(
        project_path.join("build.gradle"),
        "plugins {\n    id 'java'\n}\n\nversion = '2.0.0'\ngroup = 'com.example'\n",
    )?;

    let updater = GradleUpdater::new();

    // Run update
    let updated_files = updater.update_version(project_path, "2.1.0")?;

    // Backup should exist
    assert!(
        project_path
            .join("build.gradle.autoversion.backup")
            .exists(),
        "build.gradle backup missing"
    );

    // Updated files list should contain build.gradle
    assert!(updated_files.iter().any(|f| f.ends_with("build.gradle")));

    // Contents should reflect new version
    let content = fs::read_to_string(project_path.join("build.gradle"))?;
    assert!(
        content.contains("version = '2.1.0'"),
        "Version not updated in build.gradle"
    );

    Ok(())
}

/// Contract test: GradleUpdater should create backup and update build.gradle.kts
#[test]
fn gradle_updater_creates_backup_and_updates_build_gradle_kts() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let project_path = temp.path();

    // Create build.gradle.kts
    fs::write(
        project_path.join("build.gradle.kts"),
        "plugins {\n    java\n}\n\nversion = \"3.0.0\"\ngroup = \"com.example\"\n",
    )?;

    let updater = GradleUpdater::new();

    // Run update
    let updated_files = updater.update_version(project_path, "3.1.0")?;

    // Backup should exist
    assert!(
        project_path
            .join("build.gradle.kts.autoversion.backup")
            .exists(),
        "build.gradle.kts backup missing"
    );

    // Updated files list should contain build.gradle.kts
    assert!(updated_files
        .iter()
        .any(|f| f.ends_with("build.gradle.kts")));

    // Contents should reflect new version
    let content = fs::read_to_string(project_path.join("build.gradle.kts"))?;
    assert!(
        content.contains("version = \"3.1.0\""),
        "Version not updated in build.gradle.kts"
    );

    Ok(())
}
