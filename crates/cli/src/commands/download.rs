//! Download command

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use jellyfin_api::JellyfinClient;
use jellyfin_core::Result;
use std::path::PathBuf;

/// Download a media item
pub async fn download(
    item_id: String,
    output: Option<String>,
    resume: bool,
    verify: bool,
    profile: Option<&str>,
) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;

    let item = client.get_item(&item_id).await?;

    // Determine output path
    let dest = if let Some(ref path) = output {
        PathBuf::from(path)
    } else {
        let filename = derive_filename(&item.name, &item.item_type);
        PathBuf::from(filename)
    };

    // Check for resume
    let resume_from = if resume && dest.exists() {
        let metadata = std::fs::metadata(&dest)?;
        let existing_size = metadata.len();
        if existing_size > 0 {
            tracing::info!(
                "Resuming download from {} bytes: {}",
                existing_size,
                dest.display()
            );
            existing_size
        } else {
            0
        }
    } else {
        0
    };

    // Get download stream
    let response = client.download_stream(&item_id).await?;

    // Download with progress
    let result = jellyfin_core::download_file_async(response, &dest, resume_from).await?;

    // Optional checksum verification
    let checksum_info = if verify {
        match jellyfin_core::sha256_checksum(&result.path) {
            Ok(hash) => Some(hash),
            Err(e) => {
                tracing::warn!("Checksum verification failed: {}", e);
                None
            }
        }
    } else {
        None
    };

    let speed = if result.elapsed.as_secs() > 0 {
        format!(
            "{}/s",
            jellyfin_core::format_bytes(result.size_bytes / result.elapsed.as_secs())
        )
    } else {
        "N/A".to_string()
    };

    let mut data = serde_json::json!({
        "item_id": item_id,
        "title": item.name,
        "path": result.path.to_string_lossy(),
        "size_bytes": result.size_bytes,
        "elapsed_secs": result.elapsed.as_secs(),
        "speed": speed,
        "resumed": result.resumed,
    });

    if let Some(hash) = checksum_info {
        data["sha256"] = serde_json::Value::String(hash);
    }

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin download",
        format!("Downloaded: {}", item.name),
    )
    .with_data(data)
    .with_next_step(NextStep::new(
        "play",
        format!("jellyfin play {}", item_id),
        "Play the item",
    ));

    Ok(envelope)
}

fn derive_filename(name: &str, item_type: &str) -> String {
    let extension = match item_type.to_lowercase().as_str() {
        "movie" | "video" => "mkv",
        "episode" => "mkv",
        "audio" | "musicalbum" | "music" => "mp3",
        "photo" => "jpg",
        _ => "bin",
    };

    let sanitized: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' })
        .collect();

    format!("{}.{}", sanitized.trim(), extension)
}
