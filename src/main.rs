#![allow(dead_code)]

use anyhow::Result;
use clap::Parser;
use std::path::Path;

mod cli;
mod constants;
mod core;
mod git;
mod updaters;
mod utils;

use cli::args::Args;
use cli::output::ActionOutput;
use cli::info::show_project_info;
use cli::analyze::analyze_commits;
use core::detector::TechnologyDetector;
use core::semver::VersionBumper;
use updaters::factory::UpdaterFactory;
use utils::security::validate_project_path;

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Handle info-only commands
    if args.show_info {
        return show_project_info(&args.path, args.verbose);
    }
    
    if args.analyze {
        return analyze_commits(&args.path, args.verbose);
    }
    
    // Validate and execute version bump
    let validated_path = validate_project_path(&args.path)?;
    execute_version_bump(&args, &validated_path)
}

/// Validates and detects the technology for the project
///
/// # Arguments
///
/// * `technology_arg` - Technology string from CLI ("auto" for detection)
/// * `project_path` - Validated project path
///
/// # Returns
///
/// Detected or specified technology string
fn detect_technology(technology_arg: &str, project_path: &Path) -> Result<String> {
    if technology_arg == "auto" {
        let detector = TechnologyDetector::new();
        detector.detect(project_path)
    } else {
        Ok(technology_arg.to_string())
    }
}

/// Calculates the new version based on bump type
///
/// # Arguments
///
/// * `current_version` - Current version string
/// * `bump_type` - Bump type ("auto", "major", "minor", "patch")
/// * `project_path` - Project path for auto bump analysis
///
/// # Returns
///
/// Tuple of (new_version, bump_type_used)
fn calculate_new_version(
    current_version: &str,
    bump_type: &str,
    project_path: &Path,
) -> Result<(String, String)> {
    let bumper = VersionBumper::new();
    if bump_type == "auto" {
        bumper.auto_bump(current_version, project_path)
    } else {
        bumper.manual_bump(current_version, bump_type)
    }
}

/// Executes git operations (tagging and committing)
///
/// # Arguments
///
/// * `args` - CLI arguments
/// * `project_path` - Validated project path
/// * `new_version` - New version string
/// * `output` - Output handler for GitHub Actions
fn handle_git_operations(
    args: &Args,
    project_path: &Path,
    new_version: &str,
    output: &mut ActionOutput,
) -> Result<()> {
    if args.create_tag {
        let tag_name = format!("{}{}", args.tag_prefix, new_version);
        git::operations::create_tag_in(project_path, &tag_name, new_version)?;
        output.set_tag_created(true);
    }
    Ok(())
}

/// Main version bump execution logic
///
/// Orchestrates the version bump process:
/// 1. Detect technology
/// 2. Get current version
/// 3. Calculate new version
/// 4. Update files (if not dry run)
/// 5. Handle git operations
/// 6. Write outputs
fn execute_version_bump(args: &Args, project_path: &Path) -> Result<()> {
    let mut output = ActionOutput::new();
    
    // Detect technology
    let technology = detect_technology(&args.technology, project_path)?;
    output.set_technology(&technology);
    output.set_tag_prefix(&args.tag_prefix);
    
    // Create updater and get current version
    let updater = UpdaterFactory::create(&technology)?;
    let current_version = updater.get_current_version(project_path)?;
    output.set_previous_version(&current_version);
    
    // Calculate new version
    let (new_version, bump_type) = calculate_new_version(
        &current_version,
        &args.bump_type,
        project_path,
    )?;
    output.set_version(&new_version);
    output.set_version_type(&bump_type);
    
    // Execute updates if not dry run
    if !args.dry_run {
        let updated_files = updater.update_version(project_path, &new_version)?;
        output.set_files_updated(&updated_files);
        handle_git_operations(args, project_path, &new_version, &mut output)?;
    }
    
    // Write outputs and display results
    output.write_outputs()?;
    print_results(&current_version, &new_version, args.dry_run);
    
    Ok(())
}

/// Prints the results of the version bump operation
fn print_results(current_version: &str, new_version: &str, dry_run: bool) {
    println!("✅ Version updated from {} to {}", current_version, new_version);
    if dry_run {
        println!("🔍 Dry run mode - no changes made");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_detect_technology_auto() -> Result<()> {
        let temp = TempDir::new()?;
        let project_path = temp.path();
        
        // Create a package.json for NPM detection
        fs::write(project_path.join("package.json"), r#"{"version": "1.0.0"}"#)?;
        
        let tech = detect_technology("auto", project_path)?;
        assert_eq!(tech, "npm");
        
        Ok(())
    }

    #[test]
    fn test_detect_technology_explicit() -> Result<()> {
        let temp = TempDir::new()?;
        let project_path = temp.path();
        
        let tech = detect_technology("cargo", project_path)?;
        assert_eq!(tech, "cargo");
        
        Ok(())
    }

    #[test]
    fn test_calculate_new_version_manual() -> Result<()> {
        let temp = TempDir::new()?;
        let project_path = temp.path();
        
        let (new_version, bump_type) = calculate_new_version("1.2.3", "patch", project_path)?;
        assert_eq!(new_version, "1.2.4");
        assert_eq!(bump_type, "patch");
        
        Ok(())
    }

    #[test]
    fn test_calculate_new_version_major() -> Result<()> {
        let temp = TempDir::new()?;
        let project_path = temp.path();
        
        let (new_version, bump_type) = calculate_new_version("1.2.3", "major", project_path)?;
        assert_eq!(new_version, "2.0.0");
        assert_eq!(bump_type, "major");
        
        Ok(())
    }

    #[test]
    fn test_calculate_new_version_minor() -> Result<()> {
        let temp = TempDir::new()?;
        let project_path = temp.path();
        
        let (new_version, bump_type) = calculate_new_version("1.2.3", "minor", project_path)?;
        assert_eq!(new_version, "1.3.0");
        assert_eq!(bump_type, "minor");
        
        Ok(())
    }
}