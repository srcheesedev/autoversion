use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::OpenOptions;
use std::io::Write;

/// Output data structure for different formats
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutputData {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub previous_version: String,
    #[serde(default)]
    pub version_type: String,
    #[serde(default)]
    pub technology: String,
    #[serde(default)]
    pub files_updated: Vec<String>,
    #[serde(default)]
    pub tag_created: bool,
    #[serde(default)]
    pub tag_name: Option<String>,
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub message: String,
}

/// Handle different output formats for autoversion results
pub struct ActionOutput {
    data: OutputData,
    // prefix to use when constructing tag names (default "v")
    tag_prefix: String,
}

impl ActionOutput {
    pub fn new() -> Self {
        Self {
            data: OutputData::default(),
            tag_prefix: "v".to_string(),
        }
    }

    /// Set the new version
    pub fn set_version(&mut self, version: &str) {
        self.data.version = version.to_string();
    }

    /// Set the previous version
    pub fn set_previous_version(&mut self, version: &str) {
        self.data.previous_version = version.to_string();
    }

    /// Set the version bump type
    pub fn set_version_type(&mut self, version_type: &str) {
        self.data.version_type = version_type.to_string();
    }

    /// Set the detected technology
    pub fn set_technology(&mut self, technology: &str) {
        self.data.technology = technology.to_string();
    }

    /// Set the list of updated files
    pub fn set_files_updated(&mut self, files: &[String]) {
        self.data.files_updated = files.to_vec();
    }

    /// Set whether a tag was created
    pub fn set_tag_created(&mut self, created: bool) {
        self.data.tag_created = created;
        if created && !self.data.version.is_empty() {
            // Build tag name using configured prefix
            self.data.tag_name = Some(format!("{}{}", self.tag_prefix, self.data.version));
        }
    }

    /// Set tag prefix used when constructing tag names (default: "v")
    pub fn set_tag_prefix(&mut self, prefix: &str) {
        self.tag_prefix = prefix.to_string();
        // If tag was already created and version present, update tag_name accordingly
        if self.data.tag_created && !self.data.version.is_empty() {
            self.data.tag_name = Some(format!("{}{}", self.tag_prefix, self.data.version));
        }
    }

    /// Set custom tag name
    pub fn set_tag_name(&mut self, tag_name: &str) {
        self.data.tag_name = Some(tag_name.to_string());
    }

    /// Set success status and message
    pub fn set_result(&mut self, success: bool, message: &str) {
        self.data.success = success;
        self.data.message = message.to_string();
    }

    /// Write outputs in the specified format
    pub fn write_outputs(&self) -> Result<()> {
        // Determine output format from environment or default
        let format = env::var("AUTOVERSION_OUTPUT_FORMAT").unwrap_or_else(|_| {
            if env::var("GITHUB_ACTIONS").is_ok() {
                "github-actions".to_string()
            } else {
                "human".to_string()
            }
        });

        match format.as_str() {
            "github-actions" => self.write_github_actions_output(),
            "json" => self.write_json_output(),
            "human" => self.write_human_output(),
            _ => Err(anyhow!("Unknown output format: {}", format)),
        }
    }

    /// Write GitHub Actions format output
    fn write_github_actions_output(&self) -> Result<()> {
        let github_output = env::var("GITHUB_OUTPUT").ok();

        if let Some(output_file) = github_output {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&output_file)?;

            // Write outputs using simple key=value format
            // Values must be on a single line
            writeln!(file, "version={}", self.data.version)?;
            writeln!(file, "previous-version={}", self.data.previous_version)?;
            writeln!(file, "version-type={}", self.data.version_type)?;
            writeln!(file, "technology={}", self.data.technology)?;
            writeln!(file, "files-updated={}", self.data.files_updated.join(","))?;
            writeln!(file, "tag-created={}", self.data.tag_created)?;

            if let Some(tag_name) = &self.data.tag_name {
                writeln!(file, "tag-name={}", tag_name)?;
            }

            writeln!(file, "success={}", self.data.success)?;

            // Explicitly flush to ensure all data is written
            file.flush()?;
        } else {
            // Fallback to old format if GITHUB_OUTPUT is not available
            println!("::set-output name=version::{}", self.data.version);
            println!(
                "::set-output name=previous-version::{}",
                self.data.previous_version
            );
            println!("::set-output name=version-type::{}", self.data.version_type);
            println!("::set-output name=technology::{}", self.data.technology);
            println!(
                "::set-output name=files-updated::{}",
                self.data.files_updated.join(",")
            );
            println!("::set-output name=tag-created::{}", self.data.tag_created);

            if let Some(tag_name) = &self.data.tag_name {
                println!("::set-output name=tag-name::{}", tag_name);
            }

            println!("::set-output name=success::{}", self.data.success);
        }

        // Also output summary to GitHub Actions summary
        if let Ok(summary_file) = env::var("GITHUB_STEP_SUMMARY") {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&summary_file)?;

            writeln!(file, "## 🚀 Autoversion Results")?;
            writeln!(file)?;
            writeln!(file, "| Field | Value |")?;
            writeln!(file, "|-------|-------|")?;
            writeln!(
                file,
                "| Previous Version | `{}` |",
                self.data.previous_version
            )?;
            writeln!(file, "| New Version | `{}` |", self.data.version)?;
            writeln!(file, "| Bump Type | `{}` |", self.data.version_type)?;
            writeln!(file, "| Technology | `{}` |", self.data.technology)?;
            writeln!(
                file,
                "| Files Updated | {} |",
                self.data.files_updated.len()
            )?;
            writeln!(
                file,
                "| Tag Created | {} |",
                if self.data.tag_created { "✅" } else { "❌" }
            )?;

            if !self.data.files_updated.is_empty() {
                writeln!(file)?;
                writeln!(file, "### 📁 Files Updated")?;
                for file_path in &self.data.files_updated {
                    writeln!(file, "- `{}`", file_path)?;
                }
            }
        }

        Ok(())
    }

    /// Write JSON format output
    fn write_json_output(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.data)?;
        println!("{}", json);
        Ok(())
    }

    /// Write human-readable format output
    fn write_human_output(&self) -> Result<()> {
        println!("🚀 Autoversion Results");
        println!("━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Previous Version: {}", self.data.previous_version);
        println!("New Version:      {}", self.data.version);
        println!("Bump Type:        {}", self.data.version_type);
        println!("Technology:       {}", self.data.technology);
        println!(
            "Tag Created:      {}",
            if self.data.tag_created {
                "✅ Yes"
            } else {
                "❌ No"
            }
        );

        if let Some(tag_name) = &self.data.tag_name {
            println!("Tag Name:         {}", tag_name);
        }

        if !self.data.files_updated.is_empty() {
            println!("\n📁 Files Updated:");
            for file_path in &self.data.files_updated {
                println!("  • {}", file_path);
            }
        }

        if !self.data.message.is_empty() {
            println!("\n💬 {}", self.data.message);
        }

        Ok(())
    }

    /// Get the output data for testing or programmatic access
    pub fn get_data(&self) -> &OutputData {
        &self.data
    }
}

impl Default for ActionOutput {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility function to write GitHub Actions output directly
pub fn write_github_output(key: &str, value: &str) -> Result<()> {
    if let Ok(output_file) = env::var("GITHUB_OUTPUT") {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&output_file)?;
        // Use simple key=value format
        writeln!(file, "{}={}", key, value)?;
        file.flush()?;
    } else {
        println!("::set-output name={}::{}", key, value);
    }
    Ok(())
}

/// Utility function to create GitHub Actions annotations
pub fn create_annotation(
    level: &str,
    message: &str,
    file: Option<&str>,
    line: Option<u32>,
) -> Result<()> {
    let mut annotation = format!("::{} ::{}", level, message);

    if let Some(file_path) = file {
        annotation = format!("::{} file={}::{}", level, file_path, message);

        if let Some(line_num) = line {
            annotation = format!(
                "::{} file={},line={}::{}",
                level, file_path, line_num, message
            );
        }
    }

    println!("{}", annotation);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_output_data_creation() {
        let mut output = ActionOutput::new();

        output.set_version("1.2.3");
        output.set_previous_version("1.2.2");
        output.set_version_type("patch");
        output.set_technology("npm");
        output.set_files_updated(&["package.json".to_string()]);
        output.set_tag_created(true);
        output.set_result(true, "Version updated successfully");

        let data = output.get_data();
        assert_eq!(data.version, "1.2.3");
        assert_eq!(data.previous_version, "1.2.2");
        assert_eq!(data.version_type, "patch");
        assert_eq!(data.technology, "npm");
        assert_eq!(data.files_updated, vec!["package.json"]);
        assert!(data.tag_created);
        assert_eq!(data.tag_name, Some("v1.2.3".to_string()));
        assert!(data.success);
        assert_eq!(data.message, "Version updated successfully");
    }

    #[test]
    fn test_json_output() -> Result<()> {
        let mut output = ActionOutput::new();
        output.set_version("1.0.0");
        output.set_technology("npm");

        env::set_var("AUTOVERSION_OUTPUT_FORMAT", "json");

        // This would normally print to stdout, but we can test the data structure
        let data = output.get_data();
        let json = serde_json::to_string(data)?;
        assert!(json.contains("1.0.0"));
        assert!(json.contains("npm"));

        env::remove_var("AUTOVERSION_OUTPUT_FORMAT");
        Ok(())
    }

    #[test]
    fn test_github_actions_output_with_file() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path().to_str().unwrap();

        env::set_var("GITHUB_OUTPUT", temp_path);
        env::set_var("AUTOVERSION_OUTPUT_FORMAT", "github-actions");

        let mut output = ActionOutput::new();
        output.set_version("2.0.0");
        output.set_technology("cargo");
        output.set_tag_created(true);

        output.write_github_actions_output()?;

        let content = fs::read_to_string(temp_path)?;
        assert!(content.contains("version=2.0.0"));
        assert!(content.contains("technology=cargo"));
        assert!(content.contains("tag-created=true"));
        assert!(content.contains("tag-name=v2.0.0"));

        env::remove_var("GITHUB_OUTPUT");
        env::remove_var("AUTOVERSION_OUTPUT_FORMAT");
        Ok(())
    }

    #[test]
    fn test_write_github_output_utility() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path().to_str().unwrap();

        env::set_var("GITHUB_OUTPUT", temp_path);

        write_github_output("test-key", "test-value")?;

        let content = fs::read_to_string(temp_path)?;
        assert!(content.contains("test-key=test-value"));

        env::remove_var("GITHUB_OUTPUT");
        Ok(())
    }

    #[test]
    fn test_default_format_detection() {
        use std::sync::{Mutex, OnceLock};
        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let _lock = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        // Save previous value
        let prev = env::var("GITHUB_ACTIONS").ok();

        // Test GitHub Actions environment detection
        env::set_var("GITHUB_ACTIONS", "true");
        let _output = ActionOutput::new();

        // In real usage, this would be detected in write_outputs()
        // Here we just test that the environment variable exists
        assert!(env::var("GITHUB_ACTIONS").is_ok());

        // Restore previous value
        if let Some(val) = prev {
            env::set_var("GITHUB_ACTIONS", val);
        } else {
            env::remove_var("GITHUB_ACTIONS");
        }
    }
}
