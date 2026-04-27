//! Output envelope types.
//!
//! Defines the stable command result and error contract for the CLI.

use jellyfin_core::JellyfinError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Common JSON payload type for CLI command data.
///
/// `CommandData` normalizes recursive `null` values on construction so YAML, JSON,
/// TOML, and NDJSON stay semantically aligned.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CommandData(serde_json::Value);

impl From<serde_json::Value> for CommandData {
    fn from(mut value: serde_json::Value) -> Self {
        prune_json_nulls(&mut value);
        Self(value)
    }
}

/// Standard command output type for the CLI.
pub type CommandOutput = OutputEnvelope<CommandData>;

/// Execution status of a command.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Command executed successfully.
    Success,
    /// Command encountered an error.
    Error,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Success => write!(f, "success"),
            Status::Error => write!(f, "error"),
        }
    }
}

/// Detailed error information.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    /// Machine-readable error code.
    pub code: String,
    /// Human-readable error message.
    pub message: String,
}

impl ErrorDetail {
    /// Create a new error detail.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

/// A suggested next step for the user.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NextStep {
    /// Brief description of the action.
    pub action: String,
    /// Command to execute.
    pub command: String,
    /// Detailed description of what this step does.
    pub description: String,
}

impl NextStep {
    /// Create a new next step.
    pub fn new(
        action: impl Into<String>,
        command: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            action: action.into(),
            command: command.into(),
            description: description.into(),
        }
    }
}

/// Effective Active Context information attached to command results.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveContextState {
    /// Persisted active profile from configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persisted_profile: Option<String>,
    /// Profile explicitly requested with `--profile`, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_profile: Option<String>,
    /// Whether the requested profile exists in configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_profile_found: Option<bool>,
    /// The profile that effectively drove this command result, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_profile: Option<String>,
    /// Whether the explicit override was applied.
    pub override_applied: bool,
    /// Precedence rule for profile selection.
    pub precedence: String,
    /// Additional context for commands that surface persisted state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl ActiveContextState {
    /// Construct a new Active Context state payload.
    pub fn new(precedence: impl Into<String>) -> Self {
        Self {
            persisted_profile: None,
            requested_profile: None,
            requested_profile_found: None,
            effective_profile: None,
            override_applied: false,
            precedence: precedence.into(),
            note: None,
        }
    }
}

/// Standard output envelope for all CLI commands.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputEnvelope<T> {
    /// The command that was executed.
    pub command: String,

    /// Execution status.
    pub status: Status,

    /// Brief summary of the result.
    pub summary: String,

    /// Optional data payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    /// Suggested next steps.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub next_steps: Vec<NextStep>,

    /// Error details.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ErrorDetail>,

    /// Effective Active Context information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_context: Option<ActiveContextState>,
}

impl<T> OutputEnvelope<T> {
    /// Create a new success envelope.
    pub fn success(command: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            status: Status::Success,
            summary: summary.into(),
            data: None,
            next_steps: Vec::new(),
            errors: Vec::new(),
            active_context: None,
        }
    }

    /// Create a new error envelope.
    pub fn error(command: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            status: Status::Error,
            summary: summary.into(),
            data: None,
            next_steps: Vec::new(),
            errors: Vec::new(),
            active_context: None,
        }
    }

    /// Set the data payload.
    pub fn with_data<U>(mut self, data: U) -> Self
    where
        U: Into<T>,
    {
        self.data = Some(data.into());
        self
    }

    /// Add a next step.
    pub fn with_next_step(mut self, step: NextStep) -> Self {
        self.next_steps.push(step);
        self
    }

    /// Add an error detail.
    pub fn with_error(mut self, error: ErrorDetail) -> Self {
        self.errors.push(error);
        self
    }

    /// Replace all error details.
    pub fn with_errors(mut self, errors: Vec<ErrorDetail>) -> Self {
        self.errors = errors;
        self
    }

    /// Attach Active Context metadata.
    pub fn with_active_context(mut self, state: ActiveContextState) -> Self {
        self.active_context = Some(state);
        self
    }

    /// Normalize legacy public command strings to the canonical shipped binary name.
    pub fn normalize_public_commands(mut self, public_name: &str) -> Self {
        self.command = normalize_public_command(&self.command, public_name);
        for step in &mut self.next_steps {
            step.command = normalize_public_command(&step.command, public_name);
        }
        self
    }
}

impl OutputEnvelope<CommandData> {
    /// Build a stable error envelope from a runtime command error.
    pub fn from_command_error(command: impl Into<String>, error: &JellyfinError) -> Self {
        let command = command.into();
        let next_step_description = error
            .hint
            .clone()
            .unwrap_or_else(|| "Inspect command usage and retry.".to_string());

        Self::error(command.clone(), "Command failed.")
            .with_error(ErrorDetail::new(
                normalize_error_code(&error.code),
                error.message.clone(),
            ))
            .with_next_step(NextStep::new(
                "inspect_help",
                format!("{command} --help"),
                next_step_description,
            ))
    }
}

fn normalize_error_code(code: &str) -> String {
    code.to_ascii_lowercase()
}

fn normalize_public_command(command: &str, public_name: &str) -> String {
    if command == "jellyfin" {
        return public_name.to_string();
    }

    if let Some(rest) = command.strip_prefix("jellyfin ") {
        return format!("{public_name} {rest}");
    }

    command.to_string()
}

pub(crate) fn prune_json_nulls(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for child in map.values_mut() {
                prune_json_nulls(child);
            }
            map.retain(|_, child| !child.is_null());
        }
        serde_json::Value::Array(values) => {
            for child in values.iter_mut() {
                prune_json_nulls(child);
            }
            values.retain(|child| !child.is_null());
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_envelope() {
        let env: OutputEnvelope<()> = OutputEnvelope::success("test cmd", "it worked");
        assert_eq!(env.status, Status::Success);
        assert_eq!(env.summary, "it worked");
        assert!(env.errors.is_empty());
    }

    #[test]
    fn test_error_envelope() {
        let env: OutputEnvelope<()> = OutputEnvelope::error("test cmd", "it failed")
            .with_error(ErrorDetail::new("io_error", "Something went wrong"));
        assert_eq!(env.status, Status::Error);
        assert_eq!(env.errors.len(), 1);
        assert_eq!(env.errors[0].code, "io_error");
    }

    #[test]
    fn test_from_command_error() {
        let err = JellyfinError::auth_failed("bad credentials");
        let env = OutputEnvelope::from_command_error("jellyfin login", &err);

        assert_eq!(env.status, Status::Error);
        assert_eq!(env.summary, "Command failed.");
        assert_eq!(env.errors[0].code, "auth_failed");
        assert!(env.next_steps[0].command.ends_with("--help"));
    }

    #[test]
    fn test_command_data_prunes_recursive_nulls() {
        let data = CommandData::from(serde_json::json!({
            "server": {
                "pid": null,
                "running": false,
                "url": null
            },
            "values": [1, null, 2]
        }));

        assert_eq!(
            data,
            CommandData::from(serde_json::json!({
                "server": {
                    "running": false
                },
                "values": [1, 2]
            }))
        );
    }
}
