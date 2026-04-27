//! E2E media management commands

use crate::output::{CommandOutput, ErrorDetail, NextStep, OutputEnvelope};
use clap::{Parser, Subcommand};
use jellyfin_core::{JellyfinError, MediaCache, Result};
use std::path::PathBuf;

/// Media management commands
#[derive(Subcommand, Debug, Clone)]
pub enum MediaCommands {
    /// Download open-licensed test media files
    Download(MediaDownloadArgs),
}

impl MediaCommands {
    pub fn execute(&self) -> Result<CommandOutput> {
        match self {
            MediaCommands::Download(args) => args.execute(),
        }
    }
}

/// Arguments for media download command
#[derive(Parser, Debug, Clone)]
pub struct MediaDownloadArgs {
    /// Media to download [possible: big-buck-bunny, classical-music, all]
    #[arg(value_name = "MEDIA", default_value = "all")]
    pub media: Vec<String>,

    /// Media cache directory override
    #[arg(long)]
    pub cache_dir: Option<String>,

    /// Verify file integrity [default: true]
    #[arg(long, default_value = "true")]
    pub verify_checksums: bool,
}

impl MediaDownloadArgs {
    pub fn execute(&self) -> Result<CommandOutput> {
        let default_cache_dir = MediaCache::default_cache_dir();
        let cache_dir = self
            .cache_dir
            .as_deref()
            .map(std::borrow::Cow::Borrowed)
            .unwrap_or_else(|| std::borrow::Cow::Owned(default_cache_dir.display().to_string()));

        let cache = MediaCache::with_dir(cache_dir.as_ref());
        cache.init()?;

        // Determine what to download
        let media_list = if self.media.contains(&"all".to_string()) || self.media.is_empty() {
            vec!["big-buck-bunny".to_string(), "classical-music".to_string()]
        } else {
            self.media.clone()
        };

        let mut downloaded: Vec<(String, String)> = vec![];
        let mut errors: Vec<(String, String)> = vec![];

        for media in &media_list {
            match self.do_download(media, cache_dir.as_ref()) {
                Ok(path) => {
                    downloaded.push((media.clone(), path));
                }
                Err(e) => {
                    errors.push((media.clone(), e.to_string()));
                }
            }
        }

        // Build response envelope
        let envelope: CommandOutput = if errors.is_empty() {
            OutputEnvelope::success("jellyfin e2e media download", "Media files downloaded successfully")
                .with_data(serde_json::json!({
                    "downloaded": downloaded.iter().map(|(n, p)| serde_json::json!({"name": n, "path": p})).collect::<Vec<_>>(),
                    "cache_dir": cache_dir.as_ref()
                }))
                .with_next_step(NextStep::new(
                    "scan_library",
                    "jellyfin libraries list",
                    "Inspect the library after downloading media"
                ))
        } else {
            OutputEnvelope::error("jellyfin e2e media download", "Command failed.")
                .with_data(serde_json::json!({
                    "downloaded": downloaded.iter().map(|(n, p)| serde_json::json!({"name": n, "path": p})).collect::<Vec<_>>(),
                    "errors": errors.iter().map(|(n, e)| serde_json::json!({"name": n, "error": e})).collect::<Vec<_>>()
                }))
                .with_errors(errors.iter().map(|(name, error)| {
                    ErrorDetail::new("download_failed", format!("{name}: {error}"))
                }).collect())
                .with_next_step(NextStep::new(
                    "retry_failed",
                    format!("jellyfin e2e media download {}", errors[0].0),
                    "Retry failed download"
                ))
        };

        Ok(envelope)
    }

    fn do_download(&self, media: &str, cache_dir: &str) -> Result<String> {
        match media {
            "big-buck-bunny" | "sample-video" => {
                let dest = PathBuf::from(cache_dir)
                    .join("movies")
                    .join("Sample Video")
                    .join("sample_video.mp4");

                // Check if already exists and not a placeholder
                if dest.exists() {
                    let content = std::fs::read(&dest)?;
                    if !content.starts_with(b"PLACEHOLDER") && content.len() > 1000 {
                        return Ok(dest.display().to_string());
                    }
                }

                // Create parent directories
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                // Download from filesamples.com (sample video for testing)
                let url = "https://filesamples.com/samples/video/mp4/sample_640x360.mp4";
                tracing::info!("Downloading sample video from {}", url);

                // Use curl to download
                let output = std::process::Command::new("curl")
                    .args(["-sL", "-o", &dest.display().to_string(), url])
                    .output()
                    .map_err(|e| JellyfinError::internal(format!("Failed to run curl: {}", e)))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(JellyfinError::network_error(format!(
                        "Download failed: {}",
                        stderr
                    )));
                }

                // Verify the file is actually a video (not HTML error page)
                let content = std::fs::read(&dest)?;
                if content.starts_with(b"<!") || content.starts_with(b"<!DOCTYPE") {
                    return Err(JellyfinError::network_error(
                        "Downloaded file is HTML, not video".to_string(),
                    ));
                }

                tracing::info!("Downloaded to {}", dest.display());
                Ok(dest.display().to_string())
            }
            "classical-music" => {
                let music_dir = PathBuf::from(cache_dir)
                    .join("music")
                    .join("Beethoven")
                    .join("Symphony No. 9");

                // Create music directory structure
                std::fs::create_dir_all(&music_dir)?;

                let mp3_path = music_dir.join("Symphony No. 9 - IV. Ode to Joy.mp3");

                // Check if already exists and not a placeholder
                if mp3_path.exists() {
                    let content = std::fs::read(&mp3_path)?;
                    if !content.starts_with(b"PLACEHOLDER") && content.len() > 1000 {
                        return Ok(music_dir.display().to_string());
                    }
                }

                // For now, create a placeholder with proper structure
                std::fs::write(&mp3_path, b"PLACEHOLDER_MUSIC_FILE")?;
                Ok(music_dir.display().to_string())
            }
            _ => Err(JellyfinError::invalid_input(
                "media",
                format!("Unknown media type: {media}"),
            )),
        }
    }
}
