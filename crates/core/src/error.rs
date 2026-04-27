//! Structured error types for Jellyfin CLI.
//!
//! All errors are structured to be machine-readable for Agent consumption.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Error code categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCategory {
    /// Authentication related errors
    Auth,
    /// Network related errors
    Network,
    /// API related errors
    Api,
    /// User input validation errors
    Input,
    /// Internal unexpected errors
    Internal,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auth => write!(f, "AUTH"),
            Self::Network => write!(f, "NETWORK"),
            Self::Api => write!(f, "API"),
            Self::Input => write!(f, "INPUT"),
            Self::Internal => write!(f, "INTERNAL"),
        }
    }
}

/// Jellyfin CLI error with structured output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JellyfinError {
    /// Error category for programmatic handling
    pub category: ErrorCategory,
    /// Specific error code
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional details about the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// Helpful hint for resolution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl JellyfinError {
    /// Create a new error
    pub fn new(
        category: ErrorCategory,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            category,
            code: code.into(),
            message: message.into(),
            details: None,
            hint: None,
        }
    }

    /// Add details to the error
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Add a hint to the error
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    // Auth errors
    pub fn auth_failed(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Auth, "AUTH_FAILED", message)
            .with_hint("Check your credentials with 'jellyfin-agent-cli login'")
    }

    pub fn token_expired() -> Self {
        Self::new(
            ErrorCategory::Auth,
            "TOKEN_EXPIRED",
            "Authentication token has expired",
        )
        .with_hint("Run 'jellyfin-agent-cli login' to refresh your credentials")
    }

    pub fn not_authenticated() -> Self {
        Self::new(
            ErrorCategory::Auth,
            "NOT_AUTHENTICATED",
            "Not authenticated",
        )
        .with_hint("Run 'jellyfin-agent-cli login' first")
    }

    // Network errors
    pub fn network_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Network, "NETWORK_ERROR", message)
    }

    pub fn timeout() -> Self {
        Self::new(ErrorCategory::Network, "TIMEOUT", "Request timed out")
            .with_hint("Increase timeout in config.toml or check network connectivity")
    }

    pub fn connection_failed(url: String) -> Self {
        Self::new(
            ErrorCategory::Network,
            "CONNECTION_FAILED",
            format!("Cannot connect to {}", url),
        )
        .with_details(serde_json::json!({ "url": url }))
        .with_hint("Verify the server URL is correct and the server is running")
    }

    // API errors
    pub fn not_found(resource: String) -> Self {
        Self::new(
            ErrorCategory::Api,
            "API_NOT_FOUND",
            format!("{} not found", resource),
        )
    }

    pub fn rate_limit() -> Self {
        Self::new(
            ErrorCategory::Api,
            "API_RATE_LIMIT",
            "API rate limit exceeded",
        )
        .with_hint("Wait before retrying")
    }

    pub fn api_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Api, "API_ERROR", message)
    }

    // Input errors
    pub fn invalid_input(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(
            ErrorCategory::Input,
            "INPUT_INVALID",
            format!("Invalid {}: {}", field.into(), reason.into()),
        )
    }

    pub fn required_field(field: impl Into<String>) -> Self {
        Self::new(
            ErrorCategory::Input,
            "INPUT_REQUIRED",
            format!("{} is required", field.into()),
        )
    }

    // Internal errors
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Internal, "INTERNAL_ERROR", message.into())
            .with_hint("This is a bug. Please report this issue.")
    }

    /// Exit code for this error
    pub fn exit_code(&self) -> i32 {
        match self.category {
            ErrorCategory::Auth => 10,
            ErrorCategory::Network => 20,
            ErrorCategory::Api => 30,
            ErrorCategory::Input => 40,
            ErrorCategory::Internal => 50,
        }
    }
}

impl fmt::Display for JellyfinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}::{}] {}", self.category, self.code, self.message)
    }
}

impl std::error::Error for JellyfinError {}

// Convert from common error types

impl From<reqwest::Error> for JellyfinError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_builder() {
            // Get more details from the error
            let msg = if let Some(url) = err.url() {
                format!("Request builder error at URL: {}", url)
            } else {
                format!("Request builder error: {}", err)
            };
            return Self::internal(msg);
        }
        if err.is_timeout() {
            return Self::timeout();
        }
        if err.is_connect() {
            return Self::connection_failed(err.url().map(|u| u.to_string()).unwrap_or_default());
        }
        if err.is_request() {
            return Self::network_error(err.to_string());
        }
        Self::network_error(err.to_string())
    }
}

impl From<serde_json::Error> for JellyfinError {
    fn from(err: serde_json::Error) -> Self {
        Self::internal(format!("JSON parsing error: {}", err))
    }
}

impl From<serde_yaml::Error> for JellyfinError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::internal(format!("YAML parsing error: {}", err))
    }
}

impl From<toml::de::Error> for JellyfinError {
    fn from(err: toml::de::Error) -> Self {
        Self::internal(format!("TOML parsing error: {}", err))
    }
}

impl From<std::io::Error> for JellyfinError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Self::internal(format!("File not found: {}", err)),
            std::io::ErrorKind::PermissionDenied => {
                Self::internal(format!("Permission denied: {}", err))
            }
            _ => Self::internal(format!("IO error: {}", err)),
        }
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, JellyfinError>;
