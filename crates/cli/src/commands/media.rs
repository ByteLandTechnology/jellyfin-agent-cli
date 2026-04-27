//! Media queries (genres, studios, actors)

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use jellyfin_api::JellyfinClient;
use jellyfin_core::Result;

/// List all genres
pub async fn genres(profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    let result = client.get_genres().await?;
    let count = result.total_record_count.unwrap_or(0) as usize;
    let value = serde_json::to_value(&result)?;

    let envelope: CommandOutput =
        OutputEnvelope::success("jellyfin genres", format!("{} genres", count))
            .with_data(value)
            .with_next_step(NextStep::new(
                "search_by_genre",
                "jellyfin search --genre <GENRE>",
                "Search media by genre",
            ));

    Ok(envelope)
}

/// List all studios
pub async fn studios(profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    let result = client.get_studios().await?;
    let count = result.total_record_count.unwrap_or(0) as usize;
    let value = serde_json::to_value(&result)?;

    let envelope: CommandOutput =
        OutputEnvelope::success("jellyfin studios", format!("{} studios", count))
            .with_data(value)
            .with_next_step(NextStep::new(
                "search_by_studio",
                "jellyfin search --studio <STUDIO>",
                "Search media by studio",
            ));

    Ok(envelope)
}

/// List all actors/artists
pub async fn actors(profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    let result = client.get_actors().await?;
    let count = result.total_record_count.unwrap_or(0) as usize;
    let value = serde_json::to_value(&result)?;

    let envelope: CommandOutput =
        OutputEnvelope::success("jellyfin actors", format!("{} actors/artists", count))
            .with_data(value)
            .with_next_step(NextStep::new(
                "search_by_actor",
                "jellyfin search --actor <ACTOR>",
                "Search media by actor",
            ));

    Ok(envelope)
}
