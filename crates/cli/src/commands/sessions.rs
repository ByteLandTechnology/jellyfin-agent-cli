//! Session commands

use crate::output::{CommandOutput, OutputEnvelope};
use jellyfin_api::JellyfinClient;
use jellyfin_core::Result;

/// List all active sessions
pub async fn list(profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    let sessions = client.get_sessions().await?;
    let value = serde_json::to_value(sessions)?;
    let count = value.as_array().map(|a| a.len()).unwrap_or(0);

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin sessions list",
        format!("{} active sessions", count),
    )
    .with_data(value);

    Ok(envelope)
}
