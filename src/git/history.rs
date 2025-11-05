use anyhow::{anyhow, Result};
use git2::{Oid, Repository};
use regex::Regex;
use semver::Version;
use std::path::Path;

use crate::constants::DEFAULT_TAG_PREFIX;
use crate::core::semver::BumpType;

/// Analyzes git commit history to determine appropriate version bump
pub struct CommitAnalyzer {
    breaking_pattern: Regex,
    feature_pattern: Regex,
    fix_pattern: Regex,
}

impl CommitAnalyzer {
    pub fn new() -> Self {
        Self {
            // Conventional commits patterns
            breaking_pattern: Regex::new(r"(?i)^[^!]*!:|BREAKING CHANGE:").unwrap(),
            feature_pattern: Regex::new(r"(?i)^feat(\([^)]*\))?:").unwrap(),
            fix_pattern: Regex::new(r"(?i)^fix(\([^)]*\))?:").unwrap(),
        }
    }

    /// Analyze commits since the last version tag to determine bump type
    pub fn analyze_commits_since_version(
        &self,
        repo_path: &Path,
        current_version: &Version,
    ) -> Result<BumpType> {
        let repo = Repository::open(repo_path).map_err(|e| {
            anyhow!(
                "Failed to open git repository at {}: {}",
                repo_path.display(),
                e
            )
        })?;

        // Try to find the tag for current version
        let version_tag = format!("{}{}", DEFAULT_TAG_PREFIX, current_version);
        let tag_commit = self.find_tag_commit(&repo, &version_tag)?;

        // Get commits since the tag (or all commits if no tag found)
        let commits = self.get_commits_since(&repo, tag_commit)?;

        if commits.is_empty() {
            return Ok(BumpType::Patch); // Default to patch if no commits
        }

        // Analyze commit messages
        let mut has_breaking = false;
        let mut has_features = false;
        let mut has_fixes = false;

        for commit_message in commits {
            if self.is_breaking_change(&commit_message) {
                has_breaking = true;
                break; // Breaking changes take priority
            } else if self.is_feature(&commit_message) {
                has_features = true;
            } else if self.is_fix(&commit_message) {
                has_fixes = true;
            }
        }

        // Determine bump type based on analysis
        if has_breaking {
            Ok(BumpType::Major)
        } else if has_features {
            Ok(BumpType::Minor)
        } else if has_fixes {
            Ok(BumpType::Patch)
        } else {
            // Default to patch for other changes
            Ok(BumpType::Patch)
        }
    }

    /// Find the commit hash for a given tag
    fn find_tag_commit(&self, repo: &Repository, tag_name: &str) -> Result<Option<Oid>> {
        match repo.find_reference(&format!("refs/tags/{}", tag_name)) {
            Ok(tag_ref) => {
                let tag_oid = tag_ref
                    .target()
                    .ok_or_else(|| anyhow!("Tag reference has no target"))?;

                // Handle both lightweight and annotated tags
                if let Ok(tag_obj) = repo.find_tag(tag_oid) {
                    // Annotated tag - get the target commit
                    Ok(Some(tag_obj.target_id()))
                } else {
                    // Lightweight tag - the OID is the commit
                    Ok(Some(tag_oid))
                }
            }
            Err(_) => {
                // Tag not found - this is okay, we'll analyze all commits
                Ok(None)
            }
        }
    }

    /// Get commit messages since a specific commit (or all if None)
    fn get_commits_since(
        &self,
        repo: &Repository,
        since_commit: Option<Oid>,
    ) -> Result<Vec<String>> {
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;

        let mut commits = Vec::new();

        if let Some(since_oid) = since_commit {
            // Collect commits until we reach the since commit
            for oid in revwalk {
                let oid = oid?;

                if oid == since_oid {
                    break; // Stop when we reach the since commit
                }

                let commit = repo.find_commit(oid)?;
                if let Some(message) = commit.message() {
                    commits.push(message.to_string());
                }
            }
        } else {
            // Include all commits if no since commit specified
            for oid in revwalk {
                let oid = oid?;
                let commit = repo.find_commit(oid)?;
                if let Some(message) = commit.message() {
                    commits.push(message.to_string());
                }
            }
        }

        Ok(commits)
    }

    /// Check if commit message indicates a breaking change
    fn is_breaking_change(&self, message: &str) -> bool {
        self.breaking_pattern.is_match(message)
    }

    /// Check if commit message indicates a new feature
    fn is_feature(&self, message: &str) -> bool {
        self.feature_pattern.is_match(message)
    }

    /// Check if commit message indicates a bug fix
    fn is_fix(&self, message: &str) -> bool {
        self.fix_pattern.is_match(message)
    }

    /// Get a summary of commit analysis for debugging
    pub fn analyze_commits_detailed(
        &self,
        repo_path: &Path,
        current_version: &Version,
    ) -> Result<CommitAnalysis> {
        let repo = Repository::open(repo_path)?;
        let version_tag = format!("{}{}", DEFAULT_TAG_PREFIX, current_version);
        let tag_commit = self.find_tag_commit(&repo, &version_tag)?;
        let commits = self.get_commits_since(&repo, tag_commit)?;

        let mut analysis = CommitAnalysis {
            total_commits: commits.len(),
            breaking_changes: Vec::new(),
            features: Vec::new(),
            fixes: Vec::new(),
            other: Vec::new(),
        };

        for message in commits {
            if self.is_breaking_change(&message) {
                analysis.breaking_changes.push(message);
            } else if self.is_feature(&message) {
                analysis.features.push(message);
            } else if self.is_fix(&message) {
                analysis.fixes.push(message);
            } else {
                analysis.other.push(message);
            }
        }

        Ok(analysis)
    }
}

impl Default for CommitAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Detailed analysis of commits
#[derive(Debug)]
pub struct CommitAnalysis {
    pub total_commits: usize,
    pub breaking_changes: Vec<String>,
    pub features: Vec<String>,
    pub fixes: Vec<String>,
    pub other: Vec<String>,
}

impl CommitAnalysis {
    pub fn recommended_bump(&self) -> BumpType {
        if !self.breaking_changes.is_empty() {
            BumpType::Major
        } else if !self.features.is_empty() {
            BumpType::Minor
        } else if !self.fixes.is_empty() {
            BumpType::Patch
        } else {
            BumpType::Patch
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_patterns() {
        let analyzer = CommitAnalyzer::new();

        // Breaking changes
        assert!(analyzer.is_breaking_change("feat!: breaking change"));
        assert!(analyzer.is_breaking_change("fix!: another breaking change"));
        assert!(analyzer.is_breaking_change("feat: something\n\nBREAKING CHANGE: details"));

        // Features
        assert!(analyzer.is_feature("feat: add new functionality"));
        assert!(analyzer.is_feature("feat(scope): add scoped feature"));
        assert!(analyzer.is_feature("FEAT: uppercase variant"));

        // Fixes
        assert!(analyzer.is_fix("fix: resolve bug"));
        assert!(analyzer.is_fix("fix(scope): scoped fix"));
        assert!(analyzer.is_fix("FIX: uppercase fix"));

        // Non-matching
        assert!(!analyzer.is_feature("chore: update dependencies"));
        assert!(!analyzer.is_fix("docs: update readme"));
        assert!(!analyzer.is_breaking_change("feat: normal feature"));
    }

    #[test]
    fn test_commit_analysis_recommendation() {
        let mut analysis = CommitAnalysis {
            total_commits: 3,
            breaking_changes: vec!["feat!: breaking".to_string()],
            features: vec!["feat: new feature".to_string()],
            fixes: vec!["fix: bug".to_string()],
            other: vec![],
        };

        assert_eq!(analysis.recommended_bump(), BumpType::Major);

        analysis.breaking_changes.clear();
        assert_eq!(analysis.recommended_bump(), BumpType::Minor);

        analysis.features.clear();
        assert_eq!(analysis.recommended_bump(), BumpType::Patch);

        analysis.fixes.clear();
        assert_eq!(analysis.recommended_bump(), BumpType::Patch);
    }
}
