//! Playlist commands

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use jellyfin_api::JellyfinClient;
use jellyfin_core::Result;

/// List all playlists
pub async fn list(profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    let playlists = client.get_playlists().await?;
    let value = serde_json::to_value(playlists)?;
    let count = value.as_array().map(|a| a.len()).unwrap_or(0);

    let envelope: CommandOutput =
        OutputEnvelope::success("jellyfin playlists list", format!("{} playlists", count))
            .with_data(value)
            .with_next_step(NextStep::new(
                "get_playlist",
                "jellyfin playlists get <PLAYLIST_ID>",
                "Get playlist details",
            ))
            .with_next_step(NextStep::new(
                "create_playlist",
                "jellyfin playlists create",
                "Create a new playlist",
            ));

    Ok(envelope)
}

/// Get playlist details
pub async fn get(playlist_id: String, profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    let playlist = client.get_playlist(&playlist_id).await?;
    let value = serde_json::to_value(playlist)?;

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin playlists get",
        format!("Retrieved playlist: {}", playlist_id),
    )
    .with_data(value)
    .with_next_step(NextStep::new(
        "delete_playlist",
        format!("jellyfin playlists delete {}", playlist_id),
        "Delete this playlist",
    ));

    Ok(envelope)
}

/// Create a new playlist
pub async fn create(name: String, profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    let playlist = client.create_playlist(&name, None).await?;
    let value = serde_json::to_value(playlist)?;

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin playlists create",
        format!("Created playlist: {}", name),
    )
    .with_data(value)
    .with_next_step(NextStep::new(
        "list_playlists",
        "jellyfin playlists list",
        "List all playlists",
    ));

    Ok(envelope)
}

/// Add items to a playlist
pub async fn add(
    playlist_id: String,
    items: Vec<String>,
    profile: Option<&str>,
) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    client.add_to_playlist(&playlist_id, items).await?;

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin playlists add",
        format!("Added items to playlist: {}", playlist_id),
    )
    .with_next_step(NextStep::new(
        "get_playlist",
        format!("jellyfin playlists get {}", playlist_id),
        "View updated playlist",
    ));

    Ok(envelope)
}

/// Remove items from a playlist
pub async fn remove(
    playlist_id: String,
    items: Vec<String>,
    profile: Option<&str>,
) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    client.remove_from_playlist(&playlist_id, items).await?;

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin playlists remove",
        format!("Removed items from playlist: {}", playlist_id),
    )
    .with_next_step(NextStep::new(
        "get_playlist",
        format!("jellyfin playlists get {}", playlist_id),
        "View updated playlist",
    ));

    Ok(envelope)
}

/// Delete a playlist
pub async fn delete(playlist_id: String, profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    client.delete_playlist(&playlist_id).await?;

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin playlists delete",
        format!("Deleted playlist: {}", playlist_id),
    )
    .with_next_step(NextStep::new(
        "list_playlists",
        "jellyfin playlists list",
        "List remaining playlists",
    ));

    Ok(envelope)
}

/// Handle playlist subcommands
pub async fn handle(
    action: crate::PlaylistCommands,
    profile: Option<&str>,
) -> Result<CommandOutput> {
    match action {
        crate::PlaylistCommands::List => list(profile).await,
        crate::PlaylistCommands::Get { playlist_id } => get(playlist_id, profile).await,
        crate::PlaylistCommands::Create { name } => create(name, profile).await,
        crate::PlaylistCommands::Add { playlist_id, items } => {
            add(playlist_id, items, profile).await
        }
        crate::PlaylistCommands::Remove { playlist_id, items } => {
            remove(playlist_id, items, profile).await
        }
        crate::PlaylistCommands::Delete { playlist_id } => delete(playlist_id, profile).await,
    }
}
