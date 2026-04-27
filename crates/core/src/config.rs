//! Configuration management for Jellyfin CLI.
//!
//! Handles loading, saving, and validating configuration from files and environment variables.

use crate::{JellyfinError, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

/// Default config directory name
const CONFIG_DIR: &str = "jellyfin-cli";
/// Default config file name
const CONFIG_FILE: &str = "config.toml";
/// Default credentials file name
const CREDENTIALS_FILE: &str = "credentials.toml";

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default server profile name
    #[serde(default = "default_server")]
    pub default_server: String,
    /// Server configurations
    #[serde(default)]
    pub servers: HashMap<String, ServerConfig>,
    /// Output settings
    #[serde(default)]
    pub output: OutputConfig,
    /// Player settings
    #[serde(default)]
    pub player: PlayerConfig,
    /// Network settings
    #[serde(default)]
    pub network: NetworkConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_server: default_server(),
            servers: HashMap::new(),
            output: OutputConfig::default(),
            player: PlayerConfig::default(),
            network: NetworkConfig::default(),
        }
    }
}

fn default_server() -> String {
    "home".to_string()
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server URL
    pub url: String,
    /// Username
    pub username: String,
    /// Authentication token (managed by login/logout)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

impl ServerConfig {
    /// Create a new server config
    pub fn new(url: String, username: String) -> Self {
        Self {
            url,
            username,
            token: None,
        }
    }

    /// Set authentication token
    pub fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Default output format
    #[serde(default = "default_output_format")]
    pub format: String,
    /// Pretty print output
    #[serde(default = "default_true")]
    pub pretty: bool,
    /// Color output
    #[serde(default = "default_color")]
    pub color: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: default_output_format(),
            pretty: true,
            color: default_color(),
        }
    }
}

fn default_output_format() -> String {
    "yaml".to_string()
}

fn default_true() -> bool {
    true
}

fn default_color() -> String {
    "auto".to_string()
}

/// Player configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    /// External player command
    #[serde(default = "default_player")]
    pub external: String,
    /// Default arguments for player
    #[serde(default)]
    pub args: Vec<String>,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            external: default_player(),
            args: vec!["--fs".to_string(), "--no-osc".to_string()],
        }
    }
}

fn default_player() -> String {
    "mpv".to_string()
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// Number of retries
    #[serde(default = "default_retries")]
    pub retries: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            timeout: default_timeout(),
            retries: default_retries(),
        }
    }
}

fn default_timeout() -> u64 {
    30
}

fn default_retries() -> usize {
    3
}

/// Credentials stored separately from config
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Credentials {
    /// Map of server name to access token
    pub tokens: HashMap<String, String>,
}

impl Config {
    /// Get the config directory path
    pub fn config_dir() -> Result<PathBuf> {
        let dir = dirs::config_dir()
            .ok_or_else(|| JellyfinError::internal("Cannot determine config directory"))?
            .join(CONFIG_DIR);

        // Ensure directory exists
        std::fs::create_dir_all(&dir).map_err(|e| {
            JellyfinError::internal(format!("Cannot create config directory: {}", e))
        })?;

        Ok(dir)
    }

    /// Get the config file path
    pub fn config_file() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join(CONFIG_FILE))
    }

    /// Get the credentials file path
    pub fn credentials_file() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join(CREDENTIALS_FILE))
    }

    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file()?;

        if !config_path.exists() {
            // Return default config if file doesn't exist
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file()?;

        let content = toml::to_string_pretty(self)
            .map_err(|e| JellyfinError::internal(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(&config_path, content)?;

        Ok(())
    }

    /// Load credentials
    pub fn load_credentials() -> Result<Credentials> {
        let cred_path = Self::credentials_file()?;

        if !cred_path.exists() {
            return Ok(Credentials::default());
        }

        let content = std::fs::read_to_string(&cred_path)?;

        // Set restrictive permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&cred_path)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&cred_path, perms)?;
        }

        let creds = toml::from_str(&content)?;

        Ok(creds)
    }

    /// Save credentials
    pub fn save_credentials(creds: &Credentials) -> Result<()> {
        let cred_path = Self::credentials_file()?;

        let content = toml::to_string_pretty(creds).map_err(|e| {
            JellyfinError::internal(format!("Failed to serialize credentials: {}", e))
        })?;

        std::fs::write(&cred_path, content)?;

        // Set restrictive permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&cred_path)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&cred_path, perms)?;
        }

        Ok(())
    }

    /// Get a specific server configuration
    pub fn get_server(&self, name: &str) -> Option<&ServerConfig> {
        self.servers.get(name)
    }

    /// Get the default server configuration
    pub fn get_default_server(&self) -> Option<&ServerConfig> {
        self.get_server(&self.default_server)
    }

    /// Add or update a server configuration
    pub fn add_server(&mut self, name: String, config: ServerConfig) {
        self.servers.insert(name, config);
    }

    /// Remove a server configuration
    pub fn remove_server(&mut self, name: &str) -> Option<ServerConfig> {
        self.servers.remove(name)
    }

    /// Set the default server
    pub fn set_default_server(&mut self, name: String) {
        self.default_server = name;
    }
}

/// Environment variable overrides
pub struct EnvConfig {
    pub server: Option<String>,
    pub token: Option<String>,
    pub output: Option<String>,
}

impl EnvConfig {
    /// Load configuration from environment variables
    pub fn load() -> Self {
        Self {
            server: std::env::var("JELLYFIN_SERVER").ok(),
            token: std::env::var("JELLYFIN_TOKEN").ok(),
            output: std::env::var("JELLYFIN_OUTPUT").ok(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.default_server, "home");
        assert_eq!(config.output.format, "yaml");
        assert_eq!(config.player.external, "mpv");
        assert_eq!(config.network.timeout, 30);
    }

    #[test]
    fn test_server_config() {
        let server = ServerConfig::new("http://localhost:8096".to_string(), "admin".to_string());
        assert_eq!(server.url, "http://localhost:8096");
        assert_eq!(server.username, "admin");
        assert!(server.token.is_none());

        let server = server.with_token("token123".to_string());
        assert_eq!(server.token, Some("token123".to_string()));
    }
}
