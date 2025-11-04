# Technology Expansion Roadmap

This document analyzes potential technologies to add to Autoversion, prioritized by usage, complexity, and community demand.

## Current Coverage

✅ **Implemented (5 technologies):**
- NPM/Node.js (`package.json`)
- Cargo/Rust (`Cargo.toml`)
- Maven/Java (`pom.xml`)
- Python (`pyproject.toml`, `setup.py`)
- Generic (plain version files)

---

## 🔥 High Priority - High Demand

### 1. **Go Modules**
**Priority:** ⭐⭐⭐⭐⭐

**Manifest Files:**
- `go.mod` - Module definition with version

**Version Format:**
```go
module github.com/user/project

go 1.21

require (
    github.com/user/dep v1.2.3
)
```

**Version Location:** Not stored in go.mod! Go uses git tags only.

**Strategy:**
- Version is determined by git tags (v1.2.3)
- No file updates needed, only git tagging
- Can support optional VERSION file for display purposes
- Validate semantic import versioning (v2+)

**Complexity:** Low (git tags only)

**Market:** Huge - Go is extremely popular for CLI/backend tools

---

### 2. **Gradle (Kotlin/Groovy)**
**Priority:** ⭐⭐⭐⭐⭐

**Manifest Files:**
- `build.gradle` (Groovy)
- `build.gradle.kts` (Kotlin DSL)
- `gradle.properties`

**Version Formats:**

**build.gradle (Groovy):**
```groovy
version = '1.2.3'
// or
version '1.2.3'
```

**build.gradle.kts (Kotlin):**
```kotlin
version = "1.2.3"
```

**gradle.properties:**
```properties
version=1.2.3
```

**Strategy:**
- Regex-based for simple cases (similar to Maven approach)
- Properties file is easiest to parse
- Support both Groovy and Kotlin DSL syntax

**Complexity:** Medium (multiple formats)

**Market:** Very large - Android, Spring Boot, Kotlin projects

---

### 3. **Ruby Gems**
**Priority:** ⭐⭐⭐⭐

**Manifest Files:**
- `*.gemspec` (e.g., `myproject.gemspec`)
- `lib/*/version.rb` (common pattern)

**Version Formats:**

**gemspec:**
```ruby
Gem::Specification.new do |spec|
  spec.name        = "my_gem"
  spec.version     = "1.2.3"
  # or
  spec.version     = MyGem::VERSION
end
```

**version.rb:**
```ruby
module MyGem
  VERSION = "1.2.3"
end
```

**Strategy:**
- Regex-based parsing for both formats
- Prioritize version.rb if it exists (canonical source)
- Update gemspec if version is hardcoded there

**Complexity:** Medium (Ruby parsing, module structure)

**Market:** Large - Rails, DevOps tools, gems ecosystem

---

### 4. **PHP Composer**
**Priority:** ⭐⭐⭐⭐

**Manifest Files:**
- `composer.json`
- `composer.lock`

**Version Format:**
```json
{
  "name": "vendor/package",
  "version": "1.2.3",
  "require": {
    "php": "^8.0"
  }
}
```

**Strategy:**
- JSON parsing (similar to NPM approach)
- Update version field in composer.json
- Update composer.lock if present

**Complexity:** Low (JSON parsing, already have pattern)

**Market:** Large - PHP web development, Laravel, WordPress plugins

---

### 5. **.NET (C#/F#)**
**Priority:** ⭐⭐⭐⭐

**Manifest Files:**
- `*.csproj` (C# project)
- `*.fsproj` (F# project)
- `Directory.Build.props` (shared properties)

**Version Format:**
```xml
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
    <Version>1.2.3</Version>
    <AssemblyVersion>1.2.3.0</AssemblyVersion>
    <FileVersion>1.2.3.0</FileVersion>
  </PropertyGroup>
</Project>
```

**Strategy:**
- XML parsing (can use quick-xml crate)
- Update Version, AssemblyVersion, FileVersion
- Support Directory.Build.props for monorepos

**Complexity:** Medium (XML parsing, multiple version fields)

**Market:** Large - Enterprise .NET, ASP.NET, game development

---

### 6. **Elixir/Erlang**
**Priority:** ⭐⭐⭐

**Manifest Files:**
- `mix.exs` (Elixir)
- `rebar.config` (Erlang)
- `src/*.app.src` (Erlang OTP)

**Version Formats:**

**mix.exs:**
```elixir
defmodule MyApp.MixProject do
  use Mix.Project

  def project do
    [
      app: :my_app,
      version: "1.2.3",
      elixir: "~> 1.14"
    ]
  end
end
```

**Strategy:**
- Regex-based for mix.exs (find version: "X.Y.Z")
- Erlang term parsing for rebar.config

**Complexity:** Medium (Elixir/Erlang syntax)

**Market:** Medium - Growing Phoenix/Elixir community

---

## 📦 Medium Priority - Ecosystem Specific

### 7. **Swift Package Manager**
**Priority:** ⭐⭐⭐

**Manifest Files:**
- `Package.swift`

**Version Format:**
```swift
// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "MyPackage",
    products: [
        .library(name: "MyPackage", targets: ["MyPackage"])
    ]
)
```

**Note:** SPM uses git tags only (like Go), no version in Package.swift

**Strategy:**
- Git tags only (v1.2.3)
- Validate tag format
- Optional VERSION file support

**Complexity:** Low (git tags)

**Market:** Medium - iOS/macOS development

---

### 8. **Helm Charts (Kubernetes)**
**Priority:** ⭐⭐⭐

**Manifest Files:**
- `Chart.yaml`

**Version Format:**
```yaml
apiVersion: v2
name: my-chart
version: 1.2.3
appVersion: "1.2.3"
```

**Strategy:**
- YAML parsing
- Update both version and appVersion
- Validate against Helm schema

**Complexity:** Low (YAML parsing)

**Market:** Medium - Kubernetes/DevOps teams

---

### 9. **Ansible Collections**
**Priority:** ⭐⭐

**Manifest Files:**
- `galaxy.yml`

**Version Format:**
```yaml
namespace: mycompany
name: mycollection
version: 1.2.3
```

**Strategy:**
- YAML parsing
- Update version field

**Complexity:** Low (YAML parsing)

**Market:** Small-Medium - DevOps/Automation teams

---

### 10. **Terraform Modules**
**Priority:** ⭐⭐

**Manifest Files:**
- `version.tf` or `versions.tf`
- No standard version file! Uses git tags

**Strategy:**
- Git tags for module versioning
- Optional VERSION file
- Can parse version from variable defaults

**Complexity:** Low (mostly git tags)

**Market:** Medium - Infrastructure as Code

---

### 11. **Docker/Container Images**
**Priority:** ⭐⭐⭐

**Manifest Files:**
- `Dockerfile` (LABEL)
- `.dockerversion` or custom file

**Version Format:**
```dockerfile
FROM ubuntu:22.04
LABEL version="1.2.3"
LABEL release-date="2024-01-01"
```

**Strategy:**
- Update LABEL version in Dockerfile
- Fallback to VERSION file
- Integration with container registries (tag generation)

**Complexity:** Low-Medium

**Market:** Large - Containerized applications

---

## 🎯 Lower Priority - Niche Use Cases

### 12. **Lua/LuaRocks**
**Priority:** ⭐

**Manifest:** `*.rockspec`

**Market:** Small - Game development (Love2D), Neovim plugins

---

### 13. **Dart/Flutter**
**Priority:** ⭐⭐

**Manifest:** `pubspec.yaml`

**Market:** Medium - Flutter mobile apps

---

### 14. **Zig**
**Priority:** ⭐

**Manifest:** `build.zig.zon`

**Market:** Small but growing - Systems programming

---

### 15. **CMake (C/C++)**
**Priority:** ⭐⭐

**Manifest:** `CMakeLists.txt`

**Version Format:**
```cmake
project(MyProject VERSION 1.2.3)
```

**Market:** Large C++ codebase, but version management varies

---

## 📊 Implementation Priority Matrix

| Technology | Demand | Complexity | Effort (days) | Priority Score |
|------------|--------|------------|---------------|----------------|
| Go Modules | ⭐⭐⭐⭐⭐ | Low | 1-2 | **25** |
| Gradle | ⭐⭐⭐⭐⭐ | Medium | 3-4 | **21** |
| PHP Composer | ⭐⭐⭐⭐ | Low | 1-2 | **20** |
| Ruby Gems | ⭐⭐⭐⭐ | Medium | 2-3 | **18** |
| .NET | ⭐⭐⭐⭐ | Medium | 2-3 | **18** |
| Helm Charts | ⭐⭐⭐ | Low | 1 | **15** |
| Docker | ⭐⭐⭐ | Low-Med | 2 | **13** |
| Elixir | ⭐⭐⭐ | Medium | 2-3 | **12** |
| Swift | ⭐⭐⭐ | Low | 1 | **12** |
| Dart/Flutter | ⭐⭐ | Low | 1-2 | **8** |
| Terraform | ⭐⭐ | Low | 1 | **8** |
| Ansible | ⭐⭐ | Low | 1 | **6** |

**Priority Formula:** (Demand × 5) + (6 - Complexity) - Effort

---

## 🚀 Recommended Implementation Order

### Phase 3 (Next Sprint)
1. **Go Modules** - Massive Go ecosystem, simple implementation (git tags)
2. **PHP Composer** - Large PHP community, easy JSON parsing

### Phase 4
3. **Gradle** - Critical for Android/Kotlin, supports multiple formats
4. **Ruby Gems** - Strong Rails/DevOps community
5. **.NET** - Enterprise market, requires XML handling

### Phase 5
6. **Helm Charts** - Growing Kubernetes adoption
7. **Docker/Container** - Universal containerization
8. **Elixir** - Growing community, interesting tech

### Future Considerations
9. Swift Package Manager
10. Dart/Flutter
11. Terraform Modules
12. Ansible Collections

---

## 🔧 Technical Considerations

### Shared Infrastructure Needs

**YAML Parsing:**
- Add `serde_yaml` dependency
- Needed for: Helm, Ansible, Dart, Terraform variables

**XML Parsing (already declared):**
- Activate `quick-xml` crate
- Needed for: .NET, Maven improvements

**TOML Parsing (already available):**
- Currently used for: Cargo, Python pyproject.toml
- Works for any TOML-based configs

**Properties File Parsing:**
- Simple key=value format
- Needed for: Gradle gradle.properties, Java properties
- Can implement with regex or existing crates

### Testing Strategy

For each new technology:
1. ✅ Unit tests for version detection
2. ✅ Unit tests for version updates
3. ✅ Contract test for I/O operations
4. ✅ Integration test with real project structure
5. ✅ Doc tests in documentation
6. ✅ Edge cases (missing files, malformed content)

---

## 💡 Community Input

Consider adding:
- **User survey** to prioritize based on actual demand
- **GitHub discussions** for feature requests
- **Telemetry** (opt-in) to see which technologies are most used
- **Plugin system** for community-contributed updaters

---

## 🎯 Quick Wins

These technologies can be added in **<1 day each**:

1. **Go Modules** - Just git tags, no file parsing
2. **PHP Composer** - JSON parsing (already have pattern)
3. **Helm Charts** - Simple YAML parsing
4. **Swift Package Manager** - Git tags only

These could form a **"Quick Expansion Sprint"** to rapidly increase supported technologies from 5 to 9.

---

## 📝 Implementation Checklist Template

For each new technology:

- [ ] Create `src/updaters/[tech].rs` file
- [ ] Implement `VersionUpdater` trait
- [ ] Add comprehensive documentation with examples
- [ ] Add unit tests (5+ tests minimum)
- [ ] Add I/O contract test
- [ ] Update `TechnologyDetector` in `src/core/detector.rs`
- [ ] Update `UpdaterFactory` in `src/updaters/factory.rs`
- [ ] Add to README.md supported technologies table
- [ ] Update CLI `--technology` help text
- [ ] Add integration test
- [ ] Test with real-world project
- [ ] Document limitations and edge cases

---

**Last Updated:** November 4, 2025
**Next Review:** After Phase 3 completion
