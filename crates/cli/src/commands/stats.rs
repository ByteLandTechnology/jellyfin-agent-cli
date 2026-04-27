//! Server statistics commands

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use jellyfin_api::JellyfinClient;
use jellyfin_core::Result;
/// Get server statistics
pub async fn info(profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;

    // Get server info
    let server_info = client.get_full_server_info().await?;

    // Build stats response
    let stats = serde_json::json!({
        "server": server_info,
    });

    let envelope: CommandOutput =
        OutputEnvelope::success("jellyfin stats", "Server statistics retrieved")
            .with_data(stats)
            .with_next_step(NextStep::new(
                "view_info",
                "jellyfin info",
                "View detailed server information",
            ))
            .with_next_step(NextStep::new(
                "list_libraries",
                "jellyfin libraries list",
                "Browse media libraries",
            ));

    Ok(envelope)
}
