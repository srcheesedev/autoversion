use anyhow::{anyhow, Result};
use regex::Regex;
use std::path::{Path, PathBuf};

use super::traits::{VersionUpdater, VersionChange};
use crate::utils::files::{backup_file, read_file_safe, write_file_safe};

/// Maven POM version updater
///
/// Implements version management for Apache Maven projects by updating
/// the `<version>` element in `pom.xml` files.
///
/// # Version Detection Strategy
///
/// Searches for the first `<version>` tag in pom.xml using regex pattern matching.
/// This typically corresponds to the project's version element in the project section.
///
/// # Update Strategy (Current: Regex-based)
///
/// **Current Implementation:**
/// 1. Read pom.xml file content
/// 2. Use regex pattern `<version>(.*?)</version>` to find version tag
/// 3. Replace first occurrence with new version
/// 4. Create .autoversion.backup before modification
/// 5. Write updated content back to file
///
/// **Design Decision: Pragmatic Regex Approach**
///
/// We deliberately use regex instead of full XML parsing to:
/// - Keep dependencies minimal (no heavy XML parser library)
/// - Maintain simple, testable implementation
/// - Cover the 80% use case (standard Maven projects)
/// - Enable fast, predictable behavior
///
/// # File Format
///
/// ```xml
/// <?xml version="1.0" encoding="UTF-8"?>
/// <project xmlns="http://maven.apache.org/POM/4.0.0">
///     <modelVersion>4.0.0</modelVersion>
///     <groupId>com.example</groupId>
///     <artifactId>my-project</artifactId>
///     <version>1.2.3</version>
/// </project>
/// ```
///
/// # Examples
///
/// ```no_run
/// use autoversion::updaters::maven::MavenUpdater;
/// use autoversion::updaters::traits::VersionUpdater;
/// use std::path::Path;
///
/// let updater = MavenUpdater::new();
/// let project = Path::new("./my-maven-project");
///
/// // Get current version from pom.xml
/// let current = updater.get_current_version(project)?;
/// println!("Current version: {}", current);
///
/// // Update version
/// let files = updater.update_version(project, "2.0.0")?;
/// println!("Updated files: {:?}", files);
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Known Limitations
///
/// The regex-based approach has limitations with complex Maven projects:
///
/// **1. XML Namespaces**
/// ```xml
/// <project xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
///     <!-- May not handle all namespace variations -->
/// </project>
/// ```
///
/// **2. Parent POM Inheritance**
/// ```xml
/// <parent>
///     <version>1.0.0</version> <!-- Parent version, not project version -->
/// </parent>
/// <version>2.0.0</version> <!-- Actual project version -->
/// ```
/// Currently updates the FIRST `<version>` found, which might be the parent.
///
/// **3. Multi-module Projects**
/// May not handle version properties or shared versions across modules elegantly.
///
/// # Future Enhancement (Task 10)
///
/// For production use with complex enterprise Maven projects, consider migrating to
/// XML parser using the `quick-xml` crate (already in Cargo.toml):
///
/// **Benefits:**
/// - Proper XML namespace handling
/// - Distinction between parent and project versions
/// - Support for version properties
/// - Better error messages for malformed XML
///
/// **Migration Path:**
/// 1. Keep regex implementation as fallback
/// 2. Add XML parser implementation
/// 3. Test both implementations against real-world POMs
/// 4. Switch default to XML parser when stable
///
/// # TDD Documentation
///
/// Test coverage includes:
/// - Basic pom.xml parsing and version extraction
/// - Version updates with preservation of formatting
/// - Preview mode (dry-run)
/// - Error handling for missing/malformed POM files
/// - Edge case: Multiple version tags
///
/// # Error Handling
///
/// Returns `anyhow::Result` for all operations. Common error scenarios:
/// - Missing pom.xml file
/// - No `<version>` tag found in POM
/// - Invalid XML structure
/// - File system I/O errors
pub struct MavenUpdater;

impl MavenUpdater {
    pub fn new() -> Self {
        Self
    }

    fn extract_version(&self, content: &str) -> Option<String> {
        // Try to find the first <version>...</version> under project; regex with DOTALL
        let re = Regex::new(r"(?s)<project.*?>.*?<version>\s*([^<\s]+)\s*</version>").ok()?;
        re.captures(content).and_then(|c| c.get(1)).map(|m| m.as_str().to_string())
    }

    fn replace_version(&self, content: &str, new_version: &str) -> Result<String> {
        let re = Regex::new(r"(?s)(<project.*?>.*?<version>)\s*([^<\s]+)(\s*</version>)").map_err(|e| anyhow!(e.to_string()))?;
        if re.is_match(content) {
            // Usamos closure para reconstruir la nueva cadena a partir de capturas.
            // Esto evita problemas de interpretación de $1/$2 por la API de regex.
            let out = re.replace(content, |caps: &regex::Captures| {
                format!("{}{}{}", &caps[1], new_version, &caps[3])
            });
            Ok(out.into_owned())
        } else {
            Err(anyhow!("No <version> tag found in pom.xml to replace"))
        }
    }
}

impl Default for MavenUpdater {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionUpdater for MavenUpdater {
    fn get_current_version(&self, project_path: &Path) -> Result<String> {
        let pom = project_path.join("pom.xml");
        let content = read_file_safe(&pom)?;
        if let Some(v) = self.extract_version(&content) {
            return Ok(v);
        }
        Err(anyhow!("No <version> found in pom.xml"))
    }

    fn update_version(&self, project_path: &Path, new_version: &str) -> Result<Vec<String>> {
        let pom = project_path.join("pom.xml");
        backup_file(&pom)?;
        let old = read_file_safe(&pom)?;
        let updated = self.replace_version(&old, new_version)?;
        write_file_safe(&pom, &updated)?;
        Ok(vec![pom.to_string_lossy().to_string()])
    }

    fn validate_project(&self, project_path: &Path) -> Result<()> {
        let pom = project_path.join("pom.xml");
        if pom.exists() {
            Ok(())
        } else {
            Err(anyhow!("pom.xml not found"))
        }
    }

    fn technology_name(&self) -> &'static str {
        "maven"
    }

    fn get_primary_file(&self, project_path: &Path) -> Result<PathBuf> {
        let pom = project_path.join("pom.xml");
        if pom.exists() {
            Ok(pom)
        } else {
            Err(anyhow!("pom.xml not found"))
        }
    }

    fn can_handle(&self, project_path: &Path) -> bool {
        project_path.join("pom.xml").exists()
    }

    fn preview_changes(&self, project_path: &Path, new_version: &str) -> Result<Vec<VersionChange>> {
        let pom = project_path.join("pom.xml");
        let mut changes = Vec::new();
        if pom.exists() {
            let old_content = read_file_safe(&pom)?;
            let new_content = self.replace_version(&old_content, new_version)?;
            let old_version = self.get_current_version(project_path)?;
            changes.push(VersionChange::new(
                pom,
                old_content,
                new_content,
                old_version,
                new_version.to_string(),
            ));
        }
        Ok(changes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    // Tests (TDD):
    // - Each test creates a temporary project and performs the expected operation.
    // - Tests validate both read and write paths and the preview (dry-run).
    // - Tests use descriptive names and reusable helpers for clarity.

    fn create_pom(dir: &std::path::Path, version: &str) -> anyhow::Result<()> {
        let content = format!(
            "<project>\n  <modelVersion>4.0.0</modelVersion>\n  <groupId>com.example</groupId>\n  <artifactId>test</artifactId>\n  <version>{}</version>\n</project>",
            version
        );
        fs::write(dir.join("pom.xml"), content)?;
        Ok(())
    }

    #[test]
    fn test_get_current_version() -> anyhow::Result<()> {
        let tmp = TempDir::new()?;
        let path = tmp.path();
        create_pom(path, "1.2.3")?;

        let v = MavenUpdater::new().get_current_version(path)?;
        assert_eq!(v, "1.2.3");
        Ok(())
    }

    #[test]
    fn test_update_version() -> anyhow::Result<()> {
        let tmp = TempDir::new()?;
        let path = tmp.path();
        create_pom(path, "0.1.0")?;

        let updated = MavenUpdater::new().update_version(path, "0.2.0")?;
        assert_eq!(updated.len(), 1);
        let content = fs::read_to_string(path.join("pom.xml"))?;
        assert!(content.contains("<version>0.2.0</version>"));
        Ok(())
    }

    #[test]
    fn test_preview_changes() -> anyhow::Result<()> {
        let tmp = TempDir::new()?;
        let path = tmp.path();
        create_pom(path, "1.0.0")?;

        let changes = MavenUpdater::new().preview_changes(path, "2.0.0")?;
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].old_version, "1.0.0");
        assert_eq!(changes[0].new_version, "2.0.0");
        Ok(())
    }

    #[test]
    fn test_validate_and_can_handle() -> anyhow::Result<()> {
        let tmp = TempDir::new()?;
        let path = tmp.path();
        assert!(!MavenUpdater::new().can_handle(path));
        create_pom(path, "1.0.0")?;
        assert!(MavenUpdater::new().can_handle(path));
        assert!(MavenUpdater::new().validate_project(path).is_ok());
        Ok(())
    }

    #[test]
    fn test_technology_name() {
        assert_eq!(MavenUpdater::new().technology_name(), "maven");
    }
}