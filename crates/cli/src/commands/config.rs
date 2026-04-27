//! Configuration management commands

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use jellyfin_core::Result;
/// Show current configuration
pub async fn show() -> Result<CommandOutput> {
    let config = jellyfin_core::Config::load()?;
    let config_value = serde_json::to_value(&config)?;

    let envelope: CommandOutput =
        OutputEnvelope::success("jellyfin config show", "Current configuration")
            .with_data(config_value)
            .with_next_step(NextStep::new(
                "list_servers",
                "jellyfin config list-servers",
                "List configured servers",
            ))
            .with_next_step(NextStep::new(
                "set_default",
                "jellyfin config set-default --name <NAME>",
                "Set default server",
            ));

    Ok(envelope)
}

/// List configured servers
pub async fn list_servers() -> Result<CommandOutput> {
    let config = jellyfin_core::Config::load()?;

    let servers: Vec<serde_json::Value> = config
        .servers
        .iter()
        .map(|(name, server)| {
            serde_json::json!({
                "name": name,
                "url": server.url,
                "username": server.username,
                "is_default": name == &config.default_server
            })
        })
        .collect();

    let count = servers.len();

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin config list-servers",
        format!("{} configured server(s)", count),
    )
    .with_data(serde_json::json!({
        "servers": servers,
        "default": config.default_server
    }))
    .with_next_step(NextStep::new(
        "show_config",
        "jellyfin config show",
        "Show full configuration",
    ))
    .with_next_step(NextStep::new(
        "set_default",
        "jellyfin config set-default --name <NAME>",
        "Set default server",
    ));

    Ok(envelope)
}

/// Set default server
pub async fn set_default(name: String) -> Result<CommandOutput> {
    let mut config = jellyfin_core::Config::load()?;

    if !config.servers.contains_key(&name) {
        return Err(jellyfin_core::JellyfinError::invalid_input(
            "server",
            format!("Server '{}' not found", name),
        ));
    }

    config.set_default_server(name.clone());
    config.save()?;

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin config set-default",
        format!("Set default server to '{}'", name),
    )
    .with_data(serde_json::json!({
        "default_server": name
    }))
    .with_next_step(NextStep::new(
        "verify",
        "jellyfin info",
        "Verify connection to server",
    ));

    Ok(envelope)
}

/// Remove a server configuration
pub async fn remove_server(name: String) -> Result<CommandOutput> {
    let mut config = jellyfin_core::Config::load()?;

    if !config.servers.contains_key(&name) {
        return Err(jellyfin_core::JellyfinError::invalid_input(
            "server",
            format!("Server '{}' not found", name),
        ));
    }

    config.servers.remove(&name);

    // If we removed the default, pick a new one
    let new_default = if config.default_server == name {
        config.servers.keys().next().cloned().unwrap_or_default()
    } else {
        config.default_server.clone()
    };

    config.default_server = new_default;
    config.save()?;

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin config remove-server",
        format!("Removed server '{}'", name),
    )
    .with_next_step(NextStep::new(
        "list_servers",
        "jellyfin config list-servers",
        "List remaining servers",
    ))
    .with_next_step(NextStep::new(
        "login",
        "jellyfin login",
        "Login to add a new server",
    ));

    Ok(envelope)
}
