//! Application-wide constants
//!
//! This module centralizes all magic strings, numbers, and configuration constants
//! used throughout the application. This follows the clean code principle of
//! eliminating magic values and providing a single source of truth.

/// Default prefix for git tags (e.g., "v1.2.3")
pub const DEFAULT_TAG_PREFIX: &str = "v";

/// File extension for backup files created before version updates
pub const BACKUP_FILE_EXTENSION: &str = ".autoversion.backup";

/// Default version files searched by the Generic updater, in priority order
pub const VERSION_FILES: &[&str] = &[
    "VERSION",
    "version.txt",
    ".version",
    "version",
    "VERSION.txt",
];

/// Technology-specific manifest files
pub mod manifests {
    /// NPM package manifest
    pub const NPM_PACKAGE_JSON: &str = "package.json";
    
    /// NPM lock file
    pub const NPM_PACKAGE_LOCK: &str = "package-lock.json";
    
    /// Rust package manifest
    pub const CARGO_TOML: &str = "Cargo.toml";
    
    /// Rust lock file
    pub const CARGO_LOCK: &str = "Cargo.lock";
    
    /// Maven POM file
    pub const MAVEN_POM: &str = "pom.xml";
    
    /// Python Poetry/PEP 621 manifest
    pub const PYTHON_PYPROJECT: &str = "pyproject.toml";
    
    /// Python setuptools manifest
    pub const PYTHON_SETUP: &str = "setup.py";
}

/// Git-related constants
pub mod git {
    /// Default commit message template when tagging a version
    pub const DEFAULT_TAG_MESSAGE_PREFIX: &str = "Release version";
    
    /// Default commit message when updating version files
    pub const DEFAULT_COMMIT_MESSAGE_PREFIX: &str = "Bump version to";
}

/// Regex patterns for version detection
pub mod patterns {
    /// Semantic version pattern: MAJOR.MINOR.PATCH with optional pre-release and build metadata
    /// Matches: 1.2.3, 0.1.0-alpha, 2.0.0-rc.1+build.123
    pub const SEMVER_PATTERN: &str = r"^\d+\.\d+\.\d+(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?(\\+[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$";
    
    /// Version with optional 'v' prefix: v1.2.3 or 1.2.3
    pub const VERSION_WITH_PREFIX: &str = r"^v?\d+\.\d+\.\d+";
    
    /// Maven POM version tag pattern (XML)
    pub const MAVEN_VERSION_TAG: &str = r"<version>(.*?)</version>";
}

/// Error messages and templates
pub mod errors {
    /// Error message when no version file is found
    pub const NO_VERSION_FILE: &str = "No version file found in project";
    
    /// Error message when version format is invalid
    pub const INVALID_VERSION_FORMAT: &str = "Invalid semantic version format";
    
    /// Error message when git repository is not found
    pub const NO_GIT_REPO: &str = "Not a git repository";
}

/// Configuration defaults
pub mod defaults {
    /// Maximum file size to read for version detection (10 MB)
    pub const MAX_FILE_SIZE_BYTES: u64 = 10 * 1024 * 1024;
    
    /// Number of retries for file operations
    pub const FILE_OPERATION_RETRIES: u32 = 3;
    
    /// Timeout for git operations in seconds
    pub const GIT_OPERATION_TIMEOUT_SECS: u64 = 30;
}
