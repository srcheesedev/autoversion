use anyhow::Result;
use crate::core::detector::TechnologyDetector;
use crate::updaters::factory::UpdaterFactory;

/// Display project information without making changes
pub fn show_project_info(project_path: &std::path::Path, verbose: bool) -> Result<()> {
    println!("📋 Project Information");
    println!("━━━━━━━━━━━━━━━━━━━━━━");
    
    // Show project path
    println!("📁 Path: {}", project_path.display());
    
    // Detect technology
    let detector = TechnologyDetector::new();
    match detector.detect(project_path) {
        Ok(technology) => {
            println!("🔧 Technology: {}", technology);
            
            // Get current version
            match UpdaterFactory::create(&technology) {
                Ok(updater) => {
                    match updater.get_current_version(project_path) {
                        Ok(version) => println!("📦 Current Version: {}", version),
                        Err(e) => println!("❌ Version Error: {}", e),
                    }
                    
                    // Get primary file
                    match updater.get_primary_file(project_path) {
                        Ok(file) => println!("📄 Primary File: {}", file.display()),
                        Err(e) => {
                            if verbose {
                                println!("⚠️  Primary File Error: {}", e);
                            }
                        }
                    }
                }
                Err(e) => println!("❌ Updater Error: {}", e),
            }
            
            // Show all compatible technologies if verbose
            if verbose {
                println!("\n🔍 Detection Details:");
                let all_results = detector.detect_all(project_path)?;
                for result in all_results {
                    println!("  • {} (confidence: {:.1}%)", 
                        result.technology, 
                        result.confidence * 100.0
                    );
                }
            }
        }
        Err(e) => {
            println!("❌ Technology Detection Failed: {}", e);
            if verbose {
                println!("\n🔍 Searching for supported files:");
                let common_files = [
                    "package.json", "Cargo.toml", "pom.xml", 
                    "pyproject.toml", "setup.py", "VERSION", "version.txt"
                ];
                for file in &common_files {
                    let file_path = project_path.join(file);
                    if file_path.exists() {
                        println!("  ✅ Found: {}", file);
                    } else {
                        println!("  ❌ Missing: {}", file);
                    }
                }
            }
        }
    }
    
    // Git repository info
    if verbose {
        println!("\n🔗 Git Information:");
        match git2::Repository::open(project_path) {
            Ok(repo) => {
                // Get current branch
                if let Ok(head) = repo.head() {
                    if let Some(branch_name) = head.shorthand() {
                        println!("  🌿 Branch: {}", branch_name);
                    }
                }
                
                // Check for uncommitted changes
                match repo.statuses(None) {
                    Ok(statuses) => {
                        if statuses.is_empty() {
                            println!("  ✅ Repository: Clean");
                        } else {
                            println!("  ⚠️  Repository: {} uncommitted changes", statuses.len());
                        }
                    }
                    Err(_) => println!("  ❓ Repository status: Unknown"),
                }
                
                // Show recent tags
                if let Ok(tag_names) = repo.tag_names(None) {
                    let tags: Vec<&str> = tag_names.iter().flatten().collect();
                    if !tags.is_empty() {
                        let recent_tags: Vec<&str> = tags.iter().rev().take(3).cloned().collect();
                        println!("  🏷️  Recent tags: {}", recent_tags.join(", "));
                    } else {
                        println!("  🏷️  Recent tags: None");
                    }
                }
            }
            Err(_) => println!("  ❌ Not a git repository"),
        }
    }
    
    println!("\n✨ Use --analyze to see commit-based version recommendations");
    
    Ok(())
}