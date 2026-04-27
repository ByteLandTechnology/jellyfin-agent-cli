//! Channel commands

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use jellyfin_api::JellyfinClient;
use jellyfin_core::Result;

/// List all channels
pub async fn list(profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    let result = client.get_channels().await?;
    let value = serde_json::to_value(result.items)?;
    let count = result.total_record_count.unwrap_or(0) as usize;

    let envelope: CommandOutput =
        OutputEnvelope::success("jellyfin channels list", format!("{} channels", count))
            .with_data(value)
            .with_next_step(NextStep::new(
                "get_channel_items",
                "jellyfin channels items <CHANNEL_ID>",
                "Get channel items",
            ));

    Ok(envelope)
}

/// Get channel items
pub async fn items(channel_id: String, profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    let result = client.get_channel_items(&channel_id).await?;
    let value = serde_json::to_value(result.items)?;
    let count = result.total_record_count.unwrap_or(0) as usize;

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin channels items",
        format!("{} items in channel {}", count, channel_id),
    )
    .with_data(value);

    Ok(envelope)
}

/// Handle channel subcommands
pub async fn handle(
    action: crate::ChannelCommands,
    profile: Option<&str>,
) -> Result<CommandOutput> {
    match action {
        crate::ChannelCommands::List => list(profile).await,
        crate::ChannelCommands::Items { channel_id } => items(channel_id, profile).await,
    }
}
