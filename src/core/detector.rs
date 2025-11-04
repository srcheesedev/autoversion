use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::fs;

/// Supported technology types
#[derive(Debug, Clone, PartialEq)]
pub enum Technology {
    Npm,
    Cargo,
    Maven,
    Python,
    Go,
    Composer,
    Generic,
}

impl std::fmt::Display for Technology {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Technology::Npm => write!(f, "npm"),
            Technology::Cargo => write!(f, "cargo"),
            Technology::Maven => write!(f, "maven"),
            Technology::Python => write!(f, "python"),
            Technology::Go => write!(f, "go"),
            Technology::Composer => write!(f, "composer"),
            Technology::Generic => write!(f, "generic"),
        }
    }
}

impl std::str::FromStr for Technology {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "npm" => Ok(Technology::Npm),
            "cargo" => Ok(Technology::Cargo),
            "maven" => Ok(Technology::Maven),
            "python" => Ok(Technology::Python),
            "go" | "golang" => Ok(Technology::Go),
            "composer" | "php" => Ok(Technology::Composer),
            "generic" => Ok(Technology::Generic),
            _ => Err(anyhow!("Unsupported technology: {}. Supported: npm, cargo, maven, python, go, composer, generic", s)),
        }
    }
}

/// Detection result with confidence score
#[derive(Debug)]
pub struct DetectionResult {
    pub technology: Technology,
    pub confidence: f32,
    pub files_found: Vec<PathBuf>,
    pub primary_file: PathBuf,
}

/// Technology detection patterns
struct DetectionPattern {
    technology: Technology,
    files: Vec<&'static str>,
    priority: u8, // Higher number = higher priority
}

/// Detects project technology based on file presence
pub struct TechnologyDetector {
    patterns: Vec<DetectionPattern>,
}

impl TechnologyDetector {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                // Order by priority (higher priority first)
                DetectionPattern {
                    technology: Technology::Npm,
                    files: vec!["package.json"],
                    priority: 90,
                },
                DetectionPattern {
                    technology: Technology::Cargo,
                    files: vec!["Cargo.toml"],
                    priority: 85,
                },
                DetectionPattern {
                    technology: Technology::Maven,
                    files: vec!["pom.xml"],
                    priority: 80,
                },
                DetectionPattern {
                    technology: Technology::Python,
                    files: vec!["pyproject.toml", "setup.py", "setup.cfg"],
                    priority: 75,
                },
                DetectionPattern {
                    technology: Technology::Go,
                    files: vec!["go.mod"],
                    priority: 85,
                },
                DetectionPattern {
                    technology: Technology::Composer,
                    files: vec!["composer.json"],
                    priority: 80,
                },
                DetectionPattern {
                    technology: Technology::Generic,
                    files: vec!["VERSION", "version.txt", ".version"],
                    priority: 10, // Lowest priority - fallback
                },
            ],
        }
    }

    /// Auto-detect technology from project directory
    pub fn detect(&self, project_path: &Path) -> Result<String> {
        let results = self.detect_all(project_path)?;
        
        if results.is_empty() {
            return Err(anyhow!(
                "No supported technology detected in {}. Supported: npm (package.json), cargo (Cargo.toml), maven (pom.xml), python (pyproject.toml), go (go.mod), composer (composer.json), generic (VERSION)",
                project_path.display()
            ));
        }

        // Return the highest priority detection
        let best_result = results.into_iter()
            .max_by_key(|r| (r.confidence * 100.0) as u32)
            .unwrap();

        Ok(best_result.technology.to_string())
    }

    /// Detect all possible technologies in the project
    pub fn detect_all(&self, project_path: &Path) -> Result<Vec<DetectionResult>> {
        let mut results = Vec::new();

        for pattern in &self.patterns {
            let mut files_found = Vec::new();
            let mut primary_file = None;

            for &file_name in &pattern.files {
                let file_path = project_path.join(file_name);
                if file_path.exists() && file_path.is_file() {
                    if primary_file.is_none() {
                        primary_file = Some(file_path.clone());
                    }
                    files_found.push(file_path);
                }
            }

            if !files_found.is_empty() {
                let confidence = self.calculate_confidence(&pattern, &files_found);
                results.push(DetectionResult {
                    technology: pattern.technology.clone(),
                    confidence,
                    files_found,
                    primary_file: primary_file.unwrap(),
                });
            }
        }

        // Sort by confidence (highest first)
        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        Ok(results)
    }

    /// Check if a specific technology is present
    pub fn has_technology(&self, project_path: &Path, technology: &Technology) -> Result<bool> {
        let pattern = self.patterns.iter()
            .find(|p| &p.technology == technology)
            .ok_or_else(|| anyhow!("Unknown technology: {:?}", technology))?;

        for &file_name in &pattern.files {
            let file_path = project_path.join(file_name);
            if file_path.exists() && file_path.is_file() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Get the primary file for a technology
    pub fn get_primary_file(&self, project_path: &Path, technology: &Technology) -> Result<PathBuf> {
        let pattern = self.patterns.iter()
            .find(|p| &p.technology == technology)
            .ok_or_else(|| anyhow!("Unknown technology: {:?}", technology))?;

        for &file_name in &pattern.files {
            let file_path = project_path.join(file_name);
            if file_path.exists() && file_path.is_file() {
                return Ok(file_path);
            }
        }

        Err(anyhow!("No {} files found in {}", technology, project_path.display()))
    }

    /// Calculate confidence score for detection
    fn calculate_confidence(&self, pattern: &DetectionPattern, files_found: &[PathBuf]) -> f32 {
        let base_confidence = pattern.priority as f32 / 100.0;
        
        // Bonus for multiple files found
        let file_bonus = (files_found.len() as f32 - 1.0) * 0.1;
        
        // Bonus for specific file combinations
        let combination_bonus = self.calculate_combination_bonus(&pattern.technology, files_found);
        
        (base_confidence + file_bonus + combination_bonus).min(1.0)
    }

    /// Calculate bonus for specific file combinations
    fn calculate_combination_bonus(&self, technology: &Technology, files_found: &[PathBuf]) -> f32 {
        match technology {
            Technology::Python => {
                // Prefer pyproject.toml over setup.py
                if files_found.iter().any(|f| f.file_name().unwrap() == "pyproject.toml") {
                    0.1
                } else {
                    0.0
                }
            }
            Technology::Npm => {
                // Check for package-lock.json or yarn.lock for additional confidence
                let project_path = files_found[0].parent().unwrap();
                if project_path.join("package-lock.json").exists() || 
                   project_path.join("yarn.lock").exists() {
                    0.1
                } else {
                    0.0
                }
            }
            Technology::Cargo => {
                // Check for Cargo.lock for additional confidence
                let project_path = files_found[0].parent().unwrap();
                if project_path.join("Cargo.lock").exists() {
                    0.1
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

    /// Validate that the detected files are readable and contain version info
    pub fn validate_detection(&self, result: &DetectionResult) -> Result<()> {
        // Check if primary file is readable
        let content = fs::read_to_string(&result.primary_file)
            .map_err(|e| anyhow!("Cannot read {}: {}", result.primary_file.display(), e))?;

        // Basic validation that file contains version-like content
        match result.technology {
            Technology::Npm => {
                if !content.contains("\"version\"") {
                    return Err(anyhow!("package.json does not contain version field"));
                }
            }
            Technology::Cargo => {
                if !content.contains("version") {
                    return Err(anyhow!("Cargo.toml does not contain version field"));
                }
            }
            Technology::Maven => {
                if !content.contains("<version>") {
                    return Err(anyhow!("pom.xml does not contain version element"));
                }
            }
            Technology::Python => {
                // pyproject.toml should have version, setup.py might have it in different forms
                if result.primary_file.file_name().unwrap() == "pyproject.toml" && !content.contains("version") {
                    return Err(anyhow!("pyproject.toml does not contain version field"));
                }
            }
            Technology::Go => {
                // go.mod doesn't contain module version (uses git tags)
                // Just validate it's a valid go.mod file
                if !content.contains("module ") {
                    return Err(anyhow!("go.mod does not contain module declaration"));
                }
            }
            Technology::Composer => {
                // composer.json may or may not have version field
                // Just validate it's valid JSON
                if !content.contains("{") {
                    return Err(anyhow!("composer.json is not valid JSON"));
                }
            }
            Technology::Generic => {
                // Generic files should contain some version-like content
                if content.trim().is_empty() {
                    return Err(anyhow!("Version file is empty"));
                }
            }
        }

        Ok(())
    }
}

impl Default for TechnologyDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> Result<()> {
        fs::write(dir.join(name), content)?;
        Ok(())
    }

    #[test]
    fn test_npm_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();

        create_test_file(project_path, "package.json", r#"{"version": "1.0.0"}"#)?;

        let detector = TechnologyDetector::new();
        let technology = detector.detect(project_path)?;

        assert_eq!(technology, "npm");
        Ok(())
    }

    #[test]
    fn test_cargo_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();

        create_test_file(project_path, "Cargo.toml", r#"[package]
name = "test"
version = "1.0.0""#)?;

        let detector = TechnologyDetector::new();
        let technology = detector.detect(project_path)?;

        assert_eq!(technology, "cargo");
        Ok(())
    }

    #[test]
    fn test_maven_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();

        create_test_file(project_path, "pom.xml", r#"<project>
<version>1.0.0</version>
</project>"#)?;

        let detector = TechnologyDetector::new();
        let technology = detector.detect(project_path)?;

        assert_eq!(technology, "maven");
        Ok(())
    }

    #[test]
    fn test_python_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();

        create_test_file(project_path, "pyproject.toml", r#"[project]
version = "1.0.0""#)?;

        let detector = TechnologyDetector::new();
        let technology = detector.detect(project_path)?;

        assert_eq!(technology, "python");
        Ok(())
    }

    #[test]
    fn test_generic_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();

        create_test_file(project_path, "VERSION", "1.0.0")?;

        let detector = TechnologyDetector::new();
        let technology = detector.detect(project_path)?;

        assert_eq!(technology, "generic");
        Ok(())
    }

    #[test]
    fn test_priority_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();

        // Create both npm and generic files
        create_test_file(project_path, "package.json", r#"{"version": "1.0.0"}"#)?;
        create_test_file(project_path, "VERSION", "1.0.0")?;

        let detector = TechnologyDetector::new();
        let technology = detector.detect(project_path)?;

        // npm should have higher priority than generic
        assert_eq!(technology, "npm");
        Ok(())
    }

    #[test]
    fn test_no_technology_detected() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let detector = TechnologyDetector::new();
        let result = detector.detect(project_path);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No supported technology detected"));
    }

    #[test]
    fn test_technology_parsing() {
        assert_eq!("npm".parse::<Technology>().unwrap(), Technology::Npm);
        assert_eq!("cargo".parse::<Technology>().unwrap(), Technology::Cargo);
        assert_eq!("maven".parse::<Technology>().unwrap(), Technology::Maven);
        assert_eq!("python".parse::<Technology>().unwrap(), Technology::Python);
        assert_eq!("generic".parse::<Technology>().unwrap(), Technology::Generic);
        
        assert!("invalid".parse::<Technology>().is_err());
    }

    #[test]
    fn test_has_technology() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();

        create_test_file(project_path, "package.json", r#"{"version": "1.0.0"}"#)?;

        let detector = TechnologyDetector::new();
        
        assert!(detector.has_technology(project_path, &Technology::Npm)?);
        assert!(!detector.has_technology(project_path, &Technology::Cargo)?);
        
        Ok(())
    }
}