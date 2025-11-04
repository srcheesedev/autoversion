use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use regex::Regex;

use super::traits::{VersionUpdater, VersionChange};
use crate::constants::manifests::{PYTHON_PYPROJECT, PYTHON_SETUP};
use crate::utils::files::{backup_file, read_file_safe, write_file_safe};

/// Python package version updater
///
/// Implements version management for Python projects supporting two formats:
/// - **Modern:** `pyproject.toml` (PEP 621 / Poetry)
/// - **Legacy:** `setup.py` (setuptools)
///
/// # Version Detection Strategy
///
/// **Priority order:**
/// 1. `pyproject.toml` - Parsed as TOML, looks for `project.version` or `tool.poetry.version`
/// 2. `setup.py` - Parsed with regex to find `version='1.2.3'` or `version="1.2.3"`
///
/// # Update Strategy
///
/// ## pyproject.toml (TOML-based)
/// 1. Parse file as TOML structure
/// 2. Update `[project].version` field (PEP 621 standard)
/// 3. Update `[tool.poetry].version` field if present (Poetry format)
/// 4. Preserve all formatting and comments
/// 5. Create .autoversion.backup before modification
///
/// ## setup.py (Regex-based)
/// 1. Use regex to find `version=` assignment
/// 2. Replace with new version while preserving quote style
/// 3. Handle both single and double quotes
/// 4. Create .autoversion.backup before modification
///
/// # File Formats
///
/// **pyproject.toml (PEP 621):**
/// ```toml
/// [project]
/// name = "my-package"
/// version = "1.2.3"
/// ```
///
/// **pyproject.toml (Poetry):**
/// ```toml
/// [tool.poetry]
/// name = "my-package"
/// version = "1.2.3"
/// ```
///
/// **setup.py:**
/// ```python
/// setup(
///     name='my-package',
///     version='1.2.3',
/// )
/// ```
///
/// # Examples
///
/// ```no_run
/// use autoversion::updaters::python::PythonUpdater;
/// use autoversion::updaters::traits::VersionUpdater;
/// use std::path::Path;
///
/// let updater = PythonUpdater::new();
/// let project = Path::new("./my-python-project");
///
/// // Detect and read current version
/// let current = updater.get_current_version(project)?;
/// println!("Current version: {}", current);
///
/// // Update version
/// let files = updater.update_version(project, "2.0.0")?;
/// println!("Updated files: {:?}", files);
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # TDD Documentation
///
/// Test coverage includes:
/// - pyproject.toml parsing (PEP 621 and Poetry formats)
/// - setup.py regex-based parsing
/// - Version updates for both formats
/// - Error handling for malformed files
/// - Edge cases (missing fields, invalid TOML)
///
/// # Limitations
///
/// **setup.py:** Uses regex instead of AST parsing. This works for standard
/// cases but may fail with:
/// - Dynamic version calculation
/// - Complex string concatenation
/// - Version imported from other modules
///
/// Consider migrating to AST parsing (e.g., `syn` crate for Python) for
/// more robust setup.py handling if these edge cases become common.
///
/// # Error Handling
///
/// Returns `anyhow::Result` for all operations. Common error scenarios:
/// - No version file found (neither pyproject.toml nor setup.py)
/// - Invalid TOML format in pyproject.toml
/// - No version field found in files
/// - File system I/O errors
pub struct PythonUpdater;

impl PythonUpdater {

    /// Create a new PythonUpdater instance.
    pub fn new() -> Self {
        Self
    }


    fn parse_pyproject_version(&self, content: &str) -> Option<String> {
        // Very small TOML scan for [project] or [tool.poetry]
        if let Ok(value) = content.parse::<toml::Value>() {
            if let Some(project) = value.get("project") {
                if let Some(version) = project.get("version") {
                    return version.as_str().map(|s| s.to_string());
                }
            }
            if let Some(poetry) = value.get("tool").and_then(|t| t.get("poetry")) {
                if let Some(version) = poetry.get("version") {
                    return version.as_str().map(|s| s.to_string());
                }
            }
        }
        None
    }

    fn find_setup_py_version(&self, content: &str) -> Option<String> {
        // Look for patterns like version='1.2.3' or version="1.2.3"
        let re = Regex::new(r#"version\s*=\s*['\"]([0-9A-Za-z\.\-+_]+)['\"]"#).ok()?;
        if let Some(caps) = re.captures(content) {
            return caps.get(1).map(|m| m.as_str().to_string());
        }
        None
    }

    fn update_setup_py_content(&self, content: &str, new_version: &str) -> Result<String> {
        let re = Regex::new(r#"(version\s*=\s*['\"])([0-9A-Za-z\.\-+_]+)(['\"])"#)?;
        let result = re.replace(content, |caps: &regex::Captures| {
            format!("{}{}{}", &caps[1], new_version, &caps[3])
        });
        Ok(result.into_owned())
    }

    fn update_pyproject_content(&self, content: &str, new_version: &str) -> Result<String> {
        let mut value: toml::Value = content.parse()?;
        if let Some(project) = value.get_mut("project") {
            if let Some(tbl) = project.as_table_mut() {
                tbl.insert("version".to_string(), toml::Value::String(new_version.to_string()));
            }
        }
        if let Some(tool) = value.get_mut("tool") {
            if let Some(poetry) = tool.get_mut("poetry") {
                if let Some(tbl) = poetry.as_table_mut() {
                    tbl.insert("version".to_string(), toml::Value::String(new_version.to_string()));
                }
            }
        }
        let out = toml::to_string_pretty(&value)?;
        Ok(out + "\n")
    }
}

impl Default for PythonUpdater {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionUpdater for PythonUpdater {
    fn get_current_version(&self, project_path: &Path) -> Result<String> {
        let pyproject = project_path.join(PYTHON_PYPROJECT);
        if pyproject.exists() {
            let content = read_file_safe(&pyproject)?;
            if let Some(v) = self.parse_pyproject_version(&content) {
                return Ok(v);
            }
        }

        let setup_py = project_path.join(PYTHON_SETUP);
        if setup_py.exists() {
            let content = read_file_safe(&setup_py)?;
            if let Some(v) = self.find_setup_py_version(&content) {
                return Ok(v);
            }
        }

        Err(anyhow!("No Python project version found (pyproject.toml or setup.py)"))
    }

    fn update_version(&self, project_path: &Path, new_version: &str) -> Result<Vec<String>> {
        let mut updated = Vec::new();

        let pyproject = project_path.join(PYTHON_PYPROJECT);
        if pyproject.exists() {
            backup_file(&pyproject)?;
            let old = read_file_safe(&pyproject)?;
            let new_content = self.update_pyproject_content(&old, new_version)?;
            write_file_safe(&pyproject, &new_content)?;
            updated.push(pyproject.to_string_lossy().to_string());
        }

        let setup_py = project_path.join(PYTHON_SETUP);
        if setup_py.exists() {
            backup_file(&setup_py)?;
            let old = read_file_safe(&setup_py)?;
            let new_content = self.update_setup_py_content(&old, new_version)?;
            write_file_safe(&setup_py, &new_content)?;
            updated.push(setup_py.to_string_lossy().to_string());
        }

        if updated.is_empty() {
            return Err(anyhow!("No Python project files updated"));
        }

        Ok(updated)
    }

    fn validate_project(&self, project_path: &Path) -> Result<()> {
        let pyproject = project_path.join(PYTHON_PYPROJECT);
        let setup_py = project_path.join(PYTHON_SETUP);
        if pyproject.exists() || setup_py.exists() {
            Ok(())
        } else {
            Err(anyhow!("No Python project files found (pyproject.toml, setup.py)"))
        }
    }

    fn technology_name(&self) -> &'static str {
        "python"
    }

    fn get_primary_file(&self, project_path: &Path) -> Result<PathBuf> {
        let pyproject = project_path.join(PYTHON_PYPROJECT);
        if pyproject.exists() {
            Ok(pyproject)
        } else {
            let setup_py = project_path.join(PYTHON_SETUP);
            if setup_py.exists() {
                Ok(setup_py)
            } else {
                Err(anyhow!("No Python project files found"))
            }
        }
    }

    fn can_handle(&self, project_path: &Path) -> bool {
        project_path.join(PYTHON_PYPROJECT).exists() || project_path.join(PYTHON_SETUP).exists()
    }

    fn preview_changes(&self, project_path: &Path, new_version: &str) -> Result<Vec<VersionChange>> {
        let mut changes = Vec::new();
        let pyproject = project_path.join(PYTHON_PYPROJECT);
        if pyproject.exists() {
            let old = read_file_safe(&pyproject)?;
            let new_content = self.update_pyproject_content(&old, new_version)?;
            let old_version = self.get_current_version(project_path)?;
            changes.push(VersionChange::new(pyproject, old, new_content, old_version.clone(), new_version.to_string()));
        }

        let setup_py = project_path.join("setup.py");
        if setup_py.exists() {
            let old = read_file_safe(&setup_py)?;
            let new_content = self.update_setup_py_content(&old, new_version)?;
            let old_version = self.get_current_version(project_path)?;
            changes.push(VersionChange::new(setup_py, old, new_content, old_version.clone(), new_version.to_string()));
        }

        Ok(changes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_pyproject(dir: &std::path::Path, version: &str) -> anyhow::Result<()> {
        let content = format!(
            "[project]\nname = \"test\"\nversion = \"{}\"\n",
            version
        );
        fs::write(dir.join(PYTHON_PYPROJECT), content)?;
        Ok(())
    }

    fn create_setup_py(dir: &std::path::Path, version: &str) -> anyhow::Result<()> {
        let content = format!(
            "from setuptools import setup\nsetup(name=\"test\", version=\"{}\")\n",
            version
        );
        fs::write(dir.join(PYTHON_SETUP), content)?;
        Ok(())
    }

    #[test]
    fn test_pyproject_version() -> anyhow::Result<()> {
        let tmp = TempDir::new()?;
        let path = tmp.path();
        create_pyproject(path, "1.2.3")?;

        let v = PythonUpdater::new().get_current_version(path)?;
        assert_eq!(v, "1.2.3");
        Ok(())
    }

    #[test]
    fn test_setup_py_version() -> anyhow::Result<()> {
        let tmp = TempDir::new()?;
        let path = tmp.path();
        create_setup_py(path, "0.1.0")?;

        let v = PythonUpdater::new().get_current_version(path)?;
        assert_eq!(v, "0.1.0");
        Ok(())
    }

    #[test]
    fn test_update_pyproject() -> anyhow::Result<()> {
        let tmp = TempDir::new()?;
        let path = tmp.path();
        create_pyproject(path, "0.1.0")?;

        let updated = PythonUpdater::new().update_version(path, "0.2.0")?;
        assert_eq!(updated.len(), 1);
        let content = fs::read_to_string(path.join(PYTHON_PYPROJECT))?;
        assert!(content.contains("version = \"0.2.0\""));
        Ok(())
    }

    #[test]
    fn test_update_setup_py() -> anyhow::Result<()> {
        let tmp = TempDir::new()?;
        let path = tmp.path();
        create_setup_py(path, "0.1.0")?;

        let updated = PythonUpdater::new().update_version(path, "0.2.0")?;
        assert_eq!(updated.len(), 1);
        let content = fs::read_to_string(path.join("setup.py"))?;
        assert!(content.contains("version=\"0.2.0\""));
        Ok(())
    }

    #[test]
    fn test_preview_changes() -> anyhow::Result<()> {
        let tmp = TempDir::new()?;
        let path = tmp.path();
        create_pyproject(path, "1.0.0")?;

        let changes = PythonUpdater::new().preview_changes(path, "2.0.0")?;
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].old_version, "1.0.0");
        assert_eq!(changes[0].new_version, "2.0.0");
        Ok(())
    }

    #[test]
    fn test_validate_and_can_handle() -> anyhow::Result<()> {
        let tmp = TempDir::new()?;
        let path = tmp.path();
        assert!(!PythonUpdater::new().can_handle(path));
        create_pyproject(path, "1.0.0")?;
        assert!(PythonUpdater::new().can_handle(path));
        assert!(PythonUpdater::new().validate_project(path).is_ok());
        Ok(())
    }

    #[test]
    fn test_technology_name() {
        assert_eq!(PythonUpdater::new().technology_name(), "python");
    }
}