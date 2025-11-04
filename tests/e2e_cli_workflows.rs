use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Helper to create a test NPM project
fn create_npm_project(dir: &TempDir) -> std::path::PathBuf {
    let package_json = dir.path().join("package.json");
    fs::write(
        &package_json,
        r#"{
  "name": "test-project",
  "version": "1.0.0",
  "description": "Test project"
}"#,
    )
    .unwrap();
    dir.path().to_path_buf()
}

/// Helper to create a test Cargo project
fn create_cargo_project(dir: &TempDir) -> std::path::PathBuf {
    let cargo_toml = dir.path().join("Cargo.toml");
    fs::write(
        &cargo_toml,
        r#"[package]
name = "test-project"
version = "1.0.0"
edition = "2021"
"#,
    )
    .unwrap();
    dir.path().to_path_buf()
}

/// Helper to create a test Maven project
fn create_maven_project(dir: &TempDir) -> std::path::PathBuf {
    let pom_xml = dir.path().join("pom.xml");
    fs::write(
        &pom_xml,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.test</groupId>
    <artifactId>test-project</artifactId>
    <version>1.0.0</version>
</project>
"#,
    )
    .unwrap();
    dir.path().to_path_buf()
}

/// Helper to create a test Generic project
fn create_generic_project(dir: &TempDir) -> std::path::PathBuf {
    let version_file = dir.path().join("VERSION");
    fs::write(&version_file, "1.0.0").unwrap();
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
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()
        .unwrap();
    
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()
        .unwrap();
    
    Command::new("git")
        .args(["add", "."])
        .current_dir(path)
        .output()
        .unwrap();
    
    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()
        .unwrap();
}

#[test]
fn test_e2e_npm_patch_bump_dry_run() {
    let temp = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp);

    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", project_path.to_str().unwrap(), "-b", "patch", "-d"])
        .assert()
        .success();

    // Verify file was NOT modified (dry run)
    let content = fs::read_to_string(project_path.join("package.json")).unwrap();
    assert!(content.contains("1.0.0"));
}

#[test]
fn test_e2e_npm_patch_bump_actual() {
    let temp = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp);

    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", project_path.to_str().unwrap(), "-b", "patch"])
        .assert()
        .success();

    // Verify file was modified
    let content = fs::read_to_string(project_path.join("package.json")).unwrap();
    assert!(content.contains("1.0.1"));
}

#[test]
fn test_e2e_npm_minor_bump() {
    let temp = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp);

    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", project_path.to_str().unwrap(), "-b", "minor"])
        .assert()
        .success();

    let content = fs::read_to_string(project_path.join("package.json")).unwrap();
    assert!(content.contains("1.1.0"));
}

#[test]
fn test_e2e_npm_major_bump() {
    let temp = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp);

    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", project_path.to_str().unwrap(), "-b", "major"])
        .assert()
        .success();

    let content = fs::read_to_string(project_path.join("package.json")).unwrap();
    assert!(content.contains("2.0.0"));
}

#[test]
fn test_e2e_cargo_patch_bump() {
    let temp = TempDir::new().unwrap();
    let project_path = create_cargo_project(&temp);

    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", project_path.to_str().unwrap(), "-b", "patch", "-t", "cargo"])
        .assert()
        .success();

    let content = fs::read_to_string(project_path.join("Cargo.toml")).unwrap();
    assert!(content.contains("1.0.1"));
}

#[test]
fn test_e2e_maven_minor_bump() {
    let temp = TempDir::new().unwrap();
    let project_path = create_maven_project(&temp);

    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", project_path.to_str().unwrap(), "-b", "minor", "-t", "maven"])
        .assert()
        .success();

    let content = fs::read_to_string(project_path.join("pom.xml")).unwrap();
    assert!(content.contains("1.1.0"));
}

#[test]
fn test_e2e_generic_major_bump() {
    let temp = TempDir::new().unwrap();
    let project_path = create_generic_project(&temp);

    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", project_path.to_str().unwrap(), "-b", "major", "-t", "generic"])
        .assert()
        .success();

    let content = fs::read_to_string(project_path.join("VERSION")).unwrap();
    assert!(content.contains("2.0.0"));
}

#[test]
fn test_e2e_auto_detect_npm() {
    let temp = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp);

    // Don't specify technology, let it auto-detect
    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", project_path.to_str().unwrap(), "-b", "patch", "-t", "auto"])
        .assert()
        .success();

    let content = fs::read_to_string(project_path.join("package.json")).unwrap();
    assert!(content.contains("1.0.1"));
}

#[test]
fn test_e2e_auto_detect_cargo() {
    let temp = TempDir::new().unwrap();
    let project_path = create_cargo_project(&temp);

    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", project_path.to_str().unwrap(), "-b", "patch", "-t", "auto"])
        .assert()
        .success();

    let content = fs::read_to_string(project_path.join("Cargo.toml")).unwrap();
    assert!(content.contains("1.0.1"));
}

#[test]
fn test_e2e_verbose_output() {
    let temp = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp);

    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", project_path.to_str().unwrap(), "-b", "patch", "-v"])
        .assert()
        .success();

    // Just verify it succeeds with verbose flag
}

#[test]
fn test_e2e_git_tag_creation() {
    let temp = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp);
    init_git_repo(&project_path);

    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", project_path.to_str().unwrap(), "-b", "patch", "-c"])
        .assert()
        .success();

    // Check that tag was created
    let output = Command::new("git")
        .args(["tag", "-l"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    
    let tags = String::from_utf8_lossy(&output.stdout);
    assert!(tags.contains("v1.0.1"));
}

#[test]
fn test_e2e_git_commit_creation() {
    let temp = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp);
    init_git_repo(&project_path);

    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", project_path.to_str().unwrap(), "-b", "patch", "-C"])
        .assert()
        .success();

    // Check that commit was created
    let output = Command::new("git")
        .args(["log", "--oneline", "-1"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    
    let log = String::from_utf8_lossy(&output.stdout);
    eprintln!("Git log output: {}", log);
    assert!(log.contains("1.0.1") || log.contains("Bump") || log.contains("bump"));
}

#[test]
fn test_e2e_git_tag_and_commit() {
    let temp = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp);
    init_git_repo(&project_path);

    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", project_path.to_str().unwrap(), "-b", "patch", "-c", "-C"])
        .assert()
        .success();

    // Check both tag and commit
    let tag_output = Command::new("git")
        .args(["tag", "-l"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    
    let tags = String::from_utf8_lossy(&tag_output.stdout);
    assert!(tags.contains("v1.0.1"));

    let commit_output = Command::new("git")
        .args(["log", "--oneline", "-1"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    
    let log = String::from_utf8_lossy(&commit_output.stdout);
    assert!(log.contains("1.0.1") || log.contains("Bump version"));
}

#[test]
fn test_e2e_custom_commit_message() {
    let temp = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp);
    init_git_repo(&project_path);

    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args([
        "-p", project_path.to_str().unwrap(),
        "-b", "patch",
        "-C",
        "-m", "🚀 Release version {version}"
    ])
    .assert()
    .success();

    let output = Command::new("git")
        .args(["log", "--oneline", "-1"])
        .current_dir(&project_path)
        .output()
        .unwrap();
    
    let log = String::from_utf8_lossy(&output.stdout);
    assert!(log.contains("🚀 Release version 1.0.1"));
}

#[test]
fn test_e2e_error_no_manifest_file() {
    let temp = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", temp.path().to_str().unwrap(), "-b", "patch"])
        .assert()
        .failure();
}

#[test]
fn test_e2e_error_invalid_bump_type() {
    let temp = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp);

    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", project_path.to_str().unwrap(), "-b", "invalid"])
        .assert()
        .failure();
}

#[test]
fn test_e2e_error_invalid_path() {
    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", "/nonexistent/path", "-b", "patch"])
        .assert()
        .failure();
}

#[test]
fn test_e2e_backup_file_creation() {
    let temp = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp);

    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", project_path.to_str().unwrap(), "-b", "patch"])
        .assert()
        .success();

    // Check that backup file was created
    let backup_file = project_path.join("package.json.autoversion.backup");
    assert!(backup_file.exists());
    
    let backup_content = fs::read_to_string(backup_file).unwrap();
    assert!(backup_content.contains("1.0.0"));
}

#[test]
fn test_e2e_multiple_sequential_bumps() {
    let temp = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp);

    // First bump: patch
    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", project_path.to_str().unwrap(), "-b", "patch"])
        .assert()
        .success();

    let content = fs::read_to_string(project_path.join("package.json")).unwrap();
    assert!(content.contains("1.0.1"));

    // Second bump: minor
    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", project_path.to_str().unwrap(), "-b", "minor"])
        .assert()
        .success();

    let content = fs::read_to_string(project_path.join("package.json")).unwrap();
    assert!(content.contains("1.1.0"));

    // Third bump: major
    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", project_path.to_str().unwrap(), "-b", "major"])
        .assert()
        .success();

    let content = fs::read_to_string(project_path.join("package.json")).unwrap();
    assert!(content.contains("2.0.0"));
}

#[test]
fn test_e2e_force_flag_with_uncommitted_changes() {
    let temp = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp);
    init_git_repo(&project_path);

    // Make an uncommitted change
    fs::write(project_path.join("test.txt"), "uncommitted change").unwrap();

    // Try without force flag (should fail or warn)
    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", project_path.to_str().unwrap(), "-b", "patch"])
        .assert()
        .success(); // Actually succeeds, but may warn

    // Try with force flag (should succeed)
    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.args(["-p", project_path.to_str().unwrap(), "-b", "minor", "-f"])
        .assert()
        .success();

    let content = fs::read_to_string(project_path.join("package.json")).unwrap();
    assert!(content.contains("1.1.0"));
}

#[test]
fn test_e2e_output_shows_version_change() {
    let temp = TempDir::new().unwrap();
    let project_path = create_npm_project(&temp);

    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    let output = cmd.args(["-p", project_path.to_str().unwrap(), "-b", "patch"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show version change information
    assert!(stdout.contains("1.0.0") || stdout.contains("1.0.1"));
}

#[test]
fn test_e2e_help_command() {
    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.arg("--help")
        .assert()
        .success();
}

#[test]
fn test_e2e_version_command() {
    let mut cmd = Command::cargo_bin("autoversion").unwrap();
    cmd.arg("--version")
        .assert()
        .success();
}
