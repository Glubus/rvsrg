//! Replay file storage with Brotli compression.
//!
//! Replays are stored as compressed files in `data/r/{hash}.r` instead of
//! in the database to reduce DB size.

use brotli::{CompressorWriter, Decompressor};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

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

/// Save replay data to a compressed file.
/// Returns the relative path to the file.
pub fn save_replay(hash: &str, data: &str) -> std::io::Result<String> {
    ensure_replay_dir()?;
    
    let path = replay_path(hash);
    let file = File::create(&path)?;
    
    // Brotli quality 11 (max), window size 22 (4MB)
    let mut compressor = CompressorWriter::new(file, 4096, 11, 22);
    compressor.write_all(data.as_bytes())?;
    compressor.flush()?;
    drop(compressor); // Ensure file is closed
    
    // Return relative path
    Ok(format!("{}/{}.r", REPLAY_DIR, hash))
}

/// Load and decompress replay data from file.
pub fn load_replay(hash: &str) -> std::io::Result<String> {
    let path = replay_path(hash);
    load_replay_from_path(&path)
}

/// Load replay data from a specific path.
pub fn load_replay_from_path(path: &Path) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut decompressor = Decompressor::new(file, 4096);
    
    let mut decompressed = String::new();
    decompressor.read_to_string(&mut decompressed)?;
    
    Ok(decompressed)
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

    #[test]
    fn test_compress_decompress() {
        let test_data = r#"{"hits":[1,2,3],"misses":0}"#;
        let hash = "test_replay_hash";
        
        // Save
        let path = save_replay(hash, test_data).unwrap();
        assert!(Path::new(&path).exists());
        
        // Load
        let loaded = load_replay(hash).unwrap();
        assert_eq!(loaded, test_data);
        
        // Cleanup
        delete_replay(hash).unwrap();
        assert!(!replay_exists(hash));
    }
}
