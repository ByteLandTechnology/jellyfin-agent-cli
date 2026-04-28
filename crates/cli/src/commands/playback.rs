//! Playback commands

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use jellyfin_api::{ItemQuery, JellyfinClient, SessionInfo};
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

/// Pause playback on a remote session
pub async fn pause(profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    let (session_id, session) = find_active_session(&client).await?;
    let item_name = session
        .now_playing_item
        .as_ref()
        .map(|i| i.name.as_str())
        .unwrap_or("unknown");

    client.send_pause(&session_id).await?;

    let envelope: CommandOutput =
        OutputEnvelope::success("jellyfin pause", format!("Paused: {}", item_name))
            .with_data(serde_json::json!({
                "session_id": session_id,
                "item": item_name
            }))
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

/// Resume playback on a remote session
pub async fn resume(profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    let (session_id, session) = find_active_session(&client).await?;
    let item_name = session
        .now_playing_item
        .as_ref()
        .map(|i| i.name.as_str())
        .unwrap_or("unknown");

    client.send_unpause(&session_id).await?;

    let envelope: CommandOutput =
        OutputEnvelope::success("jellyfin resume", format!("Resumed: {}", item_name))
            .with_data(serde_json::json!({
                "session_id": session_id,
                "item": item_name
            }))
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
            let client = JellyfinClient::from_config(profile).await?;
            let sessions = client.get_sessions().await?;
            let active: Vec<&SessionInfo> = sessions
                .iter()
                .filter(|s| s.now_playing_item.is_some())
                .collect();

            if active.is_empty() {
                return Ok(OutputEnvelope::success(
                    "jellyfin playback info",
                    "No active playback session",
                )
                .with_data(serde_json::json!({"status": "no active session"})));
            }

            let items: Vec<serde_json::Value> = active
                .iter()
                .map(|s| {
                    let item = s.now_playing_item.as_ref().unwrap();
                    let state = s.play_state.as_ref();
                    serde_json::json!({
                        "session_id": s.id,
                        "device": s.device_name,
                        "item_id": item.id,
                        "title": item.name,
                        "type": item.item_type,
                        "is_paused": state.and_then(|p| p.is_paused),
                        "position_ticks": state.and_then(|p| p.position_ticks),
                        "volume_level": state.and_then(|p| p.volume_level),
                    })
                })
                .collect();

            let envelope: CommandOutput = OutputEnvelope::success(
                "jellyfin playback info",
                format!("{} active session(s)", items.len()),
            )
            .with_data(serde_json::json!({"sessions": items}))
            .with_next_step(NextStep::new(
                "pause",
                "jellyfin pause",
                "Pause playback",
            ))
            .with_next_step(NextStep::new(
                "stop",
                "jellyfin playback stop",
                "Stop playback",
            ));

            Ok(envelope)
        }
        crate::PlaybackCommands::Seek { position } => {
            let client = JellyfinClient::from_config(profile).await?;
            let (session_id, session) = find_active_session(&client).await?;
            let seconds = parse_position(&position)?;
            let ticks = seconds * 10_000_000;

            client.send_seek(&session_id, ticks).await?;

            let item_name = session
                .now_playing_item
                .as_ref()
                .map(|i| i.name.as_str())
                .unwrap_or("unknown");

            let envelope: CommandOutput = OutputEnvelope::success(
                "jellyfin playback seek",
                format!("Seeked {} to {}", item_name, position),
            )
            .with_data(serde_json::json!({
                "session_id": session_id,
                "position": position,
                "position_ticks": ticks
            }))
            .with_next_step(NextStep::new("pause", "jellyfin pause", "Pause playback"));

            Ok(envelope)
        }
        crate::PlaybackCommands::Stop => {
            let client = JellyfinClient::from_config(profile).await?;
            let (session_id, session) = find_active_session(&client).await?;
            let item_name = session
                .now_playing_item
                .as_ref()
                .map(|i| i.name.as_str())
                .unwrap_or("unknown");

            client.send_stop(&session_id).await?;

            let envelope: CommandOutput =
                OutputEnvelope::success("jellyfin playback stop", format!("Stopped: {}", item_name))
                    .with_data(serde_json::json!({
                        "session_id": session_id,
                        "item": item_name
                    }))
                    .with_next_step(NextStep::new(
                        "continue_watching",
                        "jellyfin continue",
                        "Browse continue watching list",
                    ));

            Ok(envelope)
        }
        crate::PlaybackCommands::Queue => {
            let client = JellyfinClient::from_config(profile).await?;
            let sessions = client.get_sessions().await?;
            let queue: Vec<serde_json::Value> = sessions
                .iter()
                .filter(|s| s.now_playing_item.is_some())
                .map(|s| {
                    let item = s.now_playing_item.as_ref().unwrap();
                    let state = s.play_state.as_ref();
                    serde_json::json!({
                        "session_id": s.id,
                        "device": s.device_name,
                        "item_id": item.id,
                        "title": item.name,
                        "is_paused": state.and_then(|p| p.is_paused),
                    })
                })
                .collect();

            let count = queue.len();
            let envelope: CommandOutput =
                OutputEnvelope::success("jellyfin playback queue", format!("{} item(s) in queue", count))
                    .with_data(serde_json::json!({"queue": queue}))
                    .with_next_step(NextStep::new(
                        "add_to_queue",
                        "jellyfin play <ITEM_ID>",
                        "Add item to queue",
                    ));

            Ok(envelope)
        }
    }
}

/// Find an active session (one with now_playing_item) across all sessions
async fn find_active_session(client: &JellyfinClient) -> Result<(String, SessionInfo)> {
    let sessions = client.get_sessions().await?;
    sessions
        .into_iter()
        .find(|s| s.now_playing_item.is_some())
        .and_then(|s| s.id.clone().map(|id| (id, s)))
        .ok_or_else(|| {
            jellyfin_core::JellyfinError::not_found(
                "active playback session".to_string(),
            )
        })
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
