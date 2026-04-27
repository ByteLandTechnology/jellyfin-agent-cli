//! E2E command integration tests
//!
//! These tests verify the E2E commands work correctly end-to-end.

use std::process::Command;

fn jellyfin_cmd() -> Command {
    // Use the binary built by cargo for testing
    let bin_path = option_env!("CARGO_BIN_EXE_jellyfin-cli-internal")
        .unwrap_or("./target/debug/jellyfin-cli-internal");
    Command::new(bin_path)
}

#[test]
fn test_e2e_setup_command() {
    let output = jellyfin_cmd()
        .args(&["e2e", "setup"])
        .output()
        .expect("Failed to execute jellyfin e2e setup");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify output contains expected elements
    assert!(stdout.contains("command: jellyfin-agent-cli e2e setup"));
    assert!(stdout.contains("status: "));
    assert!(
        stdout.contains("config_dir:") || stdout.contains("errors:"),
        "setup should return either success data or a structured error envelope"
    );
}

#[test]
fn test_e2e_status_command() {
    let output = jellyfin_cmd()
        .args(&["e2e", "status"])
        .output()
        .expect("Failed to execute jellyfin e2e status");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("command: jellyfin-agent-cli e2e status"));
    assert!(stdout.contains("environment:"));
    assert!(stdout.contains("server:"));
}

#[test]
fn test_e2e_status_detailed() {
    let output = jellyfin_cmd()
        .args(&["e2e", "status", "--detailed"])
        .output()
        .expect("Failed to execute jellyfin e2e status detailed");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("command: jellyfin-agent-cli e2e status"));
    // With detailed flag, should have more info
    assert!(stdout.contains("environment:"));
}

#[test]
fn test_e2e_config_show() {
    let output = jellyfin_cmd()
        .args(&["e2e", "config", "show"])
        .output()
        .expect("Failed to execute jellyfin e2e config show");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("command: jellyfin-agent-cli e2e config show"));
    assert!(stdout.contains("server_url:"));
    assert!(stdout.contains("port:"));
}

#[test]
fn test_e2e_logs_list() {
    let output = jellyfin_cmd()
        .args(&["e2e", "logs", "list"])
        .output()
        .expect("Failed to execute jellyfin e2e logs list");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("command: jellyfin-agent-cli e2e logs list"));
    assert!(stdout.contains("log_dir:"));
}

#[test]
fn test_e2e_help() {
    let output = jellyfin_cmd()
        .args(&["e2e", "--help"])
        .output()
        .expect("Failed to execute jellyfin e2e help");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify all subcommands are listed
    assert!(stdout.contains("setup"));
    assert!(stdout.contains("start"));
    assert!(stdout.contains("stop"));
    assert!(stdout.contains("media"));
    assert!(stdout.contains("status"));
    assert!(stdout.contains("logs"));
    assert!(stdout.contains("reset"));
    assert!(stdout.contains("config"));
}

#[test]
fn test_e2e_output_formats_are_structured() {
    for format in &["yaml", "json", "toml", "ndjson"] {
        let output = jellyfin_cmd()
            .args(&["e2e", "status", "--output", format])
            .output()
            .expect(&format!(
                "Failed to execute jellyfin e2e status --output {}",
                format
            ));

        assert!(output.status.success());

        let stdout = String::from_utf8_lossy(&output.stdout);
        match *format {
            "yaml" => {
                assert!(stdout.contains("status: success"));
                assert!(stdout.contains("command: jellyfin-agent-cli e2e status"));
            }
            "json" => {
                assert!(stdout.contains("\"status\": \"success\""));
                assert!(stdout.contains("\"command\": \"jellyfin-agent-cli e2e status\""));
            }
            "toml" => {
                assert!(stdout.contains("status = \"success\""));
                assert!(stdout.contains("command = \"jellyfin-agent-cli e2e status\""));
            }
            "ndjson" => {
                assert_eq!(stdout.lines().count(), 1);
                assert!(stdout.contains("\"status\":\"success\""));
                assert!(stdout.contains("\"command\":\"jellyfin-agent-cli e2e status\""));
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn test_e2e_subcommand_help() {
    // Test setup help
    let output = jellyfin_cmd()
        .args(&["e2e", "setup", "--help"])
        .output()
        .expect("Failed to execute jellyfin e2e setup --help");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Initialize E2E test environment"));

    // Test media help
    let output = jellyfin_cmd()
        .args(&["e2e", "media", "--help"])
        .output()
        .expect("Failed to execute jellyfin e2e media --help");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Manage test media files"));

    // Test media download help
    let output = jellyfin_cmd()
        .args(&["e2e", "media", "download", "--help"])
        .output()
        .expect("Failed to execute jellyfin e2e media download --help");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Download open-licensed test media files"));
}

#[test]
fn test_e2e_media_download() {
    let output = jellyfin_cmd()
        .args(&["e2e", "media", "download", "big-buck-bunny"])
        .output()
        .expect("Failed to execute jellyfin e2e media download");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("command: jellyfin-agent-cli e2e media download"));
    assert!(stdout.contains("downloaded:") || stdout.contains("errors:"));
}

#[test]
fn test_e2e_reset_media() {
    // First ensure media exists
    let _ = jellyfin_cmd()
        .args(&["e2e", "media", "download", "classical-music"])
        .output();

    // Then reset media
    let output = jellyfin_cmd()
        .args(&["e2e", "reset", "media"])
        .output()
        .expect("Failed to execute jellyfin e2e reset media");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("command: jellyfin-agent-cli e2e reset media"));
    assert!(stdout.contains("status: success"));
}
