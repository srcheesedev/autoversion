# CI/CD Workflow Usage

## 🚀 Automatic Versioning & Releases

This project uses an automated CI/CD pipeline that handles testing, building, and releasing across all platforms.

## 📋 How It Works

### Regular Development
```bash
# Work on feature
git commit -m "feat: add new feature"
git push origin develop

# CI runs: Tests on all 4 platforms ✅
# No release created
```

### Creating a Release
```bash
# When ready to release, include [release] in commit message
git commit -m "feat: complete rollback CLI [release]"
git push origin develop

# CI automatically:
# 1. Tests on all platforms ✅
# 2. Builds binaries ✅
# 3. Runs autoversion (bumps patch version) ✅
# 4. Creates tag (e.g., v0.2.3) ✅
# 5. Creates GitHub Release with binaries ✅
```

### Specifying Version Bump Type

**Patch (default):**
```bash
git commit -m "fix: bug fix [release]"
# Results in: 0.2.2 → 0.2.3
```

**Minor:**
```bash
git commit -m "feat: new feature [release:minor]"
# Results in: 0.2.2 → 0.3.0
```

**Major:**
```bash
git commit -m "feat!: breaking change [release:major]"
# Results in: 0.2.2 → 1.0.0
```

## 🎯 Workflow Stages

### Stage 1: Test (Always runs)
- **Platforms:** Linux, macOS x64, macOS ARM, Windows
- **Duration:** ~4 minutes (parallel)
- **Actions:**
  - Clippy linting
  - All 419 tests
  - Doc tests

### Stage 2: Build (After tests pass)
- **Platforms:** Linux (musl), macOS x64, macOS ARM, Windows
- **Duration:** ~4 minutes (parallel)
- **Artifacts:**
  - `autoversion-linux-x86_64.tar.gz`
  - `autoversion-macos-x86_64.tar.gz`
  - `autoversion-macos-aarch64.tar.gz`
  - `autoversion-windows-x86_64.exe.zip`
  - `checksums.txt` (SHA256)

### Stage 3: Auto-Version (Only on `[release]`)
- **Condition:** Commit message contains `[release]`
- **Actions:**
  - Runs autoversion to bump version
  - Updates Cargo.toml
  - Creates version commit
  - Creates git tag
  - Pushes to develop

### Stage 4: Release (On tag push)
- **Trigger:** Tag push from auto-version or manual
- **Actions:**
  - Downloads all binaries
  - Generates release notes from commits
  - Creates GitHub Release
  - Uploads all binaries

## 📦 Downloading Releases

Releases are available at: https://github.com/srcheesedev/autoversion/releases

Each release includes:
- Pre-built binaries for all platforms
- SHA256 checksums
- Auto-generated changelog

### Verify Downloads
```bash
# Download your platform's binary and checksums.txt
sha256sum -c checksums.txt
```

## 🔧 Manual Release (Alternative)

If you prefer manual control:

```bash
# Build and version locally
cargo build --release
./target/release/autoversion -b patch --create-tag --commit

# Push tag to trigger release
git push origin develop
git push origin v0.2.3

# CI builds and releases automatically
```

## ⚙️ CI Configuration

### Branch Protection
- `develop`: All tests must pass before merge
- `main`: Protected, only accepts PRs from develop

### Triggers
- **Push to develop:** Test + Build
- **Push tag `v*`:** Test + Build + Release
- **Commit with `[release]`:** Auto-version + Release

## 🐛 Troubleshooting

### CI Failed on Test Stage
Check the logs for the specific platform that failed. Common issues:
- Clippy warnings (fix with `cargo clippy --fix`)
- Test failures (run `cargo test` locally)

### Release Not Created
Ensure:
- Commit message contains `[release]`
- All tests passed
- Pushed to `develop` branch

### Binary Not Working
- Verify SHA256 checksum
- Check you downloaded correct platform binary
- Linux: May need to `chmod +x autoversion`

## 📊 CI Minutes Usage

Estimated monthly usage (free tier: 2,000 minutes):
- Regular commits: ~400 minutes
- Releases: ~90 minutes
- **Total: ~490 minutes/month (25% of free tier)** ✅
