use clap::Parser;
use std::path::PathBuf;

/// Universal semantic versioning automation for any technology stack
#[derive(Parser, Debug)]
#[command(name = "autoversion")]
#[command(author = "srcheesedev")]
#[command(version = "0.2.1")]
#[command(about = "Automatically manage semantic versioning for any project")]
#[command(long_about = None)]
pub struct Args {
    /// Type of version bump to perform
    #[arg(
        short = 'b',
        long = "bump-type",
        default_value = "auto",
        help = "Version bump type: auto, major, minor, patch"
    )]
    pub bump_type: String,

    /// Technology type to use for version management
    #[arg(
        short = 't',
        long = "technology",
        default_value = "auto",
        help = "Technology type: auto, npm, cargo, maven, python, generic"
    )]
    pub technology: String,

    /// Whether to create a git tag for the new version
    #[arg(
        short = 'c',
        long = "create-tag",
        action = clap::ArgAction::SetTrue,
        help = "Create git tag for the new version"
    )]
    pub create_tag: bool,

    /// Prefix for git tags
    #[arg(
        long = "tag-prefix",
        default_value = "v",
        help = "Prefix for git tags (e.g., 'v' creates 'v1.0.0')"
    )]
    pub tag_prefix: String,

    /// Dry run mode - show what would be done without making changes
    #[arg(
        short = 'd',
        long = "dry-run",
        action = clap::ArgAction::SetTrue,
        help = "Preview changes without modifying files"
    )]
    pub dry_run: bool,

    /// Project path (defaults to current directory)
    #[arg(
        short = 'p',
        long = "path",
        default_value = ".",
        help = "Path to the project directory"
    )]
    pub path: PathBuf,

    /// Force version bump even if repository has uncommitted changes
    #[arg(
        short = 'f',
        long = "force",
        action = clap::ArgAction::SetTrue,
        help = "Force version bump even with uncommitted changes"
    )]
    pub force: bool,

    /// Commit version changes to git
    #[arg(
        short = 'C',
        long = "commit",
        action = clap::ArgAction::SetTrue,
        help = "Commit version changes to git"
    )]
    pub commit: bool,

    /// Custom commit message (only used with --commit)
    #[arg(
        short = 'm',
        long = "commit-message",
        help = "Custom commit message template (use {version} placeholder)"
    )]
    pub commit_message: Option<String>,

    /// Verbose output
    #[arg(
        short = 'v',
        long = "verbose",
        action = clap::ArgAction::SetTrue,
        help = "Enable verbose output"
    )]
    pub verbose: bool,

    /// Output format for machine consumption
    #[arg(
        short = 'o',
        long = "output",
        default_value = "human",
        help = "Output format: human, json, github-actions"
    )]
    pub output_format: String,

    /// Show detected technology and version without making changes
    #[arg(
        short = 'i',
        long = "show-info",
        action = clap::ArgAction::SetTrue,
        help = "Show project info and exit"
    )]
    pub show_info: bool,

    /// Analyze commit history and show recommended bump
    #[arg(
        short = 'a',
        long = "analyze",
        action = clap::ArgAction::SetTrue,
        help = "Analyze commits and show recommended version bump"
    )]
    pub analyze: bool,

    /// Rollback the last version bump
    #[arg(
        short = 'r',
        long = "rollback",
        action = clap::ArgAction::SetTrue,
        help = "Rollback the last version bump"
    )]
    pub rollback: bool,

    /// Specific version to rollback (for tag deletion)
    #[arg(
        long = "rollback-version",
        help = "Specific version to rollback (e.g., '1.2.3')"
    )]
    pub rollback_version: Option<String>,

    /// Only restore files when rolling back, skip git operations
    #[arg(
        long = "rollback-files-only",
        action = clap::ArgAction::SetTrue,
        help = "Only restore files from backups, skip git operations"
    )]
    pub rollback_files_only: bool,

    /// Only perform git operations when rolling back, skip file restoration
    #[arg(
        long = "rollback-git-only",
        action = clap::ArgAction::SetTrue,
        help = "Only perform git operations, skip file restoration"
    )]
    pub rollback_git_only: bool,

    /// Revert the last commit when rolling back
    #[arg(
        long = "rollback-revert-commit",
        action = clap::ArgAction::SetTrue,
        help = "Revert the last git commit"
    )]
    pub rollback_revert_commit: bool,
}

impl Args {
    /// Validate arguments and return errors for invalid combinations
    pub fn validate(&self) -> Result<(), String> {
        // Validate bump type
        if !["auto", "major", "minor", "patch"].contains(&self.bump_type.as_str()) {
            return Err(format!(
                "Invalid bump type '{}'. Valid options: auto, major, minor, patch",
                self.bump_type
            ));
        }

        // Validate technology
        if !["auto", "npm", "cargo", "maven", "python", "generic"].contains(&self.technology.as_str()) {
            return Err(format!(
                "Invalid technology '{}'. Valid options: auto, npm, cargo, maven, python, generic",
                self.technology
            ));
        }

        // Validate output format
        if !["human", "json", "github-actions"].contains(&self.output_format.as_str()) {
            return Err(format!(
                "Invalid output format '{}'. Valid options: human, json, github-actions",
                self.output_format
            ));
        }

        // Validate path exists
        if !self.path.exists() {
            return Err(format!(
                "Project path does not exist: {}",
                self.path.display()
            ));
        }

        // Validate path is directory
        if !self.path.is_dir() {
            return Err(format!(
                "Project path is not a directory: {}",
                self.path.display()
            ));
        }

        // Validate commit message only used with commit
        if self.commit_message.is_some() && !self.commit {
            return Err("--commit-message can only be used with --commit".to_string());
        }

        // Validate conflicting options
        if self.show_info && self.analyze {
            return Err("Cannot use --show-info and --analyze together".to_string());
        }

        if (self.show_info || self.analyze) && self.dry_run {
            return Err("--dry-run is redundant with --show-info or --analyze".to_string());
        }

        // Validate rollback options
        if self.rollback {
            if self.rollback_files_only && self.rollback_git_only {
                return Err("Cannot use --rollback-files-only and --rollback-git-only together".to_string());
            }

            // Rollback mode conflicts with version bump operations
            if self.create_tag || self.commit {
                return Err("Cannot use --create-tag or --commit with --rollback".to_string());
            }

            if !self.dry_run && self.bump_type != "auto" {
                return Err("Cannot specify --bump-type with --rollback".to_string());
            }
        }

        // Validate rollback-specific options only used with rollback
        if !self.rollback {
            if self.rollback_version.is_some() {
                return Err("--rollback-version can only be used with --rollback".to_string());
            }
            if self.rollback_files_only {
                return Err("--rollback-files-only can only be used with --rollback".to_string());
            }
            if self.rollback_git_only {
                return Err("--rollback-git-only can only be used with --rollback".to_string());
            }
            if self.rollback_revert_commit {
                return Err("--rollback-revert-commit can only be used with --rollback".to_string());
            }
        }

        Ok(())
    }

    /// Get expanded path (resolve relative paths)
    pub fn get_absolute_path(&self) -> std::io::Result<PathBuf> {
        self.path.canonicalize()
    }

    /// Check if we're in GitHub Actions environment
    pub fn is_github_actions(&self) -> bool {
        std::env::var("GITHUB_ACTIONS").is_ok() || self.output_format == "github-actions"
    }

    /// Get default output format based on environment
    pub fn get_effective_output_format(&self) -> &str {
        if self.is_github_actions() && self.output_format == "human" {
            "github-actions"
        } else {
            &self.output_format
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use tempfile::TempDir;
    use std::sync::{Mutex, OnceLock};

    // Global lock to serialize tests that mutate environment variables
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    #[test]
    fn test_default_args() {
        let args = Args::parse_from(&["autoversion"]);
        
        assert_eq!(args.bump_type, "auto");
        assert_eq!(args.technology, "auto");
        assert!(!args.create_tag); // false by default because flag wasn't specified
        assert_eq!(args.tag_prefix, "v");
        assert!(!args.dry_run);
        assert_eq!(args.path, PathBuf::from("."));
        assert!(!args.force);
        assert!(!args.commit);
        assert!(args.commit_message.is_none());
        assert!(!args.verbose);
        assert_eq!(args.output_format, "human");
        assert!(!args.show_info);
        assert!(!args.analyze);
    }

    #[test]
    fn test_custom_args() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap();
        
        let args = Args::parse_from(&[
            "autoversion",
            "--bump-type", "major",
            "--technology", "npm",
            "--tag-prefix", "release-",
            "--dry-run",
            "--path", path,
            "--force",
            "--commit",
            "--commit-message", "Release {version}",
            "--verbose",
            "--output", "json",
        ]);
        
        assert_eq!(args.bump_type, "major");
        assert_eq!(args.technology, "npm");
        assert!(!args.create_tag); // false because --create-tag was not specified
        assert_eq!(args.tag_prefix, "release-");
        assert!(args.dry_run);
        assert_eq!(args.path, PathBuf::from(path));
        assert!(args.force);
        assert!(args.commit);
        assert_eq!(args.commit_message, Some("Release {version}".to_string()));
        assert!(args.verbose);
        assert_eq!(args.output_format, "json");
    }

    #[test]
    fn test_create_tag_flag() {
        let args = Args::parse_from(&[
            "autoversion",
            "--create-tag",
            "--tag-prefix", "v",
        ]);
        
        assert!(args.create_tag); // true because --create-tag was specified
        assert_eq!(args.tag_prefix, "v");
    }

    #[test]
    fn test_validation_invalid_bump_type() {
        let mut args = Args::parse_from(&["autoversion"]);
        args.bump_type = "invalid".to_string();
        
        assert!(args.validate().is_err());
        assert!(args.validate().unwrap_err().contains("Invalid bump type"));
    }

    #[test]
    fn test_validation_invalid_technology() {
        let mut args = Args::parse_from(&["autoversion"]);
        args.technology = "invalid".to_string();
        
        assert!(args.validate().is_err());
        assert!(args.validate().unwrap_err().contains("Invalid technology"));
    }

    #[test]
    fn test_validation_invalid_output_format() {
        let mut args = Args::parse_from(&["autoversion"]);
        args.output_format = "invalid".to_string();
        
        assert!(args.validate().is_err());
        assert!(args.validate().unwrap_err().contains("Invalid output format"));
    }

    #[test]
    fn test_validation_nonexistent_path() {
        let mut args = Args::parse_from(&["autoversion"]);
        args.path = PathBuf::from("/nonexistent/path");
        
        assert!(args.validate().is_err());
        assert!(args.validate().unwrap_err().contains("does not exist"));
    }

    #[test]
    fn test_validation_commit_message_without_commit() {
        let mut args = Args::parse_from(&["autoversion"]);
        args.commit_message = Some("test".to_string());
        args.commit = false;
        
        assert!(args.validate().is_err());
        assert!(args.validate().unwrap_err().contains("--commit-message can only be used with --commit"));
    }

    #[test]
    fn test_validation_conflicting_options() {
        let mut args = Args::parse_from(&["autoversion"]);
        args.show_info = true;
        args.analyze = true;
        
        assert!(args.validate().is_err());
        assert!(args.validate().unwrap_err().contains("Cannot use --show-info and --analyze together"));
    }

    #[test]
    fn test_github_actions_detection() {
        let _lock = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|e| e.into_inner());

        // Save previous value and restore at the end
        let prev = std::env::var("GITHUB_ACTIONS").ok();
        
        // Test environment variable detection
        std::env::set_var("GITHUB_ACTIONS", "true");
        let args = Args::parse_from(&["autoversion"]);
        assert!(args.is_github_actions());

        // Clean up
        if let Some(val) = prev {
            std::env::set_var("GITHUB_ACTIONS", val);
        } else {
            std::env::remove_var("GITHUB_ACTIONS");
        }

        // Test explicit format (without environment variable)
        let mut args = Args::parse_from(&["autoversion"]);
        args.output_format = "github-actions".to_string();
        assert!(args.is_github_actions());
    }

    #[test]
    fn test_effective_output_format() {
        let _lock = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|e| e.into_inner());

        // Save previous value
        let prev = std::env::var("GITHUB_ACTIONS").ok();

        // Test GitHub Actions environment detection
        std::env::set_var("GITHUB_ACTIONS", "true");
        let args = Args::parse_from(&["autoversion"]);
        assert_eq!(args.get_effective_output_format(), "github-actions");

        // Test explicit format overrides auto-detection
        std::env::set_var("GITHUB_ACTIONS", "true");
        let args = Args::parse_from(&["autoversion", "--output", "json"]);
        assert_eq!(args.get_effective_output_format(), "json");

        // Remove GitHub Actions env var for remaining tests
        std::env::remove_var("GITHUB_ACTIONS");

        // Test normal format without GitHub Actions
        let args = Args::parse_from(&["autoversion", "--output", "json"]);
        assert_eq!(args.get_effective_output_format(), "json");

        // Test default format without GitHub Actions
        let args = Args::parse_from(&["autoversion"]);
        assert_eq!(args.get_effective_output_format(), "human");

        // Restore previous value
        if let Some(val) = prev {
            std::env::set_var("GITHUB_ACTIONS", val);
        }
    }
}