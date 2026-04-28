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
        if !self.environment.is_port_available() {
            return Err(JellyfinError::invalid_input(
                "port",
                format!("Port {} is already in use", self.environment.port),
            ));
        }

        self.environment.create_directories()?;

        let binary = Self::find_jellyfin_binary()?;

        let child = std::process::Command::new(&binary)
            .arg("--configdir")
            .arg(&self.environment.config_dir)
            .arg("--datadir")
            .arg(&self.environment.data_dir)
            .arg("--port")
            .arg(self.environment.port.to_string())
            .arg("--logdir")
            .arg(self.environment.log_dir())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| {
                JellyfinError::internal(format!("Failed to start Jellyfin server: {}", e))
            })?;

        let pid = child.id();
        self.environment.server_pid = Some(pid);
        self.environment.save_pid(pid)?;

        tracing::info!(
            "Started Jellyfin server on port {} (PID: {}) with config: {}, data: {}",
            self.environment.port,
            pid,
            self.environment.config_dir.display(),
            self.environment.data_dir.display()
        );

        // Drop the child handle so the process keeps running independently
        drop(child);

        Ok(())
    }

    /// Wait for the server to be ready
    pub fn wait_ready(&self, timeout: Duration) -> Result<()> {
        let start = std::time::Instant::now();

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
    pub fn stop(&mut self) -> Result<()> {
        if let Some(pid) = self.environment.server_pid {
            tracing::info!("Stopping Jellyfin server (PID: {})", pid);

            #[cfg(unix)]
            {
                let kill_result = std::process::Command::new("kill")
                    .arg("-TERM")
                    .arg(pid.to_string())
                    .output();

                match kill_result {
                    Ok(output) if output.status.success() => {
                        // Wait for graceful shutdown
                        let deadline = std::time::Instant::now() + Duration::from_secs(10);
                        while std::time::Instant::now() < deadline {
                            if !Self::is_process_running(pid) {
                                break;
                            }
                            std::thread::sleep(Duration::from_millis(500));
                        }
                    }
                    Ok(_) => {
                        // Process may have already exited
                        tracing::warn!(
                            "kill -TERM {} did not succeed (process may have already exited)",
                            pid
                        );
                    }
                    Err(e) => {
                        return Err(JellyfinError::internal(format!(
                            "Failed to send SIGTERM to PID {}: {}",
                            pid, e
                        )));
                    }
                }
            }

            #[cfg(windows)]
            {
                let _ = std::process::Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/F"])
                    .output();
            }

            self.environment.server_pid = None;
            self.environment.clear_pid()?;

            tracing::info!("Stopped Jellyfin server (PID: {})", pid);
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

        if self.environment.data_dir.exists() {
            std::fs::remove_dir_all(&self.environment.data_dir).map_err(|e| {
                JellyfinError::internal(format!("Failed to remove data directory: {}", e))
            })?;
        }

        if self.environment.config_dir.exists() {
            std::fs::remove_dir_all(&self.environment.config_dir).map_err(|e| {
                JellyfinError::internal(format!("Failed to remove config directory: {}", e))
            })?;
        }

        Ok(())
    }

    /// Check if a process with the given PID is still running
    fn is_process_running(pid: u32) -> bool {
        #[cfg(unix)]
        {
            std::process::Command::new("kill")
                .arg("-0")
                .arg(pid.to_string())
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }

        #[cfg(windows)]
        {
            std::process::Command::new("tasklist")
                .args(["/FI", &format!("PID eq {}", pid)])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
                .unwrap_or(false)
        }
    }

    /// Find the jellyfin binary in PATH or common locations
    fn find_jellyfin_binary() -> Result<String> {
        let candidates = ["jellyfin", "/usr/bin/jellyfin", "/usr/local/bin/jellyfin"];

        for candidate in candidates {
            if std::path::Path::new(candidate).is_file() {
                return Ok(candidate.to_string());
            }
            // Check via `which` for PATH-based lookup
            if let Ok(output) = std::process::Command::new("which")
                .arg(candidate)
                .output()
            {
                if output.status.success()
                    && !String::from_utf8_lossy(&output.stdout).trim().is_empty()
                {
                    return Ok(String::from_utf8_lossy(&output.stdout)
                        .trim()
                        .to_string());
                }
            }
        }

        Err(JellyfinError::internal(
            "jellyfin binary not found. Install Jellyfin server or add it to PATH.".to_string(),
        ))
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
        let env = E2EEnvironment::new().with_port(0);
        let manager = ServerManager::new(env);

        let result = manager.wait_ready(Duration::from_millis(100));
        assert!(result.is_err());
    }
}
