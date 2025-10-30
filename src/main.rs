use anyhow::Result;
use clap::Parser;

mod cli;
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

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Handle info-only commands first
    if args.show_info {
        return show_project_info(&args.path, args.verbose);
    }
    
    if args.analyze {
        return analyze_commits(&args.path, args.verbose);
    }
    
    // Initialize action output handler
    let mut output = ActionOutput::new();
    
    // Detect technology if auto mode
    let technology = if args.technology == "auto" {
        let detector = TechnologyDetector::new();
        detector.detect(&args.path)?
    } else {
        args.technology.clone()
    };
    
    output.set_technology(&technology);
    
    // Create appropriate updater
    let updater = UpdaterFactory::create(&technology)?;
    
    // Get current version
    let current_version = updater.get_current_version(&args.path)?;
    output.set_previous_version(&current_version);
    
    // Calculate new version
    let bumper = VersionBumper::new();
    let (new_version, bump_type) = if args.bump_type == "auto" {
        bumper.auto_bump(&current_version, &args.path)?
    } else {
        bumper.manual_bump(&current_version, &args.bump_type)?
    };
    
    output.set_version(&new_version);
    output.set_version_type(&bump_type);
    
    if !args.dry_run {
        // Update version files
        let updated_files = updater.update_version(&args.path, &new_version)?;
        output.set_files_updated(&updated_files);
        
        // Create git tag if requested
        if args.create_tag {
            let tag_name = format!("{}{}", args.tag_prefix, new_version);
            git::operations::create_tag(&tag_name, &new_version)?;
            output.set_tag_created(true);
        }
    }
    
    // Output results for GitHub Actions
    output.write_outputs()?;
    
    println!("✅ Version updated from {} to {}", current_version, new_version);
    if args.dry_run {
        println!("🔍 Dry run mode - no changes made");
    }
    
    Ok(())
}