//! Git operations module
//!
//! This module provides safe git operations using the `git2` library
//! instead of shell commands to prevent command injection vulnerabilities.
//!
//! ## Security
//!
//! All git operations use the `git2` crate's safe Rust API:
//! - No shell command execution
//! - No string interpolation in commands
//! - No user input passed to shell
//! - Type-safe git operations
//!
//! This eliminates common security risks:
//! - Command injection attacks
//! - Shell metacharacter exploitation
//! - Unvalidated input execution
//!
//! ## Modules
//!
//! - `operations`: Tag creation, commits, and basic git operations
//! - `history`: Commit analysis for version bump determination

pub mod history;
pub mod operations;
