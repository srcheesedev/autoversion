# 🚀 Autoversion (Rust CLI)

Universal semantic versioning automation for any technology stack. Built in Rust for maximum performance and reliability.

## ✨ Features

- 🌍 **Universal**: Supports npm, Cargo, Maven, Python, and generic projects
- ⚡ **Fast**: Rust binary with <100ms startup time
- 🔒 **Reliable**: Comprehensive testing and type safety
- 🤖 **Automatic**: Smart commit analysis for version bumping
- 🏷️ **Git Integration**: Automatic tagging and commit creation
- 📊 **Rich Output**: Detailed CLI output

## 🎯 Supported Technologies

| Technology | Files | Status |
|------------|-------|--------|
| **NPM** | `package.json`, `package-lock.json` | ✅ Ready |
| **Generic** | `VERSION`, `version.txt`, `.version` | ✅ Ready |
| **Cargo** | `Cargo.toml`, `Cargo.lock` | 🔄 Phase 2 |
| **Maven** | `pom.xml` | 🔄 Phase 2 |
| **Python** | `pyproject.toml`, `setup.py` | 🔄 Phase 2 |

## 🚀 Quick Start

```bash
# Build the binary (if not built)
cargo build --release

# Show help
./target/release/autoversion --help

# Bump version automatically (auto-detect technology)
./target/release/autoversion -b auto

# Bump patch version for NPM project, create tag, and commit
./target/release/autoversion -t npm -b patch -c -C

# Preview changes without modifying files
./target/release/autoversion -d -v
```

## 📖 Usage

Run `autoversion` in your project directory. Example:

```bash
./autoversion -b auto
```

### Common Options

- `-b`, `--bump-type`: Version bump type (`auto`, `major`, `minor`, `patch`)
- `-t`, `--technology`: Technology (`auto`, `npm`, `cargo`, `maven`, `python`, `generic`)
- `-c`, `--create-tag`: Create git tag for new version
- `-C`, `--commit`: Commit version changes to git
- `-d`, `--dry-run`: Preview changes without modifying files
- `-f`, `--force`: Force bump even with uncommitted changes
- `-v`, `--verbose`: Enable verbose output
- `-m`, `--commit-message`: Custom commit message
- `-p`, `--path`: Path to project directory (default: current)

See `./autoversion --help` for all options.

## 📤 Outputs

| Output | Description |
|--------|-------------|
| `version` | The new version that was set |
| `previous-version` | The previous version before the bump |
| `version-type` | The type of version bump applied |
| `technology` | The technology that was detected/used |
| `files-updated` | Comma-separated list of updated files |
| `tag-created` | Whether a git tag was created |
| `tag-name` | The name of the git tag created |
| `success` | Whether the operation completed successfully |

## 🤖 Automatic Version Bumping

When `bump-type` is set to `auto`, the action analyzes commit messages using [Conventional Commits](https://www.conventionalcommits.org/):

- **Major**: Breaking changes (`BREAKING CHANGE:` or `!` suffix)
- **Minor**: New features (`feat:` prefix)
- **Patch**: Bug fixes (`fix:` prefix) or other changes

### Examples

```bash
feat: add new user authentication        # → Minor bump
fix: resolve login validation bug        # → Patch bump
feat!: redesign API endpoints           # → Major bump
feat: add search BREAKING CHANGE: ...   # → Major bump
```

## 🏷️ Git Integration

### Automatic Tagging

When `create-tag: true` (default), creates annotated tags:

```bash
git tag -a v1.2.3 -m "Release version 1.2.3"
```

### Automatic Commits

When `commit: true`, commits version changes:

```bash
git add package.json package-lock.json
git commit -m "chore: bump version to 1.2.3"
```

Custom commit messages support `{version}` placeholder:

```yaml
commit-message: "🚀 Release version {version}"
# Results in: "🚀 Release version 1.2.3"
```

## 🔍 Examples by Technology (CLI)

### NPM Project

```bash
# Run in the project root
./autoversion -t npm -b auto --create-tag
```

Updates: `package.json`, `package-lock.json`

### Generic Project

```bash
# Run in the project root
./autoversion -t generic -b patch
```

Updates: `VERSION`, `version.txt`, or `.version`

### Multi-Technology Project

```bash
# Auto-detect technology and bump
./autoversion -t auto -b auto
```

## 🛡️ Security & Permissions

This CLI interacts with your local git repository. Locally, ensure you have a git user configured:

```bash
git config user.name "Your Name"
git config user.email "you@example.com"
```

If you integrate this tool into CI via a wrapper (GitHub Action, GitLab job, Jenkins task, etc.), the wrapper repository will handle setting the required permissions and tokens — see the corresponding wrapper docs when available.

## 🧪 Dry Run Mode

Preview changes without modifying files:

```bash
./autoversion -d -v
```

Output shows what would be changed, for example:

```
🔍 Dry run mode - no changes made
Technology: npm
Current version: 1.2.3
New version: 1.3.0 (minor)
Files to update:
  • package.json
  • package-lock.json
Tag to create: v1.3.0
```

## ⚠️ Troubleshooting

### Common Issues

**"No supported technology detected"**
- Ensure your project has recognized files (`package.json`, `VERSION`, etc.)
- Use `technology` input to specify explicitly
- Use `verbose: true` for detailed detection info

**"Failed to create tag"**
- Check repository permissions
- Ensure `contents: write` permission
- Verify tag doesn't already exist

**"Repository has uncommitted changes"**
- Commit or stash changes first
- Use `force: true` to override (not recommended)

### Debug Mode

Enable verbose output for troubleshooting (CLI):

```bash
./autoversion -v
```

## 🔄 Migration Guide

If you used an action or plugin previously, switch to the CLI by invoking the equivalent commands in your CI job or locally. Example translations:

- From an npm-oriented action that did `version: patch` → run:

```bash
./autoversion -t npm -b patch
```

- From a semantic-release flow that relied on conventional commits → run:

```bash
./autoversion -b auto --create-tag --commit
```

When using in CI, prefer a small wrapper repo per platform that downloads the compiled binary and handles credential setup for that CI provider.

## 🚧 Roadmap

### Phase 2 (Coming Soon)

- ✅ Full Cargo.toml support
- ✅ Maven pom.xml support  
- ✅ Python pyproject.toml support
- ✅ Monorepo support
- ✅ Changelog generation

### Phase 3 (Future)

- 🔄 Go modules support
- 🔄 Ruby gems support
- 🔄 Custom version patterns
- 🔄 Rollback functionality

## 🤝 Contributing

We welcome contributions! See our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone repository
git clone https://github.com/srcheesedev/autoversion-ghaction.git
cd autoversion-ghaction

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build project
cargo build

# Run tests
cargo test

# Test locally
cargo run -- --help
```

## 📄 License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- [Semantic Versioning](https://semver.org/) specification
- [Conventional Commits](https://www.conventionalcommits.org/) standard
-- The communities and tools that inspired this project
- [Rust Programming Language](https://www.rust-lang.org/)

---

**Made with ❤️ and ⚡ by [@srcheesedev](https://github.com/srcheesedev)**

For support, please [open an issue](https://github.com/srcheesedev/autoversion-ghaction/issues) or start a [discussion](https://github.com/srcheesedev/autoversion-ghaction/discussions).
