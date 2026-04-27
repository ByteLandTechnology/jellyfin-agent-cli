use std::process::Command;

fn shipped_bin() -> Command {
    let bin_path = option_env!("CARGO_BIN_EXE_jellyfin-agent-cli")
        .unwrap_or("./target/debug/jellyfin-agent-cli");
    Command::new(bin_path)
}

#[test]
fn root_binary_renders_human_help() {
    let output = shipped_bin()
        .arg("--help")
        .output()
        .expect("failed to run jellyfin-agent-cli --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("NAME"));
    assert!(stdout.contains("SYNOPSIS"));
    assert!(stdout.contains("EXIT CODES"));
}

#[test]
fn root_binary_exposes_structured_help_command() {
    let output = shipped_bin()
        .args(["help", "--format", "json"])
        .output()
        .expect("failed to run jellyfin-agent-cli help --format json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"command\""));
    assert!(stdout.contains("\"runtime_directories\""));
}
