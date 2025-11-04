# Contributing to Autoversion

Thank you for your interest in contributing to Autoversion! This guide will help you get started.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Coding Standards](#coding-standards)
- [Testing Strategy](#testing-strategy)
- [Submitting Changes](#submitting-changes)
- [Commit Conventions](#commit-conventions)
- [Architecture Overview](#architecture-overview)

## Code of Conduct

Be respectful, inclusive, and constructive. We're all here to build great software together.

## Getting Started

### Prerequisites

- **Rust**: Version 1.70 or later
- **Git**: For version control
- **Cargo**: Comes with Rust

### Fork and Clone

```bash
# Fork the repository on GitHub, then clone your fork
git clone https://github.com/YOUR_USERNAME/autoversion.git
cd autoversion

# Add upstream remote
git remote add upstream https://github.com/srcheesedev/autoversion.git
```

## Development Setup

### 1. Install Rust

If you haven't already:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Build the Project

```bash
# Build in debug mode
cargo build

# Build in release mode (optimized)
cargo build --release
```

### 3. Run Tests

```bash
# Run all tests
cargo test

# Run tests with single thread (for stability)
cargo test -- --test-threads=1

# Run specific test
cargo test test_name

# Run with verbose output
cargo test -- --nocapture
```

### 4. Run the CLI

```bash
# Debug build
cargo run -- --help

# Release build
./target/release/autoversion --help

# Test on a project
cargo run -- -b patch -d -v
```

### 5. Check Code Quality

```bash
# Format code
cargo fmt

# Run clippy (linter)
cargo clippy -- -D warnings

# Check for compilation errors without building
cargo check
```

## Coding Standards

### Rust Style Guide

Follow the official [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/README.html):

- **Use `cargo fmt`** before committing
- **Fix all clippy warnings**: `cargo clippy -- -D warnings`
- **Use meaningful names**: Prefer clarity over brevity
- **Keep functions small**: < 20 lines when possible
- **Single Responsibility**: Each function does one thing well

### Documentation

- **Public items must have doc comments**: Use `///` for public APIs
- **Include examples**: Demonstrate usage in doc comments
- **Explain why, not just what**: Document design decisions
- **Keep docs up to date**: Update docs when changing code

Example:

```rust
/// Extracts the version from a package.json file.
///
/// This function parses the JSON content and returns the value of the
/// "version" field. It handles both standard and non-standard formatting.
///
/// # Arguments
///
/// * `content` - The raw JSON content as a string
///
/// # Returns
///
/// Returns the version string if found, or an error if parsing fails.
///
/// # Examples
///
/// ```
/// let content = r#"{"name": "app", "version": "1.2.3"}"#;
/// let version = extract_version_from_json(content)?;
/// assert_eq!(version, "1.2.3");
/// ```
pub fn extract_version_from_json(content: &str) -> Result<String> {
    // Implementation
}
```

### Error Handling

- **Use `Result<T, E>`** for operations that can fail
- **Provide context**: Use `.context()` or `.with_context()` from `anyhow`
- **Don't panic**: Use `Result` instead of `panic!`, `unwrap()`, or `expect()`
- **Sanitize errors**: Remove sensitive information before displaying

Example:

```rust
use anyhow::{Context, Result};

fn read_config(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read config from {}", path.display()))?;
    
    serde_json::from_str(&content)
        .context("Failed to parse config JSON")
}
```

### Constants

- **Use the constants module**: No magic strings or numbers
- **Document constants**: Explain their purpose
- **Group related constants**: Use logical organization

### Security

- **Validate all paths**: Use `security::validate_project_path()` and `security::validate_file_path()`
- **Sanitize filenames**: Use `security::sanitize_filename()`
- **Lock files during writes**: Use `security::lock_file_for_write()`
- **No shell commands**: Use pure Rust or safe libraries like `git2`

## Testing Strategy

We follow **Test-Driven Development (TDD)** for all new features.

### TDD Workflow

1. **RED**: Write a failing test
2. **GREEN**: Write minimal code to make it pass
3. **REFACTOR**: Improve the code quality
4. **REPEAT**: Add more tests for edge cases

### Test Categories

#### 1. Unit Tests

Test individual functions in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let version = "1.2.3";
        let (major, minor, patch) = parse_version(version).unwrap();
        assert_eq!(major, 1);
        assert_eq!(minor, 2);
        assert_eq!(patch, 3);
    }
}
```

**Guidelines**:
- Test happy paths and edge cases
- Use descriptive test names: `test_<function>_<scenario>_<expected>`
- Keep tests focused and independent
- Use `assert_eq!`, `assert!`, `assert!(matches!(...))` appropriately

#### 2. Contract Tests

Test I/O boundaries and file operations:

```rust
// In tests/my_contract_test.rs
use tempfile::TempDir;

#[test]
fn updater_creates_backup_before_update() {
    let temp = TempDir::new().unwrap();
    // Test file operations with real filesystem
}
```

**Guidelines**:
- Use `tempfile::TempDir` for isolated environments
- Place in `/tests` directory
- Test actual file creation, reading, writing
- Verify backup/restore functionality

#### 3. Edge Case Tests

Test unusual or extreme conditions:

```rust
#[test]
fn test_large_file_handling() {
    // Test with 5MB file
}

#[test]
fn test_invalid_utf8_content() {
    // Test with binary data
}
```

**Guidelines**:
- Cover boundary conditions
- Test error scenarios
- Include concurrent access tests
- Test malformed input data

#### 4. Doc Tests

Embed tests in documentation:

```rust
/// Returns the square of a number.
///
/// # Examples
///
/// ```
/// let result = square(4);
/// assert_eq!(result, 16);
/// ```
pub fn square(x: i32) -> i32 {
    x * x
}
```

**Guidelines**:
- Include in all public API docs
- Show realistic usage examples
- Keep examples simple and focused

### Test Organization

```
autoversion/
├── src/
│   ├── lib.rs              # Unit tests here with #[cfg(test)]
│   ├── main.rs             # Unit tests here with #[cfg(test)]
│   └── updaters/
│       └── npm.rs          # Unit tests in #[cfg(test)] mod tests
│
└── tests/
    ├── updater_io_npm_contract.rs       # Contract tests
    ├── edge_cases_file_operations.rs    # Edge case tests
    └── edge_cases_version_parsing.rs    # Edge case tests
```

### Running Specific Test Types

```bash
# Unit tests only (in src/)
cargo test --lib

# All tests in tests/ directory
cargo test --test '*'

# Specific test file
cargo test --test edge_cases_file_operations

# Doc tests only
cargo test --doc
```

## Submitting Changes

### Before Submitting

1. **Write tests**: Follow TDD methodology
2. **Run all tests**: `cargo test -- --test-threads=1`
3. **Format code**: `cargo fmt`
4. **Check lints**: `cargo clippy -- -D warnings`
5. **Update documentation**: Add/update doc comments and README if needed
6. **Test locally**: Run the CLI with your changes

### Pull Request Process

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/my-new-feature
   ```

2. **Make your changes** following our coding standards

3. **Commit with conventional commits** (see below)

4. **Push to your fork**:
   ```bash
   git push origin feature/my-new-feature
   ```

5. **Open a Pull Request**:
   - Use a clear, descriptive title
   - Reference related issues: "Fixes #123"
   - Describe what changed and why
   - Include test results
   - Add screenshots/examples if relevant

6. **Respond to feedback**:
   - Be open to suggestions
   - Make requested changes
   - Update your PR branch if needed

### PR Template

```markdown
## Description
Brief description of changes

## Motivation
Why is this change needed?

## Changes Made
- Added X feature
- Fixed Y bug
- Refactored Z module

## Testing
- [ ] All tests pass
- [ ] Added new tests
- [ ] Tested manually with: `cargo run -- <args>`

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No clippy warnings
- [ ] Formatted with `cargo fmt`
```

## Commit Conventions

We follow [Conventional Commits](https://www.conventionalcommits.org/) specification.

### Commit Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation changes
- **style**: Code style (formatting, whitespace)
- **refactor**: Code refactoring
- **perf**: Performance improvements
- **test**: Adding or updating tests
- **chore**: Build process, tooling, dependencies

### Examples

```bash
# Simple feature
feat: add gradle support for kotlin dsl

# Bug fix with scope
fix(maven): handle xml namespaces correctly

# Breaking change
feat!: redesign updater trait interface

BREAKING CHANGE: VersionUpdater::update now returns Vec<VersionChange>
instead of Result<()>

# Documentation
docs: add architecture diagrams to ARCHITECTURE.md

# Test addition
test: add edge cases for large file handling

# Refactoring
refactor: extract main.rs into testable functions
```

### Commit Message Guidelines

- **Use imperative mood**: "add" not "added" or "adds"
- **Keep subject line short**: < 72 characters
- **Capitalize subject line**: Start with uppercase letter
- **No period at end**: Of subject line
- **Include body for complex changes**: Explain what and why
- **Reference issues**: Use "Fixes #123", "Closes #456"

## Architecture Overview

Before contributing, familiarize yourself with our architecture:

### Key Concepts

1. **Strategy Pattern**: Each technology has its own `VersionUpdater` implementation
2. **Factory Pattern**: `UpdaterFactory` creates appropriate updaters
3. **TDD Approach**: Write tests before implementation
4. **Security First**: Validate all paths and inputs

### Module Structure

```
src/
├── main.rs              # Entry point with orchestration
├── lib.rs               # Library exports
├── cli/                 # Command-line interface
│   ├── args.rs          # Argument parsing
│   ├── output.rs        # Result formatting
│   └── analyze.rs       # Commit analysis
├── core/                # Core business logic
│   ├── detector.rs      # Technology detection
│   ├── semver.rs        # Version calculations
│   └── config.rs        # Configuration
├── updaters/            # Version updaters (Strategy Pattern)
│   ├── traits.rs        # VersionUpdater trait
│   ├── factory.rs       # Factory for creating updaters
│   ├── npm.rs           # NPM implementation
│   ├── cargo.rs         # Cargo implementation
│   └── ...              # Other implementations
├── git/                 # Git operations
│   ├── operations.rs    # Tag/commit creation
│   └── history.rs       # Commit analysis
└── utils/               # Utilities
    ├── files.rs         # File operations
    ├── errors.rs        # Error types
    └── security.rs      # Security utilities
```

### Adding a New Technology

To add support for a new technology:

1. **Create the updater** in `src/updaters/your_tech.rs`:
   ```rust
   pub struct YourTechUpdater;
   
   impl VersionUpdater for YourTechUpdater {
       // Implement trait methods
   }
   ```

2. **Add to Technology enum** in `src/core/detector.rs`:
   ```rust
   pub enum Technology {
       YourTech,
       // ...
   }
   ```

3. **Add detection logic** in `src/core/detector.rs`:
   ```rust
   pub fn detect_technology(path: &Path) -> Result<Technology> {
       // Check for your_tech manifest files
   }
   ```

4. **Register in factory** in `src/updaters/factory.rs`:
   ```rust
   Technology::YourTech => Ok(Box::new(YourTechUpdater)),
   ```

5. **Add constants** in `src/constants.rs`:
   ```rust
   pub const YOUR_TECH_MANIFEST: &str = "your_tech.conf";
   ```

6. **Write comprehensive tests**:
   - Unit tests in the updater module
   - Contract tests in `tests/updater_io_your_tech_contract.rs`
   - Doc tests in documentation

7. **Update documentation**:
   - Add to README.md supported technologies table
   - Update ARCHITECTURE.md if needed

See `src/updaters/gradle.rs` for a complete reference implementation.

## Need Help?

- **Questions?** Open a [GitHub Discussion](https://github.com/srcheesedev/autoversion/discussions)
- **Bug?** Open an [Issue](https://github.com/srcheesedev/autoversion/issues)
- **Stuck?** Check [ARCHITECTURE.md](ARCHITECTURE.md) for design details
- **Want to chat?** Find us in the community channels

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Semantic Versioning](https://semver.org/)
- [Project Architecture](ARCHITECTURE.md)

---

**Thank you for contributing to Autoversion!** 🎉

Your contributions make this project better for everyone.
