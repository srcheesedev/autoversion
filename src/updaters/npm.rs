use anyhow::{anyhow, Result};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

use super::traits::{VersionUpdater, VersionChange};
use crate::utils::files::backup_file;

/// NPM package.json version updater
pub struct NpmUpdater;

impl NpmUpdater {
    pub fn new() -> Self {
        Self
    }

    /// Parse package.json and extract version
    fn parse_package_json(&self, content: &str) -> Result<Value> {
        serde_json::from_str(content)
            .map_err(|e| anyhow!("Failed to parse package.json: {}", e))
    }

    /// Update version in package.json content
    fn update_package_json_content(&self, content: &str, new_version: &str) -> Result<String> {
        let mut package_json: Value = self.parse_package_json(content)?;
        
        // Update version field
        if let Some(obj) = package_json.as_object_mut() {
            obj.insert("version".to_string(), Value::String(new_version.to_string()));
        } else {
            return Err(anyhow!("package.json root is not an object"));
        }

        // Serialize back to JSON with proper formatting
        let updated_content = serde_json::to_string_pretty(&package_json)?;
        Ok(updated_content + "\n") // Add trailing newline
    }

    /// Check for additional NPM-related files to update
    fn find_related_files(&self, project_path: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        
        // package-lock.json (NPM)
        let package_lock = project_path.join("package-lock.json");
        if package_lock.exists() && package_lock.is_file() {
            files.push(package_lock);
        }
        
        // yarn.lock doesn't contain version info, but we might want to note it
        // npm-shrinkwrap.json (if it exists)
        let shrinkwrap = project_path.join("npm-shrinkwrap.json");
        if shrinkwrap.exists() && shrinkwrap.is_file() {
            files.push(shrinkwrap);
        }
        
        files
    }

    /// Update package-lock.json version
    fn update_package_lock(&self, file_path: &PathBuf, new_version: &str) -> Result<()> {
        let content = fs::read_to_string(file_path)?;
        let mut lock_json: Value = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse package-lock.json: {}", e))?;

        if let Some(obj) = lock_json.as_object_mut() {
            // Update version at root level
            obj.insert("version".to_string(), Value::String(new_version.to_string()));
            
            // Update version in packages."" (root package)
            if let Some(packages) = obj.get_mut("packages") {
                if let Some(packages_obj) = packages.as_object_mut() {
                    if let Some(root_package) = packages_obj.get_mut("") {
                        if let Some(root_obj) = root_package.as_object_mut() {
                            root_obj.insert("version".to_string(), Value::String(new_version.to_string()));
                        }
                    }
                }
            }
        }

        let updated_content = serde_json::to_string_pretty(&lock_json)?;
        fs::write(file_path, updated_content + "\n")?;
        
        Ok(())
    }
}

impl Default for NpmUpdater {
    fn default() -> Self {
        Self::new()
    }
}

impl VersionUpdater for NpmUpdater {
    fn get_current_version(&self, project_path: &Path) -> Result<String> {
        let package_json_path = project_path.join("package.json");
        let content = fs::read_to_string(&package_json_path)
            .map_err(|e| anyhow!("Failed to read package.json: {}", e))?;

        let package_json = self.parse_package_json(&content)?;
        
        package_json
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("No version field found in package.json"))
            .map(|s| s.to_string())
    }

    fn update_version(&self, project_path: &Path, new_version: &str) -> Result<Vec<String>> {
        let package_json_path = project_path.join("package.json");
        let mut updated_files = Vec::new();

        // Backup original file
        backup_file(&package_json_path)?;

        // Update package.json
        let content = fs::read_to_string(&package_json_path)?;
        let updated_content = self.update_package_json_content(&content, new_version)?;
        fs::write(&package_json_path, updated_content)?;
        updated_files.push(package_json_path.to_string_lossy().to_string());

        // Update related files
        let related_files = self.find_related_files(project_path);
        for file_path in related_files {
            if file_path.file_name().unwrap() == "package-lock.json" {
                backup_file(&file_path)?;
                self.update_package_lock(&file_path, new_version)?;
                updated_files.push(file_path.to_string_lossy().to_string());
            }
        }

        Ok(updated_files)
    }

    fn validate_project(&self, project_path: &Path) -> Result<()> {
        let package_json_path = project_path.join("package.json");
        
        if !package_json_path.exists() {
            return Err(anyhow!("package.json not found in {}", project_path.display()));
        }

        let content = fs::read_to_string(&package_json_path)?;
        let package_json = self.parse_package_json(&content)?;

        if !package_json.get("version").and_then(|v| v.as_str()).is_some() {
            return Err(anyhow!("package.json does not contain a valid version field"));
        }

        Ok(())
    }

    fn technology_name(&self) -> &'static str {
        "npm"
    }

    fn get_primary_file(&self, project_path: &Path) -> Result<PathBuf> {
        let package_json_path = project_path.join("package.json");
        if package_json_path.exists() {
            Ok(package_json_path)
        } else {
            Err(anyhow!("package.json not found"))
        }
    }

    fn can_handle(&self, project_path: &Path) -> bool {
        project_path.join("package.json").exists()
    }

    fn preview_changes(&self, project_path: &Path, new_version: &str) -> Result<Vec<VersionChange>> {
        let mut changes = Vec::new();
        
        // Preview package.json changes
        let package_json_path = project_path.join("package.json");
        if package_json_path.exists() {
            let old_content = fs::read_to_string(&package_json_path)?;
            let new_content = self.update_package_json_content(&old_content, new_version)?;
            let old_version = self.get_current_version(project_path)?;
            
            changes.push(VersionChange::new(
                package_json_path,
                old_content,
                new_content,
                old_version.clone(),
                new_version.to_string(),
            ));
        }

        // Preview package-lock.json changes
        let related_files = self.find_related_files(project_path);
        for file_path in related_files {
            if file_path.file_name().unwrap() == "package-lock.json" {
                let old_content = fs::read_to_string(&file_path)?;
                
                // Simulate the update to get new content
                let mut lock_json: Value = serde_json::from_str(&old_content)?;
                if let Some(obj) = lock_json.as_object_mut() {
                    obj.insert("version".to_string(), Value::String(new_version.to_string()));
                }
                let new_content = serde_json::to_string_pretty(&lock_json)? + "\n";
                
                let old_version = self.get_current_version(project_path)?;
                changes.push(VersionChange::new(
                    file_path,
                    old_content,
                    new_content,
                    old_version,
                    new_version.to_string(),
                ));
            }
        }

        Ok(changes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_package_json(dir: &Path, version: &str) -> Result<()> {
        let content = format!(
            r#"{{
  "name": "test-package",
  "version": "{}",
  "description": "Test package",
  "main": "index.js",
  "dependencies": {{
    "lodash": "^4.17.21"
  }}
}}"#,
            version
        );
        fs::write(dir.join("package.json"), content)?;
        Ok(())
    }

    fn create_package_lock(dir: &Path, version: &str) -> Result<()> {
        let content = format!(
            r#"{{
  "name": "test-package",
  "version": "{}",
  "lockfileVersion": 2,
  "requires": true,
  "packages": {{
    "": {{
      "name": "test-package",
      "version": "{}",
      "dependencies": {{
        "lodash": "^4.17.21"
      }}
    }}
  }}
}}"#,
            version, version
        );
        fs::write(dir.join("package-lock.json"), content)?;
        Ok(())
    }

    #[test]
    fn test_get_current_version() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();
        
        create_package_json(project_path, "1.2.3")?;
        
        let updater = NpmUpdater::new();
        let version = updater.get_current_version(project_path)?;
        
        assert_eq!(version, "1.2.3");
        Ok(())
    }

    #[test]
    fn test_update_version() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();
        
        create_package_json(project_path, "1.2.3")?;
        create_package_lock(project_path, "1.2.3")?;
        
        let updater = NpmUpdater::new();
        let updated_files = updater.update_version(project_path, "1.3.0")?;
        
        // Should update both package.json and package-lock.json
        assert_eq!(updated_files.len(), 2);
        
        // Verify package.json was updated
        let new_version = updater.get_current_version(project_path)?;
        assert_eq!(new_version, "1.3.0");
        
        // Verify package-lock.json was updated
        let lock_content = fs::read_to_string(project_path.join("package-lock.json"))?;
        assert!(lock_content.contains(r#""version": "1.3.0""#));
        
        Ok(())
    }

    #[test]
    fn test_validate_project_valid() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();
        
        create_package_json(project_path, "1.0.0")?;
        
        let updater = NpmUpdater::new();
        let result = updater.validate_project(project_path);
        
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_validate_project_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        let updater = NpmUpdater::new();
        let result = updater.validate_project(project_path);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("package.json not found"));
    }

    #[test]
    fn test_validate_project_no_version() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();
        
        // Create package.json without version
        fs::write(
            project_path.join("package.json"),
            r#"{"name": "test"}"#
        )?;
        
        let updater = NpmUpdater::new();
        let result = updater.validate_project(project_path);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not contain a valid version field"));
        Ok(())
    }

    #[test]
    fn test_can_handle() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();
        
        let updater = NpmUpdater::new();
        
        // Should not handle without package.json
        assert!(!updater.can_handle(project_path));
        
        // Should handle with package.json
        create_package_json(project_path, "1.0.0")?;
        assert!(updater.can_handle(project_path));
        
        Ok(())
    }

    #[test]
    fn test_preview_changes() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();
        
        create_package_json(project_path, "1.0.0")?;
        
        let updater = NpmUpdater::new();
        let changes = updater.preview_changes(project_path, "2.0.0")?;
        
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].old_version, "1.0.0");
        assert_eq!(changes[0].new_version, "2.0.0");
        assert!(changes[0].new_content.contains(r#""version": "2.0.0""#));
        
        Ok(())
    }

    #[test]
    fn test_technology_name() {
        let updater = NpmUpdater::new();
        assert_eq!(updater.technology_name(), "npm");
    }
}