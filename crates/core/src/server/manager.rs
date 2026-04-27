//! Jellyfin server process manager
//!
//! Manages starting, stopping, and monitoring Jellyfin server instances.

use crate::{E2EEnvironment, JellyfinError, Result};
use std::time::Duration;

/// Manages a Jellyfin server process
#[derive(Clone, Debug)]
pub struct ServerManager {
    /// The environment configuration
    pub environment: E2EEnvironment,
}

impl ServerManager {
    /// Create a new server manager
    pub fn new(environment: E2EEnvironment) -> Self {
        Self { environment }
    }

    /// Start the Jellyfin server with isolated configuration
    pub fn start(&mut self) -> Result<()> {
        // Check if port is available
        if !self.environment.is_port_available() {
            return Err(JellyfinError::invalid_input(
                "port",
                format!("Port {} is already in use", self.environment.port),
            ));
        }

        // Create directories if they don't exist
        self.environment.create_directories()?;

        // For now, we'll use a placeholder for the actual server start
        // In production, this would launch jellyfin with:
        // --config-dir <config_dir> --data-dir <data_dir> --port <port>
        tracing::info!(
            "Starting Jellyfin server on port {} with config: {}, data: {}",
            self.environment.port,
            self.environment.config_dir.display(),
            self.environment.data_dir.display()
        );

        // TODO: Implement actual process spawning
        // This would typically use std::process::Command to launch jellyfin
        // and store the PID for later management

        Ok(())
    }

    /// Wait for the server to be ready
    pub fn wait_ready(&self, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();

        // Simple TCP connection check instead of HTTP health check
        while start.elapsed() < timeout {
            if std::net::TcpStream::connect_timeout(
                &std::net::SocketAddr::from(([127, 0, 0, 1], self.environment.port)),
                Duration::from_millis(500),
            )
            .is_ok()
            {
                tracing::info!("Server is ready at {}", self.environment.server_url());
                return Ok(());
            }

            std::thread::sleep(Duration::from_millis(500));
        }

        Err(JellyfinError::internal(format!(
            "Server did not become ready within {:?}",
            timeout
        )))
    }

    /// Stop the Jellyfin server
    pub fn stop(&self) -> Result<()> {
        if let Some(pid) = self.environment.server_pid {
            tracing::info!("Stopping Jellyfin server (PID: {})", pid);

            // TODO: Implement actual process termination
            // This would send SIGTERM to the process

            Ok(())
        } else {
            tracing::warn!("No server PID set, server may not be running");
            Ok(())
        }
    }

    /// Clean up test data directories
    pub fn cleanup(&self) -> Result<()> {
        tracing::info!(
            "Cleaning up test data: {}",
            self.environment.data_dir.display()
        );

        // Remove data directory
        if self.environment.data_dir.exists() {
            std::fs::remove_dir_all(&self.environment.data_dir).map_err(|e| {
                JellyfinError::internal(format!("Failed to remove data directory: {}", e))
            })?;
        }

        // Optionally remove config directory
        if self.environment.config_dir.exists() {
            std::fs::remove_dir_all(&self.environment.config_dir).map_err(|e| {
                JellyfinError::internal(format!("Failed to remove config directory: {}", e))
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_manager_new() {
        let env = E2EEnvironment::new();
        let manager = ServerManager::new(env.clone());
        assert_eq!(manager.environment.port, env.port);
    }

    #[test]
    fn test_wait_ready_timeout() {
        let env = E2EEnvironment::new().with_port(0); // Use any available port
        let manager = ServerManager::new(env);

        // Should timeout since no server is running
        let result = manager.wait_ready(Duration::from_millis(100));
        assert!(result.is_err());
    }
}
