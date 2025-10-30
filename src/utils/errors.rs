use std::fmt;

/// Custom error types for autoversion
#[derive(Debug)]
pub enum AutoversionError {
    /// Version parsing or validation error
    InvalidVersion(String),
    
    /// Technology detection error
    TechnologyNotDetected(String),
    
    /// File operation error
    FileOperation(String),
    
    /// Git operation error
    GitOperation(String),
    
    /// Configuration error
    Configuration(String),
    
    /// Validation error
    Validation(String),
    
    /// Network or external service error
    External(String),
}

impl fmt::Display for AutoversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AutoversionError::InvalidVersion(msg) => write!(f, "Invalid version: {}", msg),
            AutoversionError::TechnologyNotDetected(msg) => write!(f, "Technology detection failed: {}", msg),
            AutoversionError::FileOperation(msg) => write!(f, "File operation failed: {}", msg),
            AutoversionError::GitOperation(msg) => write!(f, "Git operation failed: {}", msg),
            AutoversionError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
            AutoversionError::Validation(msg) => write!(f, "Validation error: {}", msg),
            AutoversionError::External(msg) => write!(f, "External service error: {}", msg),
        }
    }
}

impl std::error::Error for AutoversionError {}

/// Result type alias for autoversion operations
pub type AutoversionResult<T> = Result<T, AutoversionError>;

/// Helper trait for converting common errors to AutoversionError
pub trait IntoAutoversionError<T> {
    fn version_error(self, msg: &str) -> AutoversionResult<T>;
    fn tech_error(self, msg: &str) -> AutoversionResult<T>;
    fn file_error(self, msg: &str) -> AutoversionResult<T>;
    fn git_error(self, msg: &str) -> AutoversionResult<T>;
    fn config_error(self, msg: &str) -> AutoversionResult<T>;
    fn validation_error(self, msg: &str) -> AutoversionResult<T>;
}

impl<T, E> IntoAutoversionError<T> for Result<T, E>
where
    E: std::error::Error,
{
    fn version_error(self, msg: &str) -> AutoversionResult<T> {
        self.map_err(|e| AutoversionError::InvalidVersion(format!("{}: {}", msg, e)))
    }

    fn tech_error(self, msg: &str) -> AutoversionResult<T> {
        self.map_err(|e| AutoversionError::TechnologyNotDetected(format!("{}: {}", msg, e)))
    }

    fn file_error(self, msg: &str) -> AutoversionResult<T> {
        self.map_err(|e| AutoversionError::FileOperation(format!("{}: {}", msg, e)))
    }

    fn git_error(self, msg: &str) -> AutoversionResult<T> {
        self.map_err(|e| AutoversionError::GitOperation(format!("{}: {}", msg, e)))
    }

    fn config_error(self, msg: &str) -> AutoversionResult<T> {
        self.map_err(|e| AutoversionError::Configuration(format!("{}: {}", msg, e)))
    }

    fn validation_error(self, msg: &str) -> AutoversionResult<T> {
        self.map_err(|e| AutoversionError::Validation(format!("{}: {}", msg, e)))
    }
}

/// Convenience functions for creating specific errors
impl AutoversionError {
    pub fn invalid_version(msg: impl Into<String>) -> Self {
        AutoversionError::InvalidVersion(msg.into())
    }

    pub fn technology_not_detected(msg: impl Into<String>) -> Self {
        AutoversionError::TechnologyNotDetected(msg.into())
    }

    pub fn file_operation(msg: impl Into<String>) -> Self {
        AutoversionError::FileOperation(msg.into())
    }

    pub fn git_operation(msg: impl Into<String>) -> Self {
        AutoversionError::GitOperation(msg.into())
    }

    pub fn configuration(msg: impl Into<String>) -> Self {
        AutoversionError::Configuration(msg.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        AutoversionError::Validation(msg.into())
    }

    pub fn external(msg: impl Into<String>) -> Self {
        AutoversionError::External(msg.into())
    }
}

/// Convert anyhow::Error to AutoversionError
impl From<anyhow::Error> for AutoversionError {
    fn from(err: anyhow::Error) -> Self {
        // Try to determine the error type from the error message
        let msg = err.to_string();
        
        if msg.contains("version") || msg.contains("semver") {
            AutoversionError::InvalidVersion(msg)
        } else if msg.contains("git") {
            AutoversionError::GitOperation(msg)
        } else if msg.contains("file") || msg.contains("read") || msg.contains("write") {
            AutoversionError::FileOperation(msg)
        } else if msg.contains("technology") || msg.contains("detect") {
            AutoversionError::TechnologyNotDetected(msg)
        } else if msg.contains("config") {
            AutoversionError::Configuration(msg)
        } else {
            AutoversionError::External(msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let errors = vec![
            AutoversionError::InvalidVersion("bad version".to_string()),
            AutoversionError::TechnologyNotDetected("no tech found".to_string()),
            AutoversionError::FileOperation("file not found".to_string()),
            AutoversionError::GitOperation("git failed".to_string()),
            AutoversionError::Configuration("bad config".to_string()),
            AutoversionError::Validation("invalid input".to_string()),
            AutoversionError::External("network error".to_string()),
        ];

        for error in errors {
            let display = format!("{}", error);
            assert!(!display.is_empty());
            println!("Error: {}", display);
        }
    }

    #[test]
    fn test_convenience_constructors() {
        let error = AutoversionError::invalid_version("test");
        assert!(matches!(error, AutoversionError::InvalidVersion(_)));

        let error = AutoversionError::technology_not_detected("test");
        assert!(matches!(error, AutoversionError::TechnologyNotDetected(_)));

        let error = AutoversionError::file_operation("test");
        assert!(matches!(error, AutoversionError::FileOperation(_)));
    }

    #[test]
    fn test_anyhow_conversion() {
        let anyhow_err = anyhow::anyhow!("version parsing failed");
        let av_err: AutoversionError = anyhow_err.into();
        
        assert!(matches!(av_err, AutoversionError::InvalidVersion(_)));
    }

    #[test]
    fn test_trait_methods() {
        let result: Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found"
        ));
        
        let av_result = result.file_error("Failed to read file");
        assert!(av_result.is_err());
        
        if let Err(AutoversionError::FileOperation(msg)) = av_result {
            assert!(msg.contains("Failed to read file"));
            assert!(msg.contains("file not found"));
        } else {
            panic!("Expected FileOperation error");
        }
    }
}