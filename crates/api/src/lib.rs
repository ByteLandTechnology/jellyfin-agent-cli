//! Jellyfin API client library.
//!
//! Provides a type-safe client for the Jellyfin API.

pub mod client;
pub mod types;

// Re-export common types
pub use client::JellyfinClient;
pub use types::*;
