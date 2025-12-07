//! Replay file storage with Zstd compression.
//!
//! Replays are stored as compressed binary files in `data/r/{hash}.r`.
//! Data is serialized with `bincode` before compression to minimize size.

use crate::models::replay::ReplayData;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use zstd::stream::{decode_all, encode_all};

/// Base directory for replay files.
const REPLAY_DIR: &str = "data/r";

/// Get the path for a replay file given its hash.
pub fn replay_path(hash: &str) -> PathBuf {
    PathBuf::from(REPLAY_DIR).join(format!("{}.r", hash))
}

/// Ensure the replay directory exists.
fn ensure_replay_dir() -> std::io::Result<()> {
    fs::create_dir_all(REPLAY_DIR)
}

/// Save replay data to a compressed binary file.
/// Returns the relative path to the file.
pub fn save_replay(hash: &str, data: &ReplayData) -> std::io::Result<String> {
    ensure_replay_dir()?;

    let path = replay_path(hash);
    let mut file = File::create(&path)?;

    // Serialize to binary first (using bincode 2.0 API)
    let binary_data =
        bincode::serde::encode_to_vec(data, bincode::config::standard()).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Serialization error: {}", e),
            )
        })?;

    // Zstd compression (Level 21 - Maximum)
    let compressed_data = encode_all(&binary_data[..], 21)?;
    file.write_all(&compressed_data)?;

    // Return relative path
    Ok(format!("{}/{}.r", REPLAY_DIR, hash))
}

/// Load and decompress replay data from file.
pub fn load_replay(hash: &str) -> std::io::Result<ReplayData> {
    let path = replay_path(hash);
    load_replay_from_path(&path)
}

/// Load replay data from a specific path.
pub fn load_replay_from_path(path: &Path) -> std::io::Result<ReplayData> {
    let file = File::open(path)?;

    // Decompress with Zstd
    let binary_data = decode_all(file)?;

    let (data, _len): (ReplayData, usize) =
        bincode::serde::decode_from_slice(&binary_data, bincode::config::standard()).map_err(
            |e| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Deserialization error: {}", e),
                )
            },
        )?;

    Ok(data)
}

/// Delete a replay file.
pub fn delete_replay(hash: &str) -> std::io::Result<()> {
    let path = replay_path(hash);
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

/// Check if a replay file exists.
pub fn replay_exists(hash: &str) -> bool {
    replay_path(hash).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::settings::HitWindowMode;

    #[test]
    fn test_compress_decompress() {
        let test_data = ReplayData::new(1.0, HitWindowMode::OsuOD, 5.0);
        let hash = "test_replay_hash";

        // Save
        let path = save_replay(hash, &test_data).unwrap();
        assert!(Path::new(&path).exists());

        // Load
        let loaded = load_replay(hash).unwrap();
        assert_eq!(loaded, test_data);

        // Cleanup
        delete_replay(hash).unwrap();
        assert!(!replay_exists(hash));
    }
}
