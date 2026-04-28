//! Media management module

pub mod cache;
pub mod checksum;
pub mod downloader;
pub mod sources;

pub use cache::{LicenseType, MediaCache, MediaEntry};
pub use checksum::{sha256_checksum, verify_checksum};
pub use downloader::{download_file, download_file_async, download_file_with_limit, format_bytes, DownloadResult};
pub use sources::{
    BIG_BUCK_BUNNY_CHECKSUM, BIG_BUCK_BUNNY_NAME, BIG_BUCK_BUNNY_URL, CLASSICAL_MUSIC_NAME,
    CLASSICAL_MUSIC_URLS, all_media_sources, big_buck_bunny_entry, classical_music_entries,
};
