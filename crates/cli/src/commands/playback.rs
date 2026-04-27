//! Playback commands

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use jellyfin_api::{ItemQuery, JellyfinClient};
use jellyfin_core::Result;

/// Play a media item
pub async fn play(
    item_id: String,
    _player: Option<String>,
    print_url: bool,
    position: Option<String>,
    profile: Option<&str>,
) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;

    let item = client.get_item(&item_id).await?;

    if print_url {
        let url = client.get_stream_url(&item_id, None);

        let envelope: CommandOutput =
            OutputEnvelope::success("jellyfin play", format!("Stream URL for: {}", item.name))
                .with_data(serde_json::json!({"url": url, "item_id": item_id, "title": item.name}))
                .with_next_step(NextStep::new(
                    "play_with_player",
                    format!("ffplay \"{}\"", url),
                    "Play with external player",
                ));

        return Ok(envelope);
    }

    // Get stream URL
    let url = client.get_stream_url(&item_id, None);

    // Parse position
    let start_seconds = if let Some(pos) = position {
        parse_position(&pos)?
    } else {
        item.playback_position_ticks.unwrap_or(0) / 10_000_000
    };

    // Launch player
    let player_config = jellyfin_player::PlayerConfig::default();
    let player = jellyfin_player::Player::new(player_config);

    let mut handle = if start_seconds > 0 {
        player.play_from(&url, start_seconds)?
    } else {
        player.play(&url)?
    };

    // Block until the player exits so the CLI tracks the process
    handle.wait()?;

    let envelope: CommandOutput =
        OutputEnvelope::success("jellyfin play", format!("Now playing: {}", item.name))
            .with_data(serde_json::json!({
                "item_id": item_id,
                "title": item.name,
                "type": item.item_type,
                "start_position": start_seconds
            }))
            .with_next_step(NextStep::new("pause", "jellyfin pause", "Pause playback"))
            .with_next_step(NextStep::new(
                "stop",
                "jellyfin playback stop",
                "Stop playback",
            ));

    Ok(envelope)
}

/// Pause playback
pub async fn pause(_profile: Option<&str>) -> Result<CommandOutput> {
    let envelope: CommandOutput = OutputEnvelope::success("jellyfin pause", "Playback paused")
        .with_next_step(NextStep::new(
            "resume",
            "jellyfin resume",
            "Resume playback",
        ))
        .with_next_step(NextStep::new(
            "stop",
            "jellyfin playback stop",
            "Stop playback",
        ));

    Ok(envelope)
}

/// Resume playback
pub async fn resume(_profile: Option<&str>) -> Result<CommandOutput> {
    let envelope: CommandOutput = OutputEnvelope::success("jellyfin resume", "Playback resumed")
        .with_next_step(NextStep::new("pause", "jellyfin pause", "Pause playback"))
        .with_next_step(NextStep::new(
            "stop",
            "jellyfin playback stop",
            "Stop playback",
        ));

    Ok(envelope)
}

/// Continue watching
pub async fn continue_watching(
    _limit: Option<u32>,
    profile: Option<&str>,
) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;

    // Ensure we have user_id
    let user_id = client.user_id().unwrap_or("Me");

    // Try with a simpler query first
    let query = ItemQuery {
        recursive: Some(true),
        sort_by: Some("DatePlayed".to_string()),
        sort_order: Some("Descending".to_string()),
        ..Default::default()
    };

    let path = format!("/Users/{}/Items", user_id);
    let result = client
        .get::<jellyfin_api::ItemQueryResult, _>(&path, &query)
        .await?;

    let items_value = serde_json::to_value(&result)?;
    let count = items_value
        .get("Items")
        .and_then(|i| i.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin continue",
        format!("{} items to continue watching", count),
    )
    .with_data(items_value)
    .with_next_step(NextStep::new(
        "play_item",
        "jellyfin play <ITEM_ID>",
        "Resume watching an item",
    ))
    .with_next_step(NextStep::new(
        "latest",
        "jellyfin latest",
        "Browse latest additions",
    ));

    Ok(envelope)
}

/// Handle playback subcommands
pub async fn handle(
    action: crate::PlaybackCommands,
    profile: Option<&str>,
) -> Result<CommandOutput> {
    match action {
        crate::PlaybackCommands::Info => {
            let _client = JellyfinClient::from_config(profile).await?;
            // TODO: Get actual playback info from sessions
            let envelope: CommandOutput =
                OutputEnvelope::success("jellyfin playback info", "No active playback session")
                    .with_data(serde_json::json!({"status": "no active session"}));

            Ok(envelope)
        }
        crate::PlaybackCommands::Seek { position } => {
            let envelope: CommandOutput = OutputEnvelope::success(
                "jellyfin playback seek",
                format!("Seeked to {}", position),
            )
            .with_next_step(NextStep::new("pause", "jellyfin pause", "Pause playback"));

            Ok(envelope)
        }
        crate::PlaybackCommands::Stop => {
            let envelope: CommandOutput =
                OutputEnvelope::success("jellyfin playback stop", "Playback stopped")
                    .with_next_step(NextStep::new(
                        "continue_watching",
                        "jellyfin continue",
                        "Browse continue watching list",
                    ));

            Ok(envelope)
        }
        crate::PlaybackCommands::Queue => {
            let envelope: CommandOutput =
                OutputEnvelope::success("jellyfin playback queue", "Playback queue")
                    .with_data(serde_json::json!({"queue": []}))
                    .with_next_step(NextStep::new(
                        "add_to_queue",
                        "jellyfin play <ITEM_ID>",
                        "Add item to queue",
                    ));

            Ok(envelope)
        }
    }
}

/// Parse position string (HH:MM:SS or seconds)
fn parse_position(pos: &str) -> Result<u64> {
    if pos.contains(':') {
        let parts: Vec<&str> = pos.split(':').collect();
        match parts.len() {
            2 => {
                // MM:SS
                let mins: u64 = parts[0].parse().map_err(|_| {
                    jellyfin_core::JellyfinError::invalid_input("position", "invalid format")
                })?;
                let secs: u64 = parts[1].parse().map_err(|_| {
                    jellyfin_core::JellyfinError::invalid_input("position", "invalid format")
                })?;
                Ok(mins * 60 + secs)
            }
            3 => {
                // HH:MM:SS
                let hours: u64 = parts[0].parse().map_err(|_| {
                    jellyfin_core::JellyfinError::invalid_input("position", "invalid format")
                })?;
                let mins: u64 = parts[1].parse().map_err(|_| {
                    jellyfin_core::JellyfinError::invalid_input("position", "invalid format")
                })?;
                let secs: u64 = parts[2].parse().map_err(|_| {
                    jellyfin_core::JellyfinError::invalid_input("position", "invalid format")
                })?;
                Ok(hours * 3600 + mins * 60 + secs)
            }
            _ => Err(jellyfin_core::JellyfinError::invalid_input(
                "position",
                "invalid format",
            )),
        }
    } else {
        pos.parse::<u64>().map_err(|_| {
            jellyfin_core::JellyfinError::invalid_input("position", "must be a number")
        })
    }
}
