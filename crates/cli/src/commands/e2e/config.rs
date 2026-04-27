//! E2E config command

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use clap::{Args, Subcommand};
use jellyfin_core::{E2EEnvironment, JellyfinError, Result};
use serde::{Deserialize, Serialize};
use serde_json;

/// E2E configuration commands
#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCommands {
    /// Show current E2E configuration
    Show,

    /// Set a configuration value
    Set(ConfigSetOptions),

    /// Reset configuration to defaults
    Reset,
}

impl ConfigCommands {
    pub fn execute(&self) -> Result<CommandOutput> {
        match self {
            ConfigCommands::Show => self.show_config(),
            ConfigCommands::Set(opts) => opts.execute(),
            ConfigCommands::Reset => self.reset_config(),
        }
    }

    fn show_config(&self) -> Result<CommandOutput> {
        let env = E2EEnvironment::new();
        let media_dir = jellyfin_core::MediaCache::default_cache_dir();

        let config_path = E2EEnvironment::default_config_file();
        let persisted = load_e2e_config(&config_path);

        let mut config = serde_json::json!({
            "config_dir": env.config_dir.display().to_string(),
            "data_dir": env.data_dir.display().to_string(),
            "media_dir": media_dir.display().to_string(),
            "port": env.port,
            "server_url": env.server_url()
        });

        if let Some(ref persisted_config) = persisted {
            config["config_file"] = serde_json::json!(config_path.display().to_string());
            config["persisted"] = serde_json::json!(true);
            if let Some(ref dir) = persisted_config.config_dir {
                config["config_dir"] = serde_json::json!(dir);
            }
            if let Some(ref dir) = persisted_config.data_dir {
                config["data_dir"] = serde_json::json!(dir);
            }
            if let Some(port) = persisted_config.port {
                config["port"] = serde_json::json!(port);
            }
        }

        let envelope: CommandOutput =
            OutputEnvelope::success("jellyfin e2e config show", "E2E configuration")
                .with_data(config)
                .with_next_step(NextStep::new(
                    "edit_config",
                    "jellyfin e2e config set <KEY> <VALUE>",
                    "Modify configuration",
                ))
                .with_next_step(NextStep::new(
                    "reset_config",
                    "jellyfin e2e config reset",
                    "Reset to defaults",
                ));

        Ok(envelope)
    }

    fn reset_config(&self) -> Result<CommandOutput> {
        // Remove persisted config file
        let config_path = E2EEnvironment::default_config_file();
        let removed = config_path.exists();
        if removed {
            std::fs::remove_file(&config_path)?;
        }

        let envelope: CommandOutput = OutputEnvelope::success(
            "jellyfin e2e config reset",
            "Configuration reset to defaults",
        )
        .with_data(serde_json::json!({
            "config_dir": E2EEnvironment::default_config_dir().display().to_string(),
            "data_dir": E2EEnvironment::default_data_dir().display().to_string(),
            "media_dir": jellyfin_core::MediaCache::default_cache_dir().display().to_string(),
            "port": 8096
        }))
        .with_next_step(NextStep::new(
            "show_config",
            "jellyfin e2e config show",
            "View configuration",
        ));

        Ok(envelope)
    }
}

/// Options for setting configuration
#[derive(Args, Debug, Clone)]
pub struct ConfigSetOptions {
    /// Configuration key to set (port, config_dir, data_dir)
    #[arg(short = 'k', long)]
    pub key: String,

    /// Configuration value
    #[arg(short = 'v', long)]
    pub value: String,
}

impl ConfigSetOptions {
    pub fn execute(&self) -> Result<CommandOutput> {
        let key = self.key.as_str();
        let value = self.value.clone();

        // Validate value
        match key {
            "port" => {
                value.parse::<u16>().map_err(|_| {
                    JellyfinError::invalid_input("port", format!("Invalid port: {value}"))
                })?;
            }
            "config_dir" | "data_dir" => {}
            _ => {
                return Err(JellyfinError::invalid_input(
                    "configuration key",
                    format!("Unknown configuration key: {key}"),
                ));
            }
        }

        // Load existing config or create default
        let config_path = E2EEnvironment::default_config_file();
        let mut config = load_e2e_config(&config_path).unwrap_or_default();

        // Apply the change
        match key {
            "port" => config.port = Some(value.parse().unwrap()),
            "config_dir" => config.config_dir = Some(value.clone()),
            "data_dir" => config.data_dir = Some(value.clone()),
            _ => unreachable!(),
        }

        // Persist
        save_e2e_config(&config_path, &config)?;

        let result = format!("{} set to {}", key, self.value);

        let envelope: CommandOutput = OutputEnvelope::success("jellyfin e2e config set", result)
            .with_data(serde_json::json!({
                "key": key,
                "value": self.value,
                "config_file": config_path.display().to_string()
            }))
            .with_next_step(NextStep::new(
                "show_config",
                "jellyfin e2e config show",
                "View updated configuration",
            ));

        Ok(envelope)
    }
}

/// Persisted E2E configuration
#[derive(Default, Debug, Serialize, Deserialize)]
struct E2EConfig {
    port: Option<u16>,
    config_dir: Option<String>,
    data_dir: Option<String>,
}

/// Load E2E config from TOML file
fn load_e2e_config(path: &std::path::Path) -> Option<E2EConfig> {
    let content = std::fs::read_to_string(path).ok()?;
    toml::from_str(&content).ok()
}

/// Save E2E config to TOML file
fn save_e2e_config(path: &std::path::Path, config: &E2EConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)
        .map_err(|e| JellyfinError::internal(format!("Failed to serialize config: {e}")))?;
    std::fs::write(path, content)?;
    Ok(())
}
