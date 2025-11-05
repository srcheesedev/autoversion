use anyhow::Result;
use autoversion::git::operations;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Helper to initialize a git repository for testing
fn init_test_repo(path: &Path) -> Result<()> {
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()?;

    // Create initial commit
    fs::write(path.join("README.md"), "# Test Project")?;
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output()?;

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()?;

    Ok(())
}

#[test]
fn test_integration_tag_creation() {
    let temp = TempDir::new().unwrap();
    init_test_repo(temp.path()).unwrap();

    // Create tag
    operations::create_tag_in(temp.path(), "v1.0.0", "1.0.0").unwrap();

    // Verify tag exists
    assert!(operations::tag_exists_in(temp.path(), "v1.0.0").unwrap());
}

#[test]
fn test_integration_duplicate_tag_fails() {
    let temp = TempDir::new().unwrap();
    init_test_repo(temp.path()).unwrap();

    // Create first tag
    operations::create_tag_in(temp.path(), "v1.0.0", "1.0.0").unwrap();

    // Try to create duplicate tag (should fail)
    let result = operations::create_tag_in(temp.path(), "v1.0.0", "1.0.0");
    assert!(result.is_err());
}

#[test]
fn test_integration_get_latest_tag() {
    let temp = TempDir::new().unwrap();
    init_test_repo(temp.path()).unwrap();

    // Initially no tags
    let latest = operations::get_latest_tag_in(temp.path()).unwrap();
    assert!(latest.is_none());

    // Create some tags
    operations::create_tag_in(temp.path(), "v1.0.0", "1.0.0").unwrap();
    operations::create_tag_in(temp.path(), "v1.1.0", "1.1.0").unwrap();

    // Get latest
    let latest = operations::get_latest_tag_in(temp.path()).unwrap();
    assert!(latest.is_some());
    // Note: git returns tags in lexicographical order, not semantic version order
}

#[test]
fn test_integration_commit_changes() {
    let temp = TempDir::new().unwrap();
    init_test_repo(temp.path()).unwrap();

    // Create a file to commit
    let test_file = temp.path().join("version.txt");
    fs::write(&test_file, "1.0.0").unwrap();

    // Commit the file
    operations::commit_version_changes_in(
        temp.path(),
        &[String::from("version.txt")],
        "1.0.0",
        None,
    )
    .unwrap();

    // Verify commit was created
    let output = std::process::Command::new("git")
        .args(["log", "--oneline", "-1"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let log = String::from_utf8_lossy(&output.stdout);
    assert!(log.contains("Bump version") || log.contains("1.0.0"));
}

#[test]
fn test_integration_commit_with_custom_message() {
    let temp = TempDir::new().unwrap();
    init_test_repo(temp.path()).unwrap();

    let test_file = temp.path().join("version.txt");
    fs::write(&test_file, "2.0.0").unwrap();

    operations::commit_version_changes_in(
        temp.path(),
        &[String::from("version.txt")],
        "2.0.0",
        Some("🚀 Release version {version}"),
    )
    .unwrap();

    let output = std::process::Command::new("git")
        .args(["log", "--oneline", "-1"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let log = String::from_utf8_lossy(&output.stdout);
    assert!(log.contains("🚀 Release version 2.0.0"));
}

#[test]
fn test_integration_tag_and_commit_workflow() {
    let temp = TempDir::new().unwrap();
    init_test_repo(temp.path()).unwrap();

    // Update version file
    let version_file = temp.path().join("VERSION");
    fs::write(&version_file, "1.5.0").unwrap();

    // Commit changes
    operations::commit_version_changes_in(temp.path(), &[String::from("VERSION")], "1.5.0", None)
        .unwrap();

    // Create tag
    operations::create_tag_in(temp.path(), "v1.5.0", "1.5.0").unwrap();

    // Verify both exist
    assert!(operations::tag_exists_in(temp.path(), "v1.5.0").unwrap());

    let output = std::process::Command::new("git")
        .args(["log", "--oneline", "-1"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let log = String::from_utf8_lossy(&output.stdout);
    assert!(log.contains("1.5.0") || log.contains("Bump version"));
}

#[test]
fn test_integration_repository_clean_check() {
    let temp = TempDir::new().unwrap();
    init_test_repo(temp.path()).unwrap();

    // Repository should be clean
    assert!(operations::is_repository_clean_in(temp.path()).unwrap());

    // Add uncommitted file
    fs::write(temp.path().join("uncommitted.txt"), "test").unwrap();

    // Repository should no longer be clean
    assert!(!operations::is_repository_clean_in(temp.path()).unwrap());
}

#[test]
fn test_integration_commit_multiple_files() {
    let temp = TempDir::new().unwrap();
    init_test_repo(temp.path()).unwrap();

    // Create multiple files
    fs::write(temp.path().join("package.json"), r#"{"version": "1.2.3"}"#).unwrap();
    fs::write(
        temp.path().join("package-lock.json"),
        r#"{"version": "1.2.3"}"#,
    )
    .unwrap();

    // Commit both files
    operations::commit_version_changes_in(
        temp.path(),
        &[
            String::from("package.json"),
            String::from("package-lock.json"),
        ],
        "1.2.3",
        None,
    )
    .unwrap();

    // Verify commit includes both files
    let output = std::process::Command::new("git")
        .args(["show", "--name-only", "--format=%H"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let files = String::from_utf8_lossy(&output.stdout);
    assert!(files.contains("package.json"));
    assert!(files.contains("package-lock.json"));
}

#[test]
fn test_integration_tag_creation_after_multiple_commits() {
    let temp = TempDir::new().unwrap();
    init_test_repo(temp.path()).unwrap();

    // Make several commits
    for i in 1..=3 {
        let file = temp.path().join(format!("file{}.txt", i));
        fs::write(&file, format!("content {}", i)).unwrap();

        operations::commit_version_changes_in(
            temp.path(),
            &[format!("file{}.txt", i)],
            &format!("1.0.{}", i),
            None,
        )
        .unwrap();
    }

    // Create tag on latest commit
    operations::create_tag_in(temp.path(), "v1.0.3", "1.0.3").unwrap();

    // Verify tag was created and points to a commit (annotated tags have their own SHA)
    let tag_output = std::process::Command::new("git")
        .args(["rev-parse", "v1.0.3^{commit}"]) // Dereference to the commit
        .current_dir(temp.path())
        .output()
        .unwrap();

    let head_output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    assert_eq!(tag_output.stdout, head_output.stdout);
}

#[test]
fn test_integration_error_commit_nonexistent_file() {
    let temp = TempDir::new().unwrap();
    init_test_repo(temp.path()).unwrap();

    // Try to commit a file that doesn't exist
    let result = operations::commit_version_changes_in(
        temp.path(),
        &[String::from("nonexistent.txt")],
        "1.0.0",
        None,
    );

    assert!(result.is_err());
}

#[test]
fn test_integration_error_tag_in_nonexistent_repo() {
    let temp = TempDir::new().unwrap();
    // Don't initialize git repo

    let result = operations::create_tag_in(temp.path(), "v1.0.0", "1.0.0");
    assert!(result.is_err());
}

#[test]
fn test_integration_error_commit_in_nonexistent_repo() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("version.txt"), "1.0.0").unwrap();

    let result = operations::commit_version_changes_in(
        temp.path(),
        &[String::from("version.txt")],
        "1.0.0",
        None,
    );

    assert!(result.is_err());
}
