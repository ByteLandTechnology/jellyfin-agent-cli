//! E2E status command

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use clap::Parser;
use jellyfin_core::{E2EEnvironment, MediaCache, Result};
use std::path::Path;
/// Options for the E2E status command
#[derive(Parser, Debug, Clone)]
pub struct StatusOptions {
    /// Show detailed status including file sizes
    #[arg(long)]
    pub detailed: bool,
}

impl StatusOptions {
    pub fn execute(&self) -> Result<CommandOutput> {
        let env = E2EEnvironment::new();

        // Check directories
        let config_exists = env.config_dir.exists();
        let data_exists = env.data_dir.exists();
        let state_dir = env.state_dir();
        let log_dir = env.log_dir();
        let cache_dir = MediaCache::default_cache_dir();
        let state_exists = state_dir.exists();
        let log_exists = log_dir.exists();
        let cache_exists = cache_dir.exists();
        let port_available = env.is_port_available();

        // Get server status
        let server_running = if let Some(pid) = env.server_pid {
            // Check if process is still running
            is_process_running(pid)
        } else {
            false
        };

        let mut status_info = serde_json::json!({
            "environment": {
                "config_dir": env.config_dir.display().to_string(),
                "data_dir": env.data_dir.display().to_string(),
                "state_dir": state_dir.display().to_string(),
                "log_dir": log_dir.display().to_string(),
                "cache_dir": cache_dir.display().to_string(),
                "port": env.port,
                "config_exists": config_exists,
                "data_exists": data_exists,
                "state_exists": state_exists,
                "log_exists": log_exists,
                "cache_exists": cache_exists,
            },
            "server": {
                "running": server_running,
                "port_available": port_available,
                "pid": env.server_pid,
                "url": if server_running { Some(env.server_url()) } else { None }
            }
        });

        // Add detailed info if requested
        if self.detailed {
            let mut detailed_info = serde_json::Map::new();

            if config_exists {
                detailed_info.insert(
                    "config_size_bytes".to_string(),
                    serde_json::Value::Number(dir_size(&env.config_dir).into()),
                );
            }

            if data_exists {
                detailed_info.insert(
                    "data_size_bytes".to_string(),
                    serde_json::Value::Number(dir_size(&env.data_dir).into()),
                );

                // Count items
                if cache_dir.exists() {
                    let movie_count = count_files(&cache_dir.join("movies"), Some("mp4"));
                    let music_count = count_files(&cache_dir.join("music"), Some("mp3"));

                    detailed_info.insert(
                        "media".to_string(),
                        serde_json::json!({
                            "movies": movie_count,
                            "music": music_count
                        }),
                    );
                }
            }

            if let serde_json::Value::Object(map) = status_info {
                for (k, v) in map {
                    detailed_info.insert(k, v);
                }
            }

            status_info = serde_json::Value::Object(detailed_info);
        }

        // Build envelope
        let envelope: CommandOutput = if server_running {
            OutputEnvelope::success("jellyfin e2e status", "E2E environment is active")
                .with_data(status_info)
                .with_next_step(NextStep::new(
                    "stop_server",
                    "jellyfin e2e stop",
                    "Stop the E2E server",
                ))
                .with_next_step(NextStep::new(
                    "view_logs",
                    "jellyfin e2e logs tail",
                    "View server logs",
                ))
        } else {
            OutputEnvelope::success("jellyfin e2e status", "E2E environment is configured")
                .with_data(status_info)
                .with_next_step(NextStep::new(
                    "start_server",
                    "jellyfin e2e start",
                    "Start the E2E server",
                ))
                .with_next_step(NextStep::new(
                    "download_media",
                    "jellyfin e2e media download all",
                    "Download test media",
                ))
        };

        Ok(envelope)
    }
}

/// Check if a process is running by PID (Linux-specific via /proc)
fn is_process_running(pid: u32) -> bool {
    std::path::Path::new("/proc").join(pid.to_string()).exists()
}

/// Get directory size in bytes
fn dir_size(path: &Path) -> u64 {
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

/// Count files in directory
fn count_files(path: &Path, extension: Option<&str>) -> u64 {
    let mut count = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    if let Some(ext) = extension {
                        if entry.path().extension().and_then(|s| s.to_str()) == Some(ext) {
                            count += 1;
                        }
                    } else {
                        count += 1;
                    }
                } else if metadata.is_dir() {
                    count += count_files(&entry.path(), extension);
                }
            }
        }
    }
    count
}
