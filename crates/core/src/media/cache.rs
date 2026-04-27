//! Media cache for E2E testing
//!
//! Manages downloaded open-licensed media files.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// License type for media
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LicenseType {
    /// Creative Commons license
    CreativeCommons(String),
    /// Public Domain
    PublicDomain,
}

/// A cached media entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MediaEntry {
    /// Name of the media
    pub name: String,

    /// Source URL
    pub source_url: String,

    /// Local file path
    pub local_path: PathBuf,

    /// SHA256 checksum
    pub checksum: String,

    /// License type
    pub license: LicenseType,
}

/// Media cache manager
#[derive(Clone, Debug)]
pub struct MediaCache {
    /// Cache directory
    pub cache_dir: PathBuf,

    /// Cached entries
    pub entries: Vec<MediaEntry>,
}

impl MediaCache {
    /// Default cache directory
    pub fn default_cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("jellyfin-cli")
            .join("e2e")
            .join("media")
    }

    /// Create a new media cache
    pub fn new() -> Self {
        Self {
            cache_dir: Self::default_cache_dir(),
            entries: Vec::new(),
        }
    }

    /// Create a new media cache with custom directory
    pub fn with_dir(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            cache_dir: cache_dir.into(),
            entries: Vec::new(),
        }
    }

    /// Initialize the cache directory
    pub fn init(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.cache_dir)
    }

    /// Get a cached entry by name
    pub fn get(&self, name: &str) -> Option<&MediaEntry> {
        self.entries.iter().find(|e| e.name == name)
    }

    /// Add or update a cache entry
    pub fn add(&mut self, entry: MediaEntry) {
        // Remove existing entry with same name if present
        self.entries.retain(|e| e.name != entry.name);
        self.entries.push(entry);
    }

    /// Remove an entry from the cache
    pub fn remove(&mut self, name: &str) -> bool {
        let len_before = self.entries.len();
        self.entries.retain(|e| e.name != name);
        self.entries.len() < len_before
    }
}

impl Default for MediaCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_cache_new() {
        let cache = MediaCache::new();
        assert_eq!(cache.cache_dir, MediaCache::default_cache_dir());
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn test_media_cache_add_get() {
        let mut cache = MediaCache::new();
        let entry = MediaEntry {
            name: "test".to_string(),
            source_url: "http://example.com/test.mp4".to_string(),
            local_path: PathBuf::from("/tmp/test.mp4"),
            checksum: "abc123".to_string(),
            license: LicenseType::PublicDomain,
        };

        cache.add(entry.clone());
        assert_eq!(cache.get("test").unwrap().name, "test");
    }

    #[test]
    fn test_media_cache_remove() {
        let mut cache = MediaCache::new();
        let entry = MediaEntry {
            name: "test".to_string(),
            source_url: "http://example.com/test.mp4".to_string(),
            local_path: PathBuf::from("/tmp/test.mp4"),
            checksum: "abc123".to_string(),
            license: LicenseType::PublicDomain,
        };

        cache.add(entry);
        assert!(cache.remove("test"));
        assert!(cache.get("test").is_none());
    }
}
