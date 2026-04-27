//! Active context management commands

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use jellyfin_core::Result;

/// Show current active context (server profile)
pub async fn show() -> Result<CommandOutput> {
    let config = jellyfin_core::Config::load()?;

    let active_context_name = config.default_server.clone();
    let server_info = config.servers.get(&active_context_name);

    let envelope: CommandOutput =
        OutputEnvelope::success("jellyfin context show", "Current active context")
            .with_data(serde_json::json!({
                "active_context": active_context_name,
                "server": server_info.map(|s| serde_json::json!({
                    "url": s.url,
                    "username": s.username,
                    "is_default": true
                }))
            }))
            .with_next_step(NextStep::new(
                "list_servers",
                "jellyfin config list-servers",
                "List all configured servers",
            ))
            .with_next_step(NextStep::new(
                "use_context",
                "jellyfin context use <NAME>",
                "Switch to a different context",
            ));

    Ok(envelope)
}

/// Switch to a different active context (server profile)
pub async fn use_context(name: String) -> Result<CommandOutput> {
    let mut config = jellyfin_core::Config::load()?;

    if !config.servers.contains_key(&name) {
        return Err(jellyfin_core::JellyfinError::invalid_input(
            "context",
            format!(
                "Context '{}' not found. Use 'jellyfin-agent-cli config list-servers' to see available contexts.",
                name
            ),
        ));
    }

    let previous = config.default_server.clone();
    config.set_default_server(name.clone());
    config.save()?;

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin context use",
        format!("Switched active context from '{}' to '{}'", previous, name),
    )
    .with_data(serde_json::json!({
        "previous_context": previous,
        "active_context": name
    }))
    .with_next_step(NextStep::new(
        "verify",
        "jellyfin info",
        "Verify connection to the new context",
    ));

    Ok(envelope)
}
