//! E2E server control commands

use crate::output::{CommandOutput, ErrorDetail, NextStep, OutputEnvelope};
use clap::Parser;
use jellyfin_core::{E2EEnvironment, JellyfinError, Result, ServerManager};
use std::time::Duration;

/// Options for starting the E2E server
#[derive(Parser, Debug, Clone)]
pub struct StartOptions {
    /// Run in background
    #[arg(long)]
    pub daemon: bool,

    /// Wait for server to be ready
    #[arg(long)]
    pub wait: bool,
}

impl StartOptions {
    pub fn execute(&self) -> Result<CommandOutput> {
        let env = E2EEnvironment::new();
        let mut manager = ServerManager::new(env.clone());

        // Start the server
        manager
            .start()
            .map_err(|e| JellyfinError::internal(format!("Failed to start server: {e}")))?;

        let mut envelope: CommandOutput =
            OutputEnvelope::success("jellyfin e2e start", "Jellyfin server started")
                .with_data(serde_json::json!({
                    "port": env.port,
                    "url": env.server_url(),
                    "daemon": self.daemon
                }))
                .with_next_step(NextStep::new(
                    "open_web",
                    env.server_url(),
                    "Open Jellyfin web interface in browser",
                ))
                .with_next_step(NextStep::new(
                    "check_health",
                    format!("curl {}", env.health_url()),
                    "Check server health status",
                ));

        // Wait for server to be ready if requested
        if self.wait {
            let wait_result = manager.wait_ready(Duration::from_secs(30));

            if let Err(e) = wait_result {
                envelope = OutputEnvelope::error("jellyfin e2e start", "Command failed.")
                    .with_error(ErrorDetail::new("timeout", e.to_string()))
                    .with_next_step(NextStep::new(
                        "check_logs",
                        "jellyfin e2e logs tail",
                        "View server logs",
                    ));
            } else {
                envelope = envelope.with_data(serde_json::json!({
                    "port": env.port,
                    "url": env.server_url(),
                    "daemon": self.daemon,
                    "status": "ready"
                }));
            }
        }

        Ok(envelope)
    }
}

/// Options for stopping the E2E server
#[derive(Parser, Debug, Clone)]
pub struct StopOptions {
    /// Remove test data
    #[arg(long)]
    pub cleanup: bool,
}

impl StopOptions {
    pub fn execute(&self) -> Result<CommandOutput> {
        let env = E2EEnvironment::new();
        let manager = ServerManager::new(env.clone());

        // Stop the server
        manager
            .stop()
            .map_err(|e| JellyfinError::internal(format!("Failed to stop server: {e}")))?;

        let mut envelope: CommandOutput =
            OutputEnvelope::success("jellyfin e2e stop", "Jellyfin server stopped").with_data(
                serde_json::json!({
                    "port": env.port
                }),
            );

        // Cleanup if requested
        if self.cleanup {
            manager
                .cleanup()
                .map_err(|e| JellyfinError::internal(format!("Failed to cleanup: {e}")))?;

            envelope = envelope
                .with_data(serde_json::json!({
                    "port": env.port,
                    "cleaned": true
                }))
                .with_next_step(NextStep::new(
                    "reinitialize",
                    "jellyfin e2e setup",
                    "Reinitialize the E2E environment",
                ));
        } else {
            envelope = envelope
                .with_next_step(NextStep::new(
                    "restart",
                    "jellyfin e2e start",
                    "Restart the server",
                ))
                .with_next_step(NextStep::new(
                    "cleanup",
                    "jellyfin e2e stop --cleanup",
                    "Stop and remove test data",
                ));
        }

        Ok(envelope)
    }
}
