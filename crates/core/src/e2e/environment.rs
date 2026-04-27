//! E2E test environment configuration
//!
//! Manages isolated Jellyfin server instances for end-to-end testing.

use crate::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// E2E test environment configuration
#[derive(Clone, Debug)]
pub struct E2EEnvironment {
    /// Jellyfin configuration directory
    pub config_dir: PathBuf,

    /// Jellyfin data directory
    pub data_dir: PathBuf,

    /// Server port
    pub port: u16,

    /// Running server process ID
    pub server_pid: Option<u32>,
}

impl E2EEnvironment {
    /// Default port
    pub const DEFAULT_PORT: u16 = 8096;

    /// Default configuration directory for the E2E environment.
    pub fn default_config_dir() -> PathBuf {
        user_scoped_root(dirs::config_dir()).join("config")
    }

    /// Default data directory for the E2E environment.
    pub fn default_data_dir() -> PathBuf {
        user_scoped_root(dirs::data_local_dir()).join("data")
    }

    /// Default state directory for the E2E environment.
    pub fn default_state_dir() -> PathBuf {
        Self::default_data_dir().join("state")
    }

    /// Default log directory for the E2E environment.
    pub fn default_log_dir() -> PathBuf {
        Self::default_state_dir().join("logs")
    }

    /// Default path for the persisted E2E config file.
    pub fn default_config_file() -> PathBuf {
        Self::default_config_dir().join("e2e-config.toml")
    }

    /// Create a new E2E environment with default paths
    pub fn new() -> Self {
        let mut env = Self {
            config_dir: Self::default_config_dir(),
            data_dir: Self::default_data_dir(),
            port: Self::DEFAULT_PORT,
            server_pid: None,
        };
        env.load_persisted_config();
        env.refresh_server_pid();
        env
    }

    /// Load persisted E2E config file if it exists
    fn load_persisted_config(&mut self) {
        let config_path = Self::default_config_file();
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str::<E2EConfigFile>(&content) {
                if let Some(port) = config.port {
                    self.port = port;
                }
                if let Some(ref dir) = config.config_dir {
                    self.config_dir = PathBuf::from(dir);
                }
                if let Some(ref dir) = config.data_dir {
                    self.data_dir = PathBuf::from(dir);
                }
                self.refresh_server_pid();
            }
        }
    }

    /// Load server PID from disk if the pid file exists
    fn load_pid(pid_file: &Path) -> Option<u32> {
        if let Ok(content) = std::fs::read_to_string(pid_file) {
            content.trim().parse::<u32>().ok()
        } else {
            None
        }
    }

    /// State directory derived from the effective data directory.
    pub fn state_dir(&self) -> PathBuf {
        self.data_dir.join("state")
    }

    /// Log directory derived from the effective state directory.
    pub fn log_dir(&self) -> PathBuf {
        self.state_dir().join("logs")
    }

    fn pid_file(&self) -> PathBuf {
        self.state_dir().join("server.pid")
    }

    fn refresh_server_pid(&mut self) {
        self.server_pid = Self::load_pid(&self.pid_file());
    }

    /// Persist server PID to disk
    pub fn save_pid(&self, pid: u32) -> Result<()> {
        std::fs::create_dir_all(self.state_dir()).map_err(|e| {
            crate::JellyfinError::internal(format!("Failed to create state directory: {}", e))
        })?;
        let pid_file = self.pid_file();
        std::fs::write(&pid_file, pid.to_string()).map_err(|e| {
            crate::JellyfinError::internal(format!("Failed to write PID file: {}", e))
        })?;
        Ok(())
    }

    /// Remove persisted PID file
    pub fn clear_pid(&self) -> Result<()> {
        let pid_file = self.pid_file();
        if pid_file.exists() {
            std::fs::remove_file(&pid_file).map_err(|e| {
                crate::JellyfinError::internal(format!("Failed to remove PID file: {}", e))
            })?;
        }
        Ok(())
    }

    /// Create a new E2E environment with custom paths
    pub fn with_paths(config_dir: impl Into<PathBuf>, data_dir: impl Into<PathBuf>) -> Self {
        Self {
            config_dir: config_dir.into(),
            data_dir: data_dir.into(),
            port: Self::DEFAULT_PORT,
            server_pid: None,
        }
    }

    /// Set the port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Create the necessary directories
    pub fn create_directories(&self) -> Result<()> {
        std::fs::create_dir_all(&self.config_dir).map_err(|e| {
            crate::JellyfinError::internal(format!("Failed to create config directory: {}", e))
        })?;

        std::fs::create_dir_all(&self.data_dir).map_err(|e| {
            crate::JellyfinError::internal(format!("Failed to create data directory: {}", e))
        })?;

        std::fs::create_dir_all(self.state_dir()).map_err(|e| {
            crate::JellyfinError::internal(format!("Failed to create state directory: {}", e))
        })?;

        std::fs::create_dir_all(self.log_dir()).map_err(|e| {
            crate::JellyfinError::internal(format!("Failed to create log directory: {}", e))
        })?;

        Ok(())
    }

    /// Check if the configured port is available
    pub fn is_port_available(&self) -> bool {
        use std::net::TcpListener;

        TcpListener::bind(("127.0.0.1", self.port)).is_ok()
    }

    /// Get the server URL for this environment
    pub fn server_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// Get the health check URL for this environment
    pub fn health_url(&self) -> String {
        format!("{}/health", self.server_url())
    }
}

impl Default for E2EEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

fn user_scoped_root(base: Option<PathBuf>) -> PathBuf {
    base.unwrap_or_else(std::env::temp_dir)
        .join("jellyfin-cli")
        .join("e2e")
}

/// Minimal config file struct for deserialization
#[derive(Deserialize)]
struct E2EConfigFile {
    port: Option<u16>,
    config_dir: Option<String>,
    data_dir: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_environment() {
        let env = E2EEnvironment::new();
        assert_eq!(env.port, 8096);
        assert_eq!(env.config_dir, E2EEnvironment::default_config_dir());
        assert_eq!(env.data_dir, E2EEnvironment::default_data_dir());
        assert_eq!(env.state_dir(), E2EEnvironment::default_state_dir());
        assert_eq!(env.log_dir(), E2EEnvironment::default_log_dir());
    }

    #[test]
    fn test_custom_port() {
        let env = E2EEnvironment::new().with_port(8097);
        assert_eq!(env.port, 8097);
    }

    #[test]
    fn test_server_url() {
        let env = E2EEnvironment::new().with_port(8096);
        assert_eq!(env.server_url(), "http://127.0.0.1:8096");
        assert_eq!(env.health_url(), "http://127.0.0.1:8096/health");
    }

    #[test]
    fn test_is_port_available() {
        let env = E2EEnvironment::new().with_port(0); // Port 0 will assign any available port
        assert!(env.is_port_available());
    }

    #[test]
    fn test_state_and_log_paths_follow_data_dir() {
        let env = E2EEnvironment::with_paths("/tmp/e2e-config", "/tmp/e2e-data");
        assert_eq!(env.state_dir(), PathBuf::from("/tmp/e2e-data/state"));
        assert_eq!(env.log_dir(), PathBuf::from("/tmp/e2e-data/state/logs"));
    }
}
