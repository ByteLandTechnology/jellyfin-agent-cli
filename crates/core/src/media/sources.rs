//! Open-licensed media sources
//!
//! URLs and metadata for test media files.

use super::LicenseType;

/// Big Buck Bunny (2008) - Creative Commons Attribution 3.0
///
/// A large and lovable rabbit deals with three tiny bullies,
/// in this Oscar-nominated short film.
pub const BIG_BUCK_BUNNY_NAME: &str = "big-buck-bunny";

/// Big Buck Bunny download URL
pub const BIG_BUCK_BUNNY_URL: &str =
    "https://archive.org/download/BigBuckBunny/big_buck_bunny_1080p_surround.fv4.mkv";

/// Big Buck Bunny expected SHA256 checksum
///
/// Note: This is a placeholder - the actual checksum should be verified
pub const BIG_BUCK_BUNNY_CHECKSUM: &str = "PLACEHOLDER_SHA256_CHECKSUM";

/// Big Buck Bunny entry
pub fn big_buck_bunny_entry() -> (String, String, String, LicenseType) {
    (
        BIG_BUCK_BUNNY_NAME.to_string(),
        BIG_BUCK_BUNNY_URL.to_string(),
        BIG_BUCK_BUNNY_CHECKSUM.to_string(),
        LicenseType::CreativeCommons("CC-BY 3.0".to_string()),
    )
}

/// Classical music sources (Public Domain)
///
/// Collection of classical music pieces from the Internet Archive
pub const CLASSICAL_MUSIC_NAME: &str = "classical-music";

/// Classical music URLs (Internet Archive public domain recordings)
pub const CLASSICAL_MUSIC_URLS: &[(&str, &str)] = &[
    (
        "beethoven_symphony_5",
        "https://archive.org/download/SymphonyNo.5/Beethoven_Symphony_No_5.mp3",
    ),
    (
        "bach_cello_suite",
        "https://archive.org/download/BachCelloSuites/Bach_Cello_Suite_No_1.mp3",
    ),
    (
        "mozart_piano_sonata",
        "https://archive.org/download/MozartPianoSonatas/Mozart_Piano_Sonata_K331.mp3",
    ),
];

pub fn classical_music_entries() -> Vec<(String, String, String, LicenseType)> {
    CLASSICAL_MUSIC_URLS
        .iter()
        .map(|(name, url)| {
            (
                name.to_string(),
                url.to_string(),
                "PLACEHOLDER_SHA256_CHECKSUM".to_string(),
                LicenseType::PublicDomain,
            )
        })
        .collect()
}

/// All available media sources
pub fn all_media_sources() -> Vec<(String, String, String, LicenseType)> {
    let mut sources = vec![big_buck_bunny_entry()];
    sources.extend(classical_music_entries());
    sources
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_big_buck_bunny_entry() {
        let (name, url, _checksum, license) = big_buck_bunny_entry();
        assert_eq!(name, "big-buck-bunny");
        assert!(url.contains("archive.org"));
        assert!(matches!(license, LicenseType::CreativeCommons(_)));
    }

    #[test]
    fn test_classical_music_entries() {
        let entries = classical_music_entries();
        assert!(!entries.is_empty());
        for (name, url, _checksum, license) in entries {
            assert!(!name.is_empty());
            assert!(url.contains("archive.org"));
            assert_eq!(license, LicenseType::PublicDomain);
        }
    }
}
