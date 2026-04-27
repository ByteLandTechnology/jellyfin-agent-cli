//! E2E environment setup command

use crate::output::{CommandOutput, ErrorDetail, NextStep, OutputEnvelope};
use clap::Parser;
use jellyfin_core::{E2EEnvironment, JellyfinError, Result};

/// Options for the E2E setup command
#[derive(Parser, Debug, Clone)]
pub struct SetupOptions {
    /// Jellyfin config directory override
    #[arg(long)]
    pub config_dir: Option<String>,

    /// Jellyfin data directory override
    #[arg(long)]
    pub data_dir: Option<String>,

    /// Server port [default: 8096]
    #[arg(long, default_value = "8096")]
    pub port: u16,
}

impl SetupOptions {
    pub fn execute(&self) -> Result<CommandOutput> {
        // Create environment, applying partial path overrides individually
        let mut env = E2EEnvironment::new();
        if let Some(config) = &self.config_dir {
            env.config_dir = std::path::PathBuf::from(config);
        }
        if let Some(data) = &self.data_dir {
            env.data_dir = std::path::PathBuf::from(data);
        }
        env.port = self.port;

        // Check if port is available
        if !env.is_port_available() {
            let envelope: CommandOutput =
                OutputEnvelope::error("jellyfin e2e setup", "Command failed.")
                    .with_error(ErrorDetail::new(
                        "port_in_use",
                        format!("Port {} is already in use by another process.", env.port),
                    ))
                    .with_next_step(NextStep::new(
                        "check_port",
                        format!("lsof -i :{} || netstat -an | grep :{}", env.port, env.port),
                        "Check what process is using the port",
                    ))
                    .with_next_step(NextStep::new(
                        "try_different_port",
                        "jellyfin e2e setup --port 8097",
                        "Try with a different port",
                    ));

            return Ok(envelope);
        }

        // Create directories
        env.create_directories()
            .map_err(|e| JellyfinError::internal(format!("Failed to create directories: {e}")))?;

        // Build success envelope
        let config_dir = env.config_dir.display().to_string();
        let data_dir = env.data_dir.display().to_string();
        let port = env.port;

        let envelope: CommandOutput = OutputEnvelope::success(
            "jellyfin e2e setup",
            "E2E environment initialized successfully",
        )
        .with_data(serde_json::json!({
            "config_dir": config_dir,
            "data_dir": data_dir,
            "port": port
        }))
        .with_next_step(NextStep::new(
            "download_media",
            "jellyfin e2e media download all",
            "Download open-licensed test media",
        ))
        .with_next_step(NextStep::new(
            "start_server",
            "jellyfin e2e start",
            "Start the Jellyfin server",
        ));

        Ok(envelope)
    }
}
