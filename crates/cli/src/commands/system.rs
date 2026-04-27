//! System commands

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use jellyfin_api::JellyfinClient;
use jellyfin_core::Result;

/// Get server info
pub async fn info(profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    let info = client.get_full_server_info().await?;
    let info_value = serde_json::to_value(info)?;

    let envelope: CommandOutput =
        OutputEnvelope::success("jellyfin info", "Server information retrieved")
            .with_data(info_value)
            .with_next_step(NextStep::new(
                "view_stats",
                "jellyfin stats",
                "View server statistics",
            ))
            .with_next_step(NextStep::new(
                "list_users",
                "jellyfin users list",
                "List server users",
            ));

    Ok(envelope)
}

/// Restart the server
pub async fn restart(profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    client.restart_server().await?;

    let envelope: CommandOutput =
        OutputEnvelope::success("jellyfin system restart", "Server restart initiated")
            .with_next_step(NextStep::new(
                "check_status",
                "jellyfin info",
                "Check if server is back online",
            ));

    Ok(envelope)
}

/// Shutdown the server
pub async fn shutdown(profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    client.shutdown_server().await?;

    let envelope: CommandOutput =
        OutputEnvelope::success("jellyfin system shutdown", "Server shutdown initiated")
            .with_next_step(NextStep::new(
                "login",
                "jellyfin login",
                "Login again after restart",
            ));

    Ok(envelope)
}

/// Handle system subcommands
pub async fn handle(action: crate::SystemCommands, profile: Option<&str>) -> Result<CommandOutput> {
    match action {
        crate::SystemCommands::Restart => restart(profile).await,
        crate::SystemCommands::Shutdown => shutdown(profile).await,
    }
}
