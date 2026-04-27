//! E2E reset command

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use clap::{Parser, Subcommand};
use jellyfin_core::{E2EEnvironment, JellyfinError, MediaCache, Result};
use std::fs;

/// Reset E2E environment commands
#[derive(Subcommand, Debug, Clone)]
pub enum ResetCommands {
    /// Reset all E2E data (requires confirmation)
    All(ResetAllOptions),

    /// Reset only media cache
    Media,

    /// Reset only server data
    Server,
}

impl ResetCommands {
    pub fn execute(&self) -> Result<CommandOutput> {
        match self {
            ResetCommands::All(opts) => opts.execute(),
            ResetCommands::Media => self.reset_media(),
            ResetCommands::Server => self.reset_server(),
        }
    }

    fn reset_media(&self) -> Result<CommandOutput> {
        let media_dir = MediaCache::default_cache_dir();

        if !media_dir.exists() {
            let envelope: CommandOutput =
                OutputEnvelope::success("jellyfin e2e reset media", "No media cache to reset");
            return Ok(envelope);
        }

        let size_before = dir_size(&media_dir);
        fs::remove_dir_all(&media_dir)?;
        fs::create_dir_all(&media_dir)?;

        let envelope: CommandOutput = OutputEnvelope::success(
            "jellyfin e2e reset media",
            format!("Media cache reset (freed {})", format_bytes(size_before)),
        )
        .with_data(serde_json::json!({
            "freed": size_before
        }))
        .with_next_step(NextStep::new(
            "download_media",
            "jellyfin e2e media download all",
            "Download test media",
        ));

        Ok(envelope)
    }

    fn reset_server(&self) -> Result<CommandOutput> {
        let env = E2EEnvironment::new();

        // First check if server is running
        if env.server_pid.is_some() {
            return Err(JellyfinError::invalid_input(
                "server state",
                "Server is running. Stop it first with 'jellyfin e2e stop'.",
            ));
        }

        let data_dir = &env.data_dir;

        if !data_dir.exists() {
            let envelope: CommandOutput =
                OutputEnvelope::success("jellyfin e2e reset server", "No server data to reset");
            return Ok(envelope);
        }

        let size_before = dir_size(data_dir);
        fs::remove_dir_all(data_dir)?;
        fs::create_dir_all(data_dir)?;

        let envelope: CommandOutput = OutputEnvelope::success(
            "jellyfin e2e reset server",
            format!("Server data reset (freed {})", format_bytes(size_before)),
        )
        .with_data(serde_json::json!({
            "freed": size_before
        }))
        .with_next_step(NextStep::new(
            "reinitialize",
            "jellyfin e2e setup",
            "Reinitialize E2E environment",
        ));

        Ok(envelope)
    }
}

/// Options for resetting all E2E data
#[derive(Parser, Debug, Clone)]
pub struct ResetAllOptions {
    /// Skip confirmation prompt
    #[arg(long)]
    pub force: bool,
}

impl ResetAllOptions {
    pub fn execute(&self) -> Result<CommandOutput> {
        let env = E2EEnvironment::new();

        // Confirm unless force is set
        if !self.force {
            eprintln!("This will reset ALL E2E data including:");
            eprintln!("  - Server configuration and data");
            eprintln!("  - Downloaded media cache");
            eprintln!();
            eprint!("Are you sure? [y/N]: ");
            std::io::Write::flush(&mut std::io::stderr())?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if !input.trim().eq_ignore_ascii_case("y") {
                let envelope: CommandOutput =
                    OutputEnvelope::success("jellyfin e2e reset all", "Reset cancelled.")
                        .with_next_step(NextStep::new(
                            "force_reset",
                            "jellyfin e2e reset all --force",
                            "Reset all E2E data without an interactive prompt",
                        ));
                return Ok(envelope);
            }
        }

        // Stop server if running
        if env.server_pid.is_some() {
            // Attempt to stop
            let _ = std::process::Command::new("jellyfin")
                .args(["e2e", "stop"])
                .status();
        }

        // Remove all E2E data
        let media_dir = MediaCache::default_cache_dir();
        let size_before =
            dir_size(&env.config_dir) + dir_size(&env.data_dir) + dir_size(&media_dir);
        if env.config_dir.exists() {
            fs::remove_dir_all(&env.config_dir)?;
            fs::create_dir_all(&env.config_dir)?;
        }
        if env.data_dir.exists() {
            fs::remove_dir_all(&env.data_dir)?;
            fs::create_dir_all(&env.data_dir)?;
        }
        if media_dir.exists() {
            fs::remove_dir_all(&media_dir)?;
            fs::create_dir_all(&media_dir)?;
        }

        let envelope: CommandOutput = OutputEnvelope::success(
            "jellyfin e2e reset all",
            format!(
                "E2E environment reset (freed {})",
                format_bytes(size_before)
            ),
        )
        .with_data(serde_json::json!({
            "freed": size_before
        }))
        .with_next_step(NextStep::new(
            "setup",
            "jellyfin e2e setup",
            "Initialize fresh E2E environment",
        ));

        Ok(envelope)
    }
}

/// Get directory size in bytes
fn dir_size(path: &std::path::Path) -> u64 {
    let mut size = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    size += metadata.len();
                } else if metadata.is_dir() {
                    size += dir_size(&entry.path());
                }
            }
        }
    }
    size
}

/// Format bytes as human-readable
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
