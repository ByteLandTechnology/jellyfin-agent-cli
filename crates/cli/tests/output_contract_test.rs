//! CLI output contract validation tests.

use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use std::process::{Command, Output};

fn jellyfin_cmd() -> Command {
    let bin_path = option_env!("CARGO_BIN_EXE_jellyfin-cli-internal")
        .unwrap_or("./target/debug/jellyfin-cli-internal");
    Command::new(bin_path)
}

fn run(args: &[&str]) -> Output {
    jellyfin_cmd()
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to execute jellyfin {:?}: {}", args, error))
}

fn stdout_yaml(output: &Output) -> YamlValue {
    serde_yaml::from_slice(&output.stdout).expect("stdout should be valid YAML")
}

fn stdout_json(output: &Output) -> JsonValue {
    serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON")
}

fn stdout_ndjson_line(output: &Output) -> JsonValue {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines = stdout.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 1, "ndjson should emit exactly one line");
    serde_json::from_str(lines[0]).expect("ndjson line should be JSON")
}

fn top_level_has(doc: &YamlValue, key: &str) -> bool {
    doc.as_mapping()
        .is_some_and(|mapping| mapping.contains_key(YamlValue::String(key.to_string())))
}

fn assert_field_order(text: &str, fields: &[&str]) {
    let mut previous_index = None;

    for field in fields {
        let index = text
            .find(field)
            .unwrap_or_else(|| panic!("missing expected field marker: {field}"));
        if let Some(previous_index) = previous_index {
            assert!(
                previous_index < index,
                "field marker {field} appeared out of order in:\n{text}"
            );
        }
        previous_index = Some(index);
    }
}

#[test]
fn test_default_help_is_human_readable() {
    let output = run(&["--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("NAME"));
    assert!(stdout.contains("jellyfin-agent-cli -"));
    assert!(stdout.contains("SYNOPSIS"));
    assert!(stdout.contains("OPTIONS"));
}

#[test]
fn test_help_table_is_available_on_explicit_request() {
    let output = run(&["--help", "--output", "table"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("NAME"));
    assert!(stdout.contains("jellyfin-agent-cli -"));
    assert!(stdout.contains("SYNOPSIS"));
    assert!(stdout.contains("OPTIONS"));
}

#[test]
fn test_usage_failure_renders_help_document_on_stderr() {
    let output = run(&["config", "set-default"]);
    assert_eq!(output.status.code(), Some(40));

    let doc = stdout_yaml(&output);
    assert_eq!(
        doc["command"].as_str(),
        Some("jellyfin-agent-cli config set-default")
    );
    assert_eq!(doc["status"].as_str(), Some("error"));
    assert!(doc["errors"].is_sequence());
}

#[test]
fn test_config_show_is_direct_yaml_envelope() {
    let output = run(&["config", "show"]);
    assert!(output.status.success());

    let doc = stdout_yaml(&output);
    assert_eq!(
        doc["command"].as_str(),
        Some("jellyfin-agent-cli config show")
    );
    assert_eq!(doc["status"].as_str(), Some("success"));
    assert!(doc["summary"].as_str().is_some());
    assert!(doc["data"].is_mapping());
    assert!(!top_level_has(&doc, "result"));
    assert!(!top_level_has(&doc, "kind"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_field_order(&stdout, &["command:", "status:", "summary:", "data:"]);
}

#[test]
fn test_runtime_error_is_direct_yaml_envelope() {
    let output = run(&["config", "remove-server", "missing"]);
    assert_eq!(output.status.code(), Some(40));

    let doc = stdout_yaml(&output);
    assert_eq!(
        doc["command"].as_str(),
        Some("jellyfin-agent-cli config remove-server")
    );
    assert_eq!(doc["status"].as_str(), Some("error"));
    assert_eq!(doc["summary"].as_str(), Some("Command failed."));
    assert!(doc["errors"].is_sequence());
}

#[test]
fn test_json_format_emits_direct_envelope() {
    let output = run(&["config", "show", "--output", "json"]);
    assert!(output.status.success());

    let doc = stdout_json(&output);
    assert_eq!(
        doc["command"].as_str(),
        Some("jellyfin-agent-cli config show")
    );
    assert_eq!(doc["status"].as_str(), Some("success"));
    assert!(doc.get("summary").is_some());
    assert!(doc.get("data").is_some());
    assert!(doc.get("result").is_none());
    assert!(doc.get("kind").is_none());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_field_order(
        &stdout,
        &["\"command\"", "\"status\"", "\"summary\"", "\"data\""],
    );
}

#[test]
fn test_ndjson_format_emits_single_json_line() {
    let output = run(&["config", "show", "--output", "ndjson"]);
    assert!(output.status.success());

    let doc = stdout_ndjson_line(&output);
    assert_eq!(
        doc["command"].as_str(),
        Some("jellyfin-agent-cli config show")
    );
    assert_eq!(doc["status"].as_str(), Some("success"));
}

#[test]
fn test_help_json_format_is_structured_document() {
    let output = run(&["help", "e2e", "config", "show", "--format", "json"]);
    assert!(output.status.success());

    let doc = stdout_json(&output);
    assert_eq!(
        doc["command"].as_str(),
        Some("jellyfin-agent-cli e2e config show")
    );
    assert_eq!(
        doc["summary"].as_str(),
        Some("Show current E2E configuration")
    );
    assert!(doc.get("usage").is_some());
    assert!(doc.get("status").is_none());
    assert!(doc.get("message").is_none());

    let output_formats = doc["output_formats"]
        .as_array()
        .expect("help should list output formats");
    assert!(
        output_formats
            .iter()
            .any(|format| format["surface"].as_str() == Some("--output"))
    );
}

#[test]
fn test_machine_formats_omit_recursive_nulls_consistently() {
    let yaml_output = run(&["e2e", "status", "--output", "yaml"]);
    let json_output = run(&["e2e", "status", "--output", "json"]);
    let toml_output = run(&["e2e", "status", "--output", "toml"]);
    let ndjson_output = run(&["e2e", "status", "--output", "ndjson"]);

    assert!(yaml_output.status.success());
    assert!(json_output.status.success());
    assert!(toml_output.status.success());
    assert!(ndjson_output.status.success());

    let yaml = stdout_yaml(&yaml_output);
    let json = stdout_json(&json_output);
    let ndjson = stdout_ndjson_line(&ndjson_output);
    let toml = String::from_utf8_lossy(&toml_output.stdout);

    let yaml_server = yaml["data"]["server"]
        .as_mapping()
        .expect("yaml server payload should be a mapping");
    assert!(!yaml_server.contains_key(YamlValue::String("pid".to_string())));
    assert!(!yaml_server.contains_key(YamlValue::String("url".to_string())));

    let json_server = json["data"]["server"]
        .as_object()
        .expect("json server payload should be an object");
    assert!(!json_server.contains_key("pid"));
    assert!(!json_server.contains_key("url"));

    let ndjson_server = ndjson["data"]["server"]
        .as_object()
        .expect("ndjson server payload should be an object");
    assert!(!ndjson_server.contains_key("pid"));
    assert!(!ndjson_server.contains_key("url"));

    assert!(!toml.contains("pid ="));
    assert!(!toml.contains("url ="));
}
