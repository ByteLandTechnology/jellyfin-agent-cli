//! E2E logs command

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use clap::{Args, Subcommand};
use jellyfin_core::Result;
use std::path::Path;

/// Logs management commands
#[derive(Subcommand, Debug, Clone)]
pub enum LogsCommands {
    /// Show recent log entries
    Tail(LogsTailOptions),

    /// Show all available logs
    List,

    /// Clear log files
    Clear,
}

impl LogsCommands {
    pub fn execute(&self) -> Result<CommandOutput> {
        match self {
            LogsCommands::Tail(opts) => opts.execute(),
            LogsCommands::List => self.list_logs(),
            LogsCommands::Clear => self.clear_logs(),
        }
    }

    fn list_logs(&self) -> Result<CommandOutput> {
        let environment = jellyfin_core::E2EEnvironment::new();
        let log_dir = environment.log_dir();

        let logs = if log_dir.exists() {
            let mut log_files = vec![];
            if let Ok(entries) = std::fs::read_dir(&log_dir) {
                for entry in entries.flatten() {
                    if let Ok(name) = entry.file_name().into_string() {
                        if name.ends_with(".log") || name.ends_with(".txt") {
                            let metadata = entry.metadata()?;
                            let modified = metadata.modified().ok();
                            let ago_str = modified
                                .and_then(|t| t.elapsed().ok())
                                .map(|d| format!("{:.0}s ago", d.as_secs()))
                                .unwrap_or_else(|| "unknown".to_string());

                            log_files.push(serde_json::json!({
                                "name": name,
                                "size": metadata.len(),
                                "modified": ago_str
                            }));
                        }
                    }
                }
            }
            log_files
        } else {
            vec![]
        };

        let envelope: CommandOutput = OutputEnvelope::success(
            "jellyfin e2e logs list",
            format!("Found {} log file(s)", logs.len()),
        )
        .with_data(serde_json::json!({
            "log_dir": log_dir.display().to_string(),
            "logs": logs
        }))
        .with_next_step(NextStep::new(
            "tail_logs",
            "jellyfin e2e logs tail",
            "Show recent log entries",
        ))
        .with_next_step(NextStep::new(
            "clear_logs",
            "jellyfin e2e logs clear",
            "Clear log files",
        ));

        Ok(envelope)
    }

    fn clear_logs(&self) -> Result<CommandOutput> {
        let environment = jellyfin_core::E2EEnvironment::new();
        let log_dir = environment.log_dir();

        if !log_dir.exists() {
            let envelope: CommandOutput =
                OutputEnvelope::success("jellyfin e2e logs clear", "No logs to clear");
            return Ok(envelope);
        }

        let mut cleared_count = 0u64;
        if let Ok(entries) = std::fs::read_dir(&log_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("log") {
                    std::fs::remove_file(entry.path())?;
                    cleared_count += 1;
                }
            }
        }

        let envelope: CommandOutput = OutputEnvelope::success(
            "jellyfin e2e logs clear",
            format!("Cleared {} log file(s)", cleared_count),
        )
        .with_next_step(NextStep::new(
            "start_server",
            "jellyfin e2e start",
            "Start server to generate fresh logs",
        ));

        Ok(envelope)
    }
}

/// Options for tailing logs
#[derive(Args, Debug, Clone)]
pub struct LogsTailOptions {
    /// Number of lines to show [default: 50]
    #[arg(short = 'n', long, default_value = "50")]
    pub lines: usize,

    /// Follow log output
    #[arg(short = 'f', long)]
    pub follow: bool,

    /// Show specific log file
    #[arg(long)]
    pub file: Option<String>,
}

impl LogsTailOptions {
    pub fn execute(&self) -> Result<CommandOutput> {
        let environment = jellyfin_core::E2EEnvironment::new();
        let log_dir = environment.log_dir();

        // Find the log file to tail
        let log_file = if let Some(ref file) = self.file {
            let joined = log_dir.join(file);
            // Canonicalize to resolve .. components and prevent path traversal
            let canonical = joined.canonicalize().map_err(|_| {
                jellyfin_core::JellyfinError::invalid_input(
                    "file",
                    format!("Invalid log file path: {file}"),
                )
            })?;
            let log_dir_canonical = log_dir.canonicalize().map_err(|_| {
                jellyfin_core::JellyfinError::invalid_input("file", "Log directory not found")
            })?;
            if !canonical.starts_with(&log_dir_canonical) {
                return Err(jellyfin_core::JellyfinError::invalid_input(
                    "file",
                    "Path escapes the log directory",
                ));
            }
            canonical
        } else {
            // Find the most recent log file
            find_latest_log(&log_dir)?
        };

        if !log_file.exists() {
            let envelope: CommandOutput =
                OutputEnvelope::success("jellyfin e2e logs tail", "No log file found")
                    .with_next_step(NextStep::new(
                        "start_server",
                        "jellyfin e2e start",
                        "Start server to generate logs",
                    ));

            return Ok(envelope);
        }

        // Read and display the last N lines
        let content = std::fs::read_to_string(&log_file)?;
        let lines: Vec<&str> = content.lines().rev().take(self.lines).collect();
        let lines: Vec<&str> = lines.into_iter().rev().collect();

        let envelope: CommandOutput = OutputEnvelope::success(
            "jellyfin e2e logs tail",
            format!("Last {} lines from {}", self.lines, log_file.display()),
        )
        .with_data(serde_json::json!({
            "file": log_file.display().to_string(),
            "lines": lines.join("\n")
        }))
        .with_next_step(NextStep::new(
            "list_logs",
            "jellyfin e2e logs list",
            "List all log files",
        ));

        Ok(envelope)
    }
}

/// Find the most recently modified log file
fn find_latest_log(log_dir: &Path) -> Result<std::path::PathBuf> {
    let mut latest = None;
    let mut latest_mtime = std::time::SystemTime::UNIX_EPOCH;

    if let Ok(entries) = std::fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if !metadata.is_file() {
                    continue;
                }
                let is_log = entry.path().extension().and_then(|s| s.to_str()) == Some("log");
                if !is_log {
                    continue;
                }
                if let Ok(mtime) = metadata.modified() {
                    if mtime > latest_mtime {
                        latest_mtime = mtime;
                        latest = Some(entry.path());
                    }
                }
            }
        }
    }

    latest.ok_or_else(|| jellyfin_core::JellyfinError::not_found("log file".to_string()))
}
