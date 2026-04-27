//! Jellyfin CLI core library.
//!
//! Contains shared business logic, configuration, error handling, and output formatting.

mod config;
mod e2e;
mod error;
mod media;
mod output;
mod server;

pub use config::{
    Config, Credentials, EnvConfig, NetworkConfig, OutputConfig, PlayerConfig, ServerConfig,
};
pub use e2e::E2EEnvironment;
pub use error::{ErrorCategory, JellyfinError, Result};
pub use media::{
    BIG_BUCK_BUNNY_CHECKSUM, BIG_BUCK_BUNNY_NAME, BIG_BUCK_BUNNY_URL, CLASSICAL_MUSIC_NAME,
    CLASSICAL_MUSIC_URLS, LicenseType, MediaCache, MediaEntry, all_media_sources,
    big_buck_bunny_entry, classical_music_entries, download_file, download_file_with_limit,
    sha256_checksum, verify_checksum,
};
pub use output::{DataValue, ErrorResponse, OutputFormat, Response, ResponseResult};
pub use server::ServerManager;
