use crate::core::detector::TechnologyDetector;
use crate::core::semver::VersionBumper;
use crate::git::history::CommitAnalyzer;
use crate::updaters::factory::UpdaterFactory;
use anyhow::Result;

/// Analyze commit history and show recommended version bump
pub fn analyze_commits(project_path: &std::path::Path, verbose: bool) -> Result<()> {
    println!("🔍 Commit Analysis");
    println!("━━━━━━━━━━━━━━━━━━");

    // Get current version
    let detector = TechnologyDetector::new();
    let technology = detector.detect(project_path)?;
    let updater = UpdaterFactory::create(&technology)?;
    let current_version = updater.get_current_version(project_path)?;

    println!("📦 Current Version: {}", current_version);
    println!("🔧 Technology: {}", technology);

    // Parse current version for commit analysis
    let version_bumper = VersionBumper::new();
    let version = version_bumper.parse_version(&current_version)?;

    // Analyze commits
    let analyzer = CommitAnalyzer::new();

    match analyzer.analyze_commits_detailed(project_path, &version) {
        Ok(analysis) => {
            println!("\n📊 Commit Summary:");
            println!(
                "  📈 Total commits since last version: {}",
                analysis.total_commits
            );

            if !analysis.breaking_changes.is_empty() {
                println!("  💥 Breaking changes: {}", analysis.breaking_changes.len());
                if verbose {
                    for change in &analysis.breaking_changes {
                        let short_msg = change.lines().next().unwrap_or(change);
                        println!("    • {}", short_msg);
                    }
                }
            }

            if !analysis.features.is_empty() {
                println!("  ✨ Features: {}", analysis.features.len());
                if verbose {
                    for feature in &analysis.features {
                        let short_msg = feature.lines().next().unwrap_or(feature);
                        println!("    • {}", short_msg);
                    }
                }
            }

            if !analysis.fixes.is_empty() {
                println!("  🐛 Fixes: {}", analysis.fixes.len());
                if verbose {
                    for fix in &analysis.fixes {
                        let short_msg = fix.lines().next().unwrap_or(fix);
                        println!("    • {}", short_msg);
                    }
                }
            }

            if !analysis.other.is_empty() {
                println!("  📝 Other changes: {}", analysis.other.len());
                if verbose {
                    for other in &analysis.other {
                        let short_msg = other.lines().next().unwrap_or(other);
                        println!("    • {}", short_msg);
                    }
                }
            }

            // Show recommendation
            let recommended_bump = analysis.recommended_bump();
            let new_version =
                version_bumper.preview_bump(&current_version, &recommended_bump.to_string())?;

            println!("\n🎯 Recommendation:");
            println!("  📋 Bump Type: {}", recommended_bump);
            println!("  📦 New Version: {} → {}", current_version, new_version);

            // Show reasoning
            println!("\n💡 Reasoning:");
            if !analysis.breaking_changes.is_empty() {
                println!("  • Breaking changes detected → MAJOR bump required");
            } else if !analysis.features.is_empty() {
                println!("  • New features found → MINOR bump recommended");
            } else if !analysis.fixes.is_empty() {
                println!("  • Bug fixes found → PATCH bump recommended");
            } else {
                println!("  • No conventional commits found → PATCH bump (default)");
            }

            println!("\n🚀 To apply this recommendation:");
            println!(
                "  ./autoversion -b {}",
                recommended_bump.to_string().to_lowercase()
            );
            println!("  ./autoversion -b auto  # Automatic detection");
        }
        Err(e) => {
            println!("❌ Failed to analyze commits: {}", e);
            println!("\n💡 This might be because:");
            println!("  • Not in a git repository");
            println!("  • No commits since last version tag");
            println!("  • Repository permissions issue");
        }
    }

    Ok(())
}
