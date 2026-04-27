//! Plugin commands

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use jellyfin_api::JellyfinClient;
use jellyfin_core::Result;

/// List all plugins
pub async fn list(profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    let plugins = client.get_plugins().await?;
    let value = serde_json::to_value(plugins)?;
    let count = value.as_array().map(|a| a.len()).unwrap_or(0);

    let envelope: CommandOutput =
        OutputEnvelope::success("jellyfin plugins list", format!("{} plugins", count))
            .with_data(value)
            .with_next_step(NextStep::new(
                "get_plugin",
                "jellyfin plugins get <PLUGIN_ID>",
                "Get plugin details",
            ));

    Ok(envelope)
}

/// Get plugin details
pub async fn get(plugin_id: String, profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    let plugin = client.get_plugin(&plugin_id).await?;
    let value = serde_json::to_value(plugin)?;

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin plugins get",
        format!("Retrieved plugin: {}", plugin_id),
    )
    .with_data(value);

    Ok(envelope)
}

/// Uninstall a plugin
pub async fn uninstall(plugin_id: String, profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    client.uninstall_plugin(&plugin_id).await?;

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin plugins uninstall",
        format!("Uninstalled plugin: {}", plugin_id),
    )
    .with_next_step(NextStep::new(
        "list_plugins",
        "jellyfin plugins list",
        "List remaining plugins",
    ));

    Ok(envelope)
}

/// Handle plugin subcommands
pub async fn handle(action: crate::PluginCommands, profile: Option<&str>) -> Result<CommandOutput> {
    match action {
        crate::PluginCommands::List => list(profile).await,
        crate::PluginCommands::Get { plugin_id } => get(plugin_id, profile).await,
        crate::PluginCommands::Uninstall { plugin_id } => uninstall(plugin_id, profile).await,
    }
}
