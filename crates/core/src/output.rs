//! Output formatting for structured responses.
//!
//! Supports YAML (default), JSON, and TOML output formats.

use crate::{JellyfinError, Result};
use serde::{Deserialize, Serialize};

/// Supported output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Human-readable YAML (default)
    #[default]
    Yaml,
    /// Machine-readable JSON
    Json,
    /// Configuration format TOML
    Toml,
}

impl OutputFormat {
    /// Parse from string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "yaml" | "yml" => Ok(Self::Yaml),
            "json" => Ok(Self::Json),
            "toml" => Ok(Self::Toml),
            _ => Err(JellyfinError::invalid_input(
                "output format",
                "must be yaml, json, or toml",
            )),
        }
    }

    /// Content type for HTTP responses
    pub fn content_type(&self) -> &'static str {
        match self {
            Self::Yaml => "application/x-yaml",
            Self::Json => "application/json",
            Self::Toml => "application/toml",
        }
    }

    /// Format a response
    pub fn format<T: Serialize>(&self, response: &Response<T>) -> Result<String> {
        match self {
            Self::Yaml => serde_yaml::to_string(response).map_err(JellyfinError::from),
            Self::Json => serde_json::to_string_pretty(response).map_err(JellyfinError::from),
            Self::Toml => toml::to_string_pretty(response)
                .map_err(|e| JellyfinError::internal(format!("TOML serialization error: {}", e))),
        }
    }
}

/// Unified response structure
///
/// All CLI commands output this structure for consistent Agent consumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response<T> {
    /// Response result
    pub result: ResponseResult,
    /// Response data on success
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// Error details on failure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorResponse>,
}

/// Result indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseResult {
    Success,
    Error,
}

/// Error response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error category
    pub category: String,
    /// Error code
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// Resolution hint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl<T> Response<T> {
    /// Create a success response with data
    pub fn success(data: T) -> Self {
        Self {
            result: ResponseResult::Success,
            data: Some(data),
            error: None,
        }
    }

    /// Create an error response from a JellyfinError
    pub fn error(err: &JellyfinError) -> Self {
        Self {
            result: ResponseResult::Error,
            data: None,
            error: Some(ErrorResponse {
                category: err.category.to_string(),
                code: err.code.clone(),
                message: err.message.clone(),
                details: err.details.clone(),
                hint: err.hint.clone(),
            }),
        }
    }

    /// Create an error response with custom fields
    pub fn error_with(
        category: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            result: ResponseResult::Error,
            data: None,
            error: Some(ErrorResponse {
                category: category.into(),
                code: code.into(),
                message: message.into(),
                details: None,
                hint: None,
            }),
        }
    }

    /// Get exit code for this response
    pub fn exit_code(&self) -> i32 {
        match self.result {
            ResponseResult::Success => 0,
            ResponseResult::Error => 1,
        }
    }

    /// Check if response is successful
    pub fn is_success(&self) -> bool {
        self.result == ResponseResult::Success
    }
}

impl<T> From<Result<T>> for Response<T> {
    fn from(result: Result<T>) -> Self {
        match result {
            Ok(data) => Self::success(data),
            Err(err) => Self::error(&err),
        }
    }
}

/// Helper for serializing simple values
///
/// Use when you just need to return a simple value like a string or number.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataValue {
    String(String),
    Number(i64),
    Float(f64),
    Boolean(bool),
    Object(serde_json::Value),
    Array(Vec<serde_json::Value>),
    Null,
}

impl From<String> for DataValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for DataValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<i64> for DataValue {
    fn from(n: i64) -> Self {
        Self::Number(n)
    }
}

impl From<bool> for DataValue {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl From<serde_json::Value> for DataValue {
    fn from(v: serde_json::Value) -> Self {
        match v {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(b) => Self::Boolean(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Self::Number(i)
                } else if let Some(f) = n.as_f64() {
                    Self::Float(f)
                } else {
                    // Store as generic object when number is out of range
                    Self::Object(serde_json::json!({"value": n.to_string()}))
                }
            }
            serde_json::Value::String(s) => Self::String(s),
            serde_json::Value::Array(arr) => Self::Array(arr),
            serde_json::Value::Object(obj) => Self::Object(serde_json::Value::Object(obj)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_str() {
        assert_eq!(OutputFormat::from_str("yaml").unwrap(), OutputFormat::Yaml);
        assert_eq!(OutputFormat::from_str("json").unwrap(), OutputFormat::Json);
        assert_eq!(OutputFormat::from_str("toml").unwrap(), OutputFormat::Toml);
        assert!(OutputFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_success_response() {
        let resp = Response::success("test data");
        assert!(resp.is_success());
        assert_eq!(resp.exit_code(), 0);
    }

    #[test]
    fn test_error_response() {
        let err = JellyfinError::auth_failed("bad password");
        let resp = Response::<()>::error(&err);
        assert!(!resp.is_success());
        assert_eq!(resp.exit_code(), 1);
        assert_eq!(resp.error.unwrap().code, "AUTH_FAILED");
    }
}
