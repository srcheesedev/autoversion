use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::Result;

/// Configuration for autoversion behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoversionConfig {
    /// Technology to use (overrides auto-detection)
    pub technology: Option<String>,
    
    /// Default bump type when auto-detection fails
    pub default_bump: String,
    
    /// Tag prefix for version tags
    pub tag_prefix: String,
    
    /// Whether to create git tags
    pub create_tag: bool,
    
    /// Files to include in version commit
    pub commit_files: Vec<String>,
    
    /// Custom commit message template
    pub commit_message: Option<String>,
    
    /// Branch patterns where versioning is allowed
    pub allowed_branches: Vec<String>,
}

impl Default for AutoversionConfig {
    fn default() -> Self {
        Self {
            technology: None,
            default_bump: "patch".to_string(),
            tag_prefix: "v".to_string(),
            create_tag: true,
            commit_files: vec![],
            commit_message: None,
            allowed_branches: vec!["main".to_string(), "master".to_string()],
        }
    }
}

impl AutoversionConfig {
    /// Load configuration from .autoversion.toml if it exists
    pub fn load(project_path: &Path) -> Result<Self> {
        let config_path = project_path.join(".autoversion.toml");
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: AutoversionConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }
    
    /// Save configuration to .autoversion.toml
    pub fn save(&self, project_path: &Path) -> Result<()> {
        let config_path = project_path.join(".autoversion.toml");
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }
}