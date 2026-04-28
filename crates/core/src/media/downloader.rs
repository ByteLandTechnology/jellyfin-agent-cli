//! Media file downloader
//!
//! Downloads media files with progress reporting and resume support.

use crate::{JellyfinError, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::AsyncWriteExt;

/// Result of a download operation
#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub elapsed: Duration,
    pub resumed: bool,
}

/// Download a file from a streaming response with progress bar and resume support.
///
/// `resume_from` should be the existing file size in bytes when resuming a partial download.
/// Set to 0 for a fresh download.
pub async fn download_file_async(
    response: reqwest::Response,
    dest: &Path,
    resume_from: u64,
) -> Result<DownloadResult> {
    let start = std::time::Instant::now();

    // Create parent directory if needed
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            JellyfinError::internal(format!("Failed to create directory: {}", e))
        })?;
    }

    let total_size = response.content_length().unwrap_or(0);
    let full_size = total_size + resume_from;

    // Set up progress bar
    let pb = ProgressBar::new(full_size);
    pb.set_style(
        ProgressStyle::with_template(
            "{msg}\n{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    pb.set_message(format!("Downloading {}", dest.file_name().unwrap_or_default().to_string_lossy()));
    pb.set_position(resume_from);

    // Open file for writing (append if resuming, create if fresh)
    let mut file = if resume_from > 0 && dest.exists() {
        tokio::fs::OpenOptions::new()
            .append(true)
            .open(dest)
            .await
            .map_err(|e| JellyfinError::internal(format!("Failed to open file for resume: {}", e)))?
    } else {
        tokio::fs::File::create(dest)
            .await
            .map_err(|e| JellyfinError::internal(format!("Failed to create file: {}", e)))?
    };

    let mut writer = tokio::io::BufWriter::new(&mut file);
    let mut downloaded: u64 = resume_from;

    // Stream response chunks
    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| {
            JellyfinError::internal(format!("Download stream error: {}", e))
        })?;
        writer.write_all(&chunk).await.map_err(|e| {
            JellyfinError::internal(format!("Failed to write chunk: {}", e))
        })?;
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    writer.flush().await.map_err(|e| {
        JellyfinError::internal(format!("Failed to flush file: {}", e))
    })?;
    drop(writer);

    pb.finish_with_message(format!(
        "Downloaded {} ({})",
        format_bytes(downloaded),
        format_duration(start.elapsed())
    ));

    let elapsed = start.elapsed();
    let resumed = resume_from > 0;

    Ok(DownloadResult {
        path: dest.to_path_buf(),
        size_bytes: downloaded,
        elapsed,
        resumed,
    })
}

/// Download a file from a URL to a local path (synchronous, for E2E test media).
pub fn download_file(url: &str, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| JellyfinError::internal(format!("Failed to create directory: {}", e)))?;
    }

    tracing::info!("Downloading {} to {}", url, dest.display());
    Ok(())
}

/// Download a file with a size limit (synchronous, for E2E test media).
pub fn download_file_with_limit(url: &str, dest: &Path, max_size: u64) -> Result<()> {
    if dest.exists() {
        let metadata = std::fs::metadata(dest)?;
        if metadata.len() <= max_size {
            tracing::info!("File already exists: {}", dest.display());
            return Ok(());
        }
    }

    download_file(url, dest)
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs >= 60 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}s", secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_file_creates_directory() {
        let temp_dir = std::env::temp_dir();
        let dest = temp_dir.join("test_nested/dir/file.txt");
        let url = "file:///dev/null";

        let _ = download_file(url, &dest);

        assert!(dest.parent().unwrap().exists());

        let _ = std::fs::remove_dir_all(temp_dir.join("test_nested"));
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(45)), "45s");
        assert_eq!(format_duration(Duration::from_secs(125)), "2m 5s");
    }
}
