//! Checksum utilities for media files

use crate::Result;
use sha2::{Digest, Sha256};
use std::path::Path;

/// Compute SHA256 checksum of a file
pub fn sha256_checksum(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path)
        .map_err(|e| crate::JellyfinError::internal(format!("Failed to open file: {}", e)))?;

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = std::io::Read::read(&mut file, &mut buffer)
            .map_err(|e| crate::JellyfinError::internal(format!("Failed to read file: {}", e)))?;

        if n == 0 {
            break;
        }

        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Verify a file's checksum matches the expected value
pub fn verify_checksum(path: &Path, expected: &str) -> Result<bool> {
    let computed = sha256_checksum(path)?;
    Ok(computed == expected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_sha256_checksum() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_checksum.txt");

        let mut file = std::fs::File::create(&test_file).unwrap();
        file.write_all(b"Hello, world!").unwrap();
        drop(file);

        let checksum = sha256_checksum(&test_file).unwrap();
        assert_eq!(
            checksum,
            "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3"
        );

        std::fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_verify_checksum() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_verify.txt");

        let mut file = std::fs::File::create(&test_file).unwrap();
        file.write_all(b"test").unwrap();
        drop(file);

        // Correct checksum
        assert!(
            verify_checksum(
                &test_file,
                "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
            )
            .unwrap()
        );

        // Incorrect checksum
        assert!(
            !verify_checksum(
                &test_file,
                "0000000000000000000000000000000000000000000000000000000000000000"
            )
            .unwrap()
        );

        std::fs::remove_file(&test_file).ok();
    }
}
