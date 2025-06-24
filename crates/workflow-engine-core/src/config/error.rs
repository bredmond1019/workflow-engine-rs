//! Configuration error types
//!
//! This module provides the error types for configuration management.

use thiserror::Error;

/// Configuration errors with rich context and error chaining
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Environment variable not found or empty
    #[error("Environment variable '{var_name}' not found{}", context.as_ref().map(|c| format!(" ({})", c)).unwrap_or_default())]
    EnvVarNotFound {
        /// Name of the missing environment variable
        var_name: String,
        /// Additional context about where this variable is used
        context: Option<String>,
    },
    
    /// Configuration value is invalid for the given key
    #[error("Invalid configuration value for '{key}' from {source}: got '{value}', expected {expected_format}")]
    InvalidValue {
        /// Configuration key
        key: String,
        /// Invalid value that was provided
        value: String,
        /// Expected format or valid values
        expected_format: String,
        /// Source of the configuration (env, file, CLI, etc.)
        source: String,
        /// Underlying parsing error if available
        #[source]
        parse_error: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Configuration validation failed
    #[error("Validation failed for {config_section}: {message}. Hint: {hint}")]
    ValidationFailed {
        /// Descriptive error message
        message: String,
        /// Configuration section that failed validation
        config_section: String,
        /// Helpful hint for resolving the issue
        hint: String,
        /// Invalid fields and their values
        invalid_fields: Vec<(String, String)>,
    },
    
    /// Error parsing configuration data
    #[error("Parsing error in {config_source} at {location}: {message}")]
    ParseError {
        /// Descriptive error message
        message: String,
        /// Source of the configuration (file path, env var name, etc.)
        config_source: String,
        /// Location within the source (line number, JSON path, etc.)
        location: String,
        /// Underlying parsing error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Required configuration field is missing
    #[error("Required field '{field_name}' missing from {config_source}. {description}")]
    RequiredFieldMissing {
        /// Name of the missing field
        field_name: String,
        /// Source where the field should be defined
        config_source: String,
        /// Description of what this field is used for
        description: String,
        /// Example value for this field
        example_value: Option<String>,
    },
    
    /// Configuration file or source not found
    #[error("Configuration source '{source_path}' not found. {suggestion}")]
    SourceNotFound {
        /// Path or identifier of the missing source
        source_path: String,
        /// Suggestion for resolving the issue
        suggestion: String,
        /// Underlying I/O error
        #[source]
        io_error: Option<std::io::Error>,
    },
    
    /// Configuration contains conflicting values
    #[error("Configuration conflict: {description}. Conflicting sources: {}", conflicting_sources.join(", "))]
    ConflictingValues {
        /// Description of the conflict
        description: String,
        /// Sources that have conflicting values
        conflicting_sources: Vec<String>,
        /// The conflicting key
        key: String,
        /// The conflicting values
        values: Vec<String>,
    },
}

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;

impl ConfigError {
    /// Create an environment variable not found error
    pub fn env_var_not_found(var_name: impl Into<String>, context: Option<String>) -> Self {
        Self::EnvVarNotFound {
            var_name: var_name.into(),
            context,
        }
    }

    /// Create an invalid value error
    pub fn invalid_value(
        key: impl Into<String>,
        value: impl Into<String>,
        expected_format: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self::InvalidValue {
            key: key.into(),
            value: value.into(),
            expected_format: expected_format.into(),
            source: source.into(),
            parse_error: None,
        }
    }

    /// Create an invalid value error with underlying parse error
    pub fn invalid_value_with_source(
        key: impl Into<String>,
        value: impl Into<String>,
        expected_format: impl Into<String>,
        source: impl Into<String>,
        parse_error: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        Self::InvalidValue {
            key: key.into(),
            value: value.into(),
            expected_format: expected_format.into(),
            source: source.into(),
            parse_error: Some(parse_error),
        }
    }

    /// Create a validation failed error
    pub fn validation_failed(
        message: impl Into<String>,
        config_section: impl Into<String>,
        hint: impl Into<String>,
        invalid_fields: Vec<(String, String)>,
    ) -> Self {
        Self::ValidationFailed {
            message: message.into(),
            config_section: config_section.into(),
            hint: hint.into(),
            invalid_fields,
        }
    }

    /// Create a parse error
    pub fn parse_error(
        message: impl Into<String>,
        config_source: impl Into<String>,
        location: impl Into<String>,
    ) -> Self {
        Self::ParseError {
            message: message.into(),
            config_source: config_source.into(),
            location: location.into(),
            source: None,
        }
    }

    /// Create a parse error with underlying source
    pub fn parse_error_with_source(
        message: impl Into<String>,
        config_source: impl Into<String>,
        location: impl Into<String>,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        Self::ParseError {
            message: message.into(),
            config_source: config_source.into(),
            location: location.into(),
            source: Some(source),
        }
    }

    /// Create a required field missing error
    pub fn required_field_missing(
        field_name: impl Into<String>,
        config_source: impl Into<String>,
        description: impl Into<String>,
        example_value: Option<String>,
    ) -> Self {
        Self::RequiredFieldMissing {
            field_name: field_name.into(),
            config_source: config_source.into(),
            description: description.into(),
            example_value,
        }
    }

    /// Create a source not found error
    pub fn source_not_found(
        source_path: impl Into<String>,
        suggestion: impl Into<String>,
        io_error: Option<std::io::Error>,
    ) -> Self {
        Self::SourceNotFound {
            source_path: source_path.into(),
            suggestion: suggestion.into(),
            io_error,
        }
    }

    /// Create a conflicting values error
    pub fn conflicting_values(
        description: impl Into<String>,
        conflicting_sources: Vec<String>,
        key: impl Into<String>,
        values: Vec<String>,
    ) -> Self {
        Self::ConflictingValues {
            description: description.into(),
            conflicting_sources,
            key: key.into(),
            values,
        }
    }
}

// Implement error categorization for ConfigError
impl super::super::error::ErrorExt for ConfigError {
    fn category(&self) -> super::super::error::ErrorCategory {
        use super::super::error::ErrorCategory;
        match self {
            Self::EnvVarNotFound { .. } |
            Self::RequiredFieldMissing { .. } |
            Self::InvalidValue { .. } |
            Self::ValidationFailed { .. } |
            Self::ConflictingValues { .. } => ErrorCategory::User,
            
            Self::SourceNotFound { .. } |
            Self::ParseError { .. } => ErrorCategory::System,
        }
    }
    
    fn severity(&self) -> super::super::error::ErrorSeverity {
        use super::super::error::ErrorSeverity;
        match self {
            Self::RequiredFieldMissing { .. } |
            Self::SourceNotFound { .. } => ErrorSeverity::Error,
            
            Self::EnvVarNotFound { .. } |
            Self::InvalidValue { .. } |
            Self::ValidationFailed { .. } |
            Self::ConflictingValues { .. } |
            Self::ParseError { .. } => ErrorSeverity::Warning,
        }
    }
    
    fn error_code(&self) -> &'static str {
        match self {
            Self::EnvVarNotFound { .. } => "CFG_ENV_VAR_NOT_FOUND",
            Self::InvalidValue { .. } => "CFG_INVALID_VALUE",
            Self::ValidationFailed { .. } => "CFG_VALIDATION_FAILED",
            Self::ParseError { .. } => "CFG_PARSE_ERROR",
            Self::RequiredFieldMissing { .. } => "CFG_REQUIRED_FIELD_MISSING",
            Self::SourceNotFound { .. } => "CFG_SOURCE_NOT_FOUND",
            Self::ConflictingValues { .. } => "CFG_CONFLICTING_VALUES",
        }
    }
}