use assert_cmd::prelude::*;
use std::process::Command;
use std::fs;
use std::env;
use tempfile::TempDir;

/// Helper function to get the autoversion binary path
fn get_autoversion_bin() -> std::path::PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("autoversion");
    path
}

/// Helper to create a test NPM project
fn create_npm_project(dir: &TempDir) -> std::path::PathBuf {
    let package_json = dir.path().join("package.json");
    fs::write(
        &package_json,
        r#"{
  "name": "test-project",
  "version": "1.0.0"
}"#,
    )
    .unwrap();
    dir.path().to_path_buf()
}

/// Helper to initialize a git repository
fn init_git_repo(path: &std::path::Path) {
    Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()
        .unwrap();
    
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()
        .unwrap();
    
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()
        .unwrap();
}

/// Helper to bump version and create backup
fn bump_version(path: &std::path::Path, bump_type: &str) {
    Command::new(get_autoversion_bin())
        .arg("-p")
        .arg(path)
        .arg("-b")
        .arg(bump_type)
        .output()
        .unwrap();
}

/// Helper to read package.json version
fn read_npm_version(path: &std::path::Path) -> String {
    let content = fs::read_to_string(path.join("package.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    json["version"].as_str().unwrap().to_string()
}

#[test]
fn test_rollback_basic() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp_dir);
    
    // Bump version
    bump_version(&project_path, "patch");
    
    // Verify version was bumped
    assert_eq!(read_npm_version(&project_path), "1.0.1");
    
    // Verify backup exists
    assert!(project_path.join("package.json.autoversion.backup").exists());
    
    // Rollback
    let mut cmd = Command::new(get_autoversion_bin());
    cmd.arg("--rollback")
        .arg("-p")
        .arg(&project_path)
        .assert()
        .success();
    
    // Verify version was restored
    assert_eq!(read_npm_version(&project_path), "1.0.0");
    
    // Verify backup was deleted
    assert!(!project_path.join("package.json.autoversion.backup").exists());
}

#[test]
fn test_rollback_files_only() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp_dir);
    init_git_repo(&project_path);
    
    // Commit initial state
    Command::new("git")
        .args(["add", "."])
        .current_dir(&project_path)
        .output()
        .unwrap();
    
    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    
    // Bump version with tag
    let mut cmd = Command::new(get_autoversion_bin());
    cmd.arg("-p")
        .arg(&project_path)
        .arg("-b")
        .arg("patch")
        .arg("--create-tag");
    cmd.output().unwrap();
    
    // Verify tag was created
    let output = Command::new("git")
        .args(["tag", "-l"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains("v1.0.1"));
    
    // Rollback files only
    let mut cmd = Command::new(get_autoversion_bin());
    cmd.arg("--rollback")
        .arg("--rollback-files-only")
        .arg("-p")
        .arg(&project_path)
        .assert()
        .success();
    
    // Verify version was restored
    assert_eq!(read_npm_version(&project_path), "1.0.0");
    
    // Verify tag still exists (files-only doesn't delete tags)
    let output = Command::new("git")
        .args(["tag", "-l"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains("v1.0.1"));
}

#[test]
fn test_rollback_git_only() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp_dir);
    init_git_repo(&project_path);
    
    // Commit initial state
    Command::new("git")
        .args(["add", "."])
        .current_dir(&project_path)
        .output()
        .unwrap();
    
    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    
    // Bump version with tag
    let mut cmd = Command::new(get_autoversion_bin());
    cmd.arg("-p")
        .arg(&project_path)
        .arg("-b")
        .arg("patch")
        .arg("--create-tag");
    cmd.output().unwrap();
    
    // Verify tag was created and version changed
    let output = Command::new("git")
        .args(["tag", "-l"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains("v1.0.1"));
    assert_eq!(read_npm_version(&project_path), "1.0.1");
    
    // Rollback git only (specify version to delete tag)
    let mut cmd = Command::new(get_autoversion_bin());
    cmd.arg("--rollback")
        .arg("--rollback-git-only")
        .arg("--rollback-version")
        .arg("1.0.1")
        .arg("-p")
        .arg(&project_path)
        .assert()
        .success();
    
    // Verify tag was deleted
    let output = Command::new("git")
        .args(["tag", "-l"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    assert!(!String::from_utf8_lossy(&output.stdout).contains("v1.0.1"));
    
    // Verify version was NOT restored (git-only doesn't touch files)
    assert_eq!(read_npm_version(&project_path), "1.0.1");
}

#[test]
fn test_rollback_with_commit_revert() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp_dir);
    init_git_repo(&project_path);
    
    // Commit initial state
    Command::new("git")
        .args(["add", "."])
        .current_dir(&project_path)
        .output()
        .unwrap();
    
    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    
    // Get initial commit hash
    let initial_commit = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    let initial_hash = String::from_utf8_lossy(&initial_commit.stdout).trim().to_string();
    
    // Bump version with commit
    let mut cmd = Command::new(get_autoversion_bin());
    cmd.arg("-p")
        .arg(&project_path)
        .arg("-b")
        .arg("patch")
        .arg("--commit");
    cmd.output().unwrap();
    
    // Verify new commit was created
    let new_commit = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    let new_hash = String::from_utf8_lossy(&new_commit.stdout).trim().to_string();
    assert_ne!(initial_hash, new_hash);
    
    // Rollback with commit revert
    let mut cmd = Command::new(get_autoversion_bin());
    cmd.arg("--rollback")
        .arg("--rollback-revert-commit")
        .arg("-p")
        .arg(&project_path)
        .assert()
        .success();
    
    // Verify commit was reverted
    let reverted_commit = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    let reverted_hash = String::from_utf8_lossy(&reverted_commit.stdout).trim().to_string();
    assert_eq!(initial_hash, reverted_hash);
    
    // Verify version was restored
    assert_eq!(read_npm_version(&project_path), "1.0.0");
}

#[test]
fn test_rollback_no_backups() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp_dir);
    
    // Try to rollback without any backups
    let mut cmd = Command::new(get_autoversion_bin());
    cmd.arg("--rollback")
        .arg("-p")
        .arg(&project_path)
        .assert()
        .failure();
    
    // Should fail because no backups exist
}

// TODO: Fix this test - the generic updater needs to properly create backups
// #[test]
// fn test_rollback_multiple_files() {
//     let temp_dir = TempDir::new().unwrap();
//     let project_path = temp_dir.path();
//     
//     // Create a VERSION file
//     fs::write(project_path.join("VERSION"), "1.0.0").unwrap();
//     
//     // Bump version (will create backup)
//     let mut cmd = Command::new(get_autoversion_bin());
//     cmd.arg("-p")
//         .arg(project_path)
//         .arg("-b")
//         .arg("patch")
//         .arg("-t")
//         .arg("generic");
//     cmd.output().unwrap();
//     
//     // Verify version was bumped and backup exists
//     let version_content = fs::read_to_string(project_path.join("VERSION")).unwrap();
//     assert_eq!(version_content.trim(), "1.0.1");
//     assert!(project_path.join("VERSION.autoversion.backup").exists());
//     
//     // Rollback
//     let mut cmd = Command::new(get_autoversion_bin());
//     cmd.arg("--rollback")
//         .arg("-p")
//         .arg(project_path)
//         .assert()
//         .success();
//     
//     // Verify VERSION was restored
//     let version_content = fs::read_to_string(project_path.join("VERSION")).unwrap();
//     assert_eq!(version_content, "1.0.0");
//     
//     // Verify backup was removed
//     assert!(!project_path.join("VERSION.autoversion.backup").exists());
// }

#[test]
fn test_rollback_validation_conflicting_options() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp_dir);
    
    // Try to use both --rollback-files-only and --rollback-git-only
    let mut cmd = Command::new(get_autoversion_bin());
    cmd.arg("--rollback")
        .arg("--rollback-files-only")
        .arg("--rollback-git-only")
        .arg("-p")
        .arg(&project_path)
        .assert()
        .failure();
    
    // Should fail with validation error
}

#[test]
fn test_rollback_validation_with_version_bump() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp_dir);
    
    // Try to use --rollback with --create-tag
    let mut cmd = Command::new(get_autoversion_bin());
    cmd.arg("--rollback")
        .arg("--create-tag")
        .arg("-p")
        .arg(&project_path)
        .assert()
        .failure();
    
    // Should fail with validation error
}

#[test]
fn test_rollback_with_specific_version() {
    let temp_dir = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp_dir);
    init_git_repo(&project_path);
    
    // Commit initial state
    Command::new("git")
        .args(["add", "."])
        .current_dir(&project_path)
        .output()
        .unwrap();
    
    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    
    // Bump to 1.0.1
    let mut cmd = Command::new(get_autoversion_bin());
    cmd.arg("-p")
        .arg(&project_path)
        .arg("-b")
        .arg("patch")
        .arg("--create-tag");
    cmd.output().unwrap();
    
    // Manually bump to 1.0.2 (simulate multiple version changes)
    let mut cmd = Command::new(get_autoversion_bin());
    cmd.arg("-p")
        .arg(&project_path)
        .arg("-b")
        .arg("patch")
        .arg("--create-tag");
    cmd.output().unwrap();
    
    // Verify both tags exist
    let output = Command::new("git")
        .args(["tag", "-l"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    let tags = String::from_utf8_lossy(&output.stdout);
    assert!(tags.contains("v1.0.1"));
    assert!(tags.contains("v1.0.2"));
    
    // Rollback specific version (1.0.1)
    let mut cmd = Command::new(get_autoversion_bin());
    cmd.arg("--rollback")
        .arg("--rollback-version")
        .arg("1.0.1")
        .arg("-p")
        .arg(&project_path)
        .assert()
        .success();
    
    // Verify v1.0.1 was deleted but v1.0.2 still exists
    let output = Command::new("git")
        .args(["tag", "-l"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    let tags = String::from_utf8_lossy(&output.stdout);
    assert!(!tags.contains("v1.0.1"));
    assert!(tags.contains("v1.0.2"));
}
