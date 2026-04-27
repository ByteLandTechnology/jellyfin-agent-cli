//! E2E integration tests
//!
//! These tests validate the end-to-end testing infrastructure.

use jellyfin_core::{E2EEnvironment, MediaCache, ServerManager, sha256_checksum};

#[test]
fn test_environment_default_paths() {
    let env = E2EEnvironment::new();
    assert_eq!(env.config_dir, E2EEnvironment::default_config_dir());
    assert_eq!(env.data_dir, E2EEnvironment::default_data_dir());
    assert_eq!(env.port, 8096);
}

#[test]
fn test_environment_custom_port() {
    let env = E2EEnvironment::new().with_port(8097);
    assert_eq!(env.port, 8097);
}

#[test]
fn test_environment_server_url() {
    let env = E2EEnvironment::new().with_port(8096);
    assert_eq!(env.server_url(), "http://127.0.0.1:8096");
}

#[test]
fn test_server_manager_new() {
    let env = E2EEnvironment::new();
    let manager = ServerManager::new(env.clone());
    assert_eq!(manager.environment.port, env.port);
}

#[test]
fn test_media_cache_new() {
    let cache = MediaCache::new();
    assert_eq!(cache.cache_dir, MediaCache::default_cache_dir());
    assert!(cache.entries.is_empty());
}

#[test]
fn test_sha256_checksum() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("checksum_test.txt");

    std::fs::write(&test_file, "Hello, checksum!").unwrap();

    let checksum = sha256_checksum(&test_file).unwrap();
    assert!(!checksum.is_empty());
    assert_eq!(checksum.len(), 64); // SHA256 is 64 hex characters

    // Cleanup
    let _ = std::fs::remove_file(&test_file);
}
