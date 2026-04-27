//! Search commands

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use jellyfin_api::JellyfinClient;
use jellyfin_core::Result;

/// Search for media
pub async fn search(
    query: String,
    _limit: Option<u32>,
    profile: Option<&str>,
) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;

    let result = client.search_items(&query).await?;

    let items = serde_json::to_value(result)?;

    // Build output envelope
    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin search",
        format!(
            "Found {} items matching '{}'",
            items.as_array().map(|a| a.len()).unwrap_or(0),
            query
        ),
    )
    .with_data(items)
    .with_next_step(NextStep::new(
        "get_details",
        "jellyfin items get <ITEM_ID>".to_string(),
        "Get detailed information about an item",
    ))
    .with_next_step(NextStep::new(
        "play_item",
        "jellyfin play <ITEM_ID>".to_string(),
        "Play the media item",
    ));

    Ok(envelope)
}
