//! Output formatters for command envelopes and help documents.

use crate::output::envelope::{OutputEnvelope, prune_json_nulls};
use crate::output::help::{HelpDocument, format_help_table};
use serde::Serialize;
use std::fmt::Display;

/// Output format options.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// YAML format (default structured format).
    #[default]
    Yaml,
    /// JSON format.
    Json,
    /// TOML format.
    Toml,
    /// Human-readable table format.
    Table,
    /// Line-oriented JSON format.
    Ndjson,
}

impl OutputFormat {
    /// Parse from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "yaml" | "yml" => Some(Self::Yaml),
            "json" => Some(Self::Json),
            "toml" => Some(Self::Toml),
            "table" | "txt" => Some(Self::Table),
            "ndjson" => Some(Self::Ndjson),
            _ => None,
        }
    }

    /// Stable list of supported format names.
    pub fn supported_values() -> Vec<&'static str> {
        vec!["table", "yaml", "toml", "json", "ndjson"]
    }

    /// Default structured format for the CLI contract.
    pub fn default_structured() -> Self {
        Self::Yaml
    }
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Yaml => write!(f, "yaml"),
            Self::Json => write!(f, "json"),
            Self::Toml => write!(f, "toml"),
            Self::Table => write!(f, "table"),
            Self::Ndjson => write!(f, "ndjson"),
        }
    }
}

/// Format an output envelope to a string.
pub fn format_output<T>(envelope: &OutputEnvelope<T>, format: OutputFormat) -> String
where
    T: Serialize,
{
    match format {
        OutputFormat::Yaml => format_yaml(envelope),
        OutputFormat::Json => format_json(envelope, true),
        OutputFormat::Toml => format_toml(envelope),
        OutputFormat::Table => format_table(envelope),
        OutputFormat::Ndjson => format_ndjson(envelope),
    }
}

/// Format a help document to a string.
pub fn format_help_document(help: &HelpDocument, format: OutputFormat) -> String {
    match format {
        OutputFormat::Yaml => format_yaml(help),
        OutputFormat::Json => format_json(help, true),
        OutputFormat::Toml => format_toml(help),
        OutputFormat::Table => format_help_table(help),
        OutputFormat::Ndjson => format_ndjson(help),
    }
}

fn format_yaml<T>(value: &T) -> String
where
    T: Serialize,
{
    serde_yaml::to_string(value).unwrap_or_else(|e| format!("Error serializing to YAML: {e}"))
}

fn format_json<T>(value: &T, pretty: bool) -> String
where
    T: Serialize,
{
    let result = if pretty {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    };
    result.unwrap_or_else(|e| format!("Error serializing to JSON: {e}"))
}

fn format_toml<T>(value: &T) -> String
where
    T: Serialize,
{
    match serde_json::to_value(value) {
        Ok(mut json_value) => {
            prune_json_nulls(&mut json_value);
            toml::to_string_pretty(&json_value)
                .unwrap_or_else(|e| format!("Error serializing to TOML: {e}"))
        }
        Err(e) => format!("Error serializing to TOML: {e}"),
    }
}

fn format_ndjson<T>(value: &T) -> String
where
    T: Serialize,
{
    match serde_json::to_string(value) {
        Ok(json) => format!("{json}\n"),
        Err(e) => {
            let fallback = serde_json::json!({"status":"error","summary":format!("Error serializing to NDJSON: {e}")});
            format!(
                "{}\n",
                serde_json::to_string(&fallback)
                    .unwrap_or_else(|_| format!("Error serializing to NDJSON: {e}"))
            )
        }
    }
}

fn format_table<T>(envelope: &OutputEnvelope<T>) -> String
where
    T: Serialize,
{
    let mut output = String::new();

    output.push_str(&format!("Command: {}\n", envelope.command));
    output.push_str(&format!("Status: {}\n", envelope.status));

    if !envelope.summary.is_empty() {
        output.push_str(&format!("Summary: {}\n", envelope.summary));
    }

    if let Some(data) = &envelope.data {
        let rendered = format_data_block(data);
        if !rendered.is_empty() {
            output.push_str("\nData:\n");
            output.push_str(&rendered);
        }
    }

    if !envelope.errors.is_empty() {
        output.push_str("\nErrors:\n");
        for error in &envelope.errors {
            output.push_str(&format!("  - [{}] {}\n", error.code, error.message));
        }
    }

    if !envelope.next_steps.is_empty() {
        output.push_str("\nNext Steps:\n");
        for step in &envelope.next_steps {
            output.push_str(&format!("  - {}: {}\n", step.action, step.description));
            output.push_str(&format!("    {}\n", step.command));
        }
    }

    output
}

fn format_data_block<T>(data: &T) -> String
where
    T: Serialize,
{
    match serde_yaml::to_string(data) {
        Ok(rendered) => rendered
            .trim_start_matches("---\n")
            .lines()
            .map(|line| format!("  {line}\n"))
            .collect(),
        Err(_) => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::envelope::{CommandOutput, ErrorDetail, NextStep, Status};
    use crate::output::help::HelpDocument;

    #[test]
    fn test_format_from_str() {
        assert_eq!(OutputFormat::from_str("yaml"), Some(OutputFormat::Yaml));
        assert_eq!(OutputFormat::from_str("JSON"), Some(OutputFormat::Json));
        assert_eq!(OutputFormat::from_str("toml"), Some(OutputFormat::Toml));
        assert_eq!(OutputFormat::from_str("table"), Some(OutputFormat::Table));
        assert_eq!(OutputFormat::from_str("ndjson"), Some(OutputFormat::Ndjson));
        assert_eq!(OutputFormat::from_str("invalid"), None);
    }

    #[test]
    fn test_format_yaml() {
        let env: OutputEnvelope<()> = OutputEnvelope::success("test", "ok");
        let yaml = format_output(&env, OutputFormat::Yaml);
        assert!(yaml.contains("command: test"));
        assert!(yaml.contains("status: success"));
    }

    #[test]
    fn test_format_ndjson() {
        let env: OutputEnvelope<()> = OutputEnvelope::success("test", "ok");
        let ndjson = format_output(&env, OutputFormat::Ndjson);
        assert!(ndjson.contains("\"command\":\"test\""));
        assert!(ndjson.ends_with('\n'));
    }

    #[test]
    fn test_format_machine_formats_prune_nulls_consistently() {
        let env: CommandOutput = OutputEnvelope::success("test", "ok")
            .with_data(serde_json::json!({"server": {"pid": null, "running": false}}));

        let yaml = format_output(&env, OutputFormat::Yaml);
        let json = format_output(&env, OutputFormat::Json);
        let toml = format_output(&env, OutputFormat::Toml);
        let ndjson = format_output(&env, OutputFormat::Ndjson);

        assert!(yaml.contains("status: success"));
        assert!(yaml.contains("running: false"));
        assert!(!yaml.contains("pid:"));

        assert!(json.contains("\"status\": \"success\""));
        assert!(json.contains("\"running\": false"));
        assert!(!json.contains("\"pid\""));

        assert!(toml.contains("status = \"success\""));
        assert!(toml.contains("running = false"));
        assert!(!toml.contains("pid ="));

        assert!(ndjson.contains("\"status\":\"success\""));
        assert!(ndjson.contains("\"running\":false"));
        assert!(!ndjson.contains("\"pid\""));
    }

    #[test]
    fn test_format_table() {
        let env: OutputEnvelope<serde_json::Value> =
            OutputEnvelope::success("test cmd", "all good")
                .with_data(serde_json::json!({"count": 1}));
        let table = format_output(&env, OutputFormat::Table);
        assert!(table.contains("Command: test cmd"));
        assert!(table.contains("Status: success"));
        assert!(table.contains("Summary: all good"));
        assert!(table.contains("count: 1"));
    }

    #[test]
    fn test_table_with_errors() {
        let env: OutputEnvelope<()> =
            OutputEnvelope::error("test", "failed").with_error(ErrorDetail::new("E001", "oops"));
        let table = format_output(&env, OutputFormat::Table);
        assert!(table.contains("Errors:"));
        assert!(table.contains("[E001] oops"));
    }

    #[test]
    fn test_table_with_next_steps() {
        let env: OutputEnvelope<()> = OutputEnvelope::success("test", "ok")
            .with_next_step(NextStep::new("retry", "test again", "give it another go"));
        let table = format_output(&env, OutputFormat::Table);
        assert!(table.contains("Next Steps:"));
        assert!(table.contains("retry"));
    }

    #[test]
    fn test_format_help_document() {
        let help = HelpDocument::new("jellyfin")
            .with_summary("CLI help")
            .with_usage("jellyfin [OPTIONS] <COMMAND>");
        let yaml = format_help_document(&help, OutputFormat::Yaml);
        let table = format_help_document(&help, OutputFormat::Table);

        assert!(yaml.contains("command: jellyfin"));
        assert!(table.contains("NAME"));
        assert!(table.contains("jellyfin - CLI help"));
    }

    #[test]
    fn test_status_display() {
        assert_eq!(Status::Success.to_string(), "success");
        assert_eq!(Status::Error.to_string(), "error");
    }
}
