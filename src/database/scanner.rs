//! Filesystem scanner that imports beatmapsets into the database.
//!
//! This scanner uses rhythm-open-exchange (ROX) to support multiple chart formats:
//! .osu (mania/taiko), .qua, .sm, .ssc, .json
//!
//! Difficulty ratings are calculated on-demand when a map is selected.

use crate::database::connection::Database;
use crate::database::query::insert_beatmap;
use md5::Context;
use rhythm_open_exchange::codec::auto_decode;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Supported chart file extensions.
const SUPPORTED_EXTENSIONS: &[&str] = &["osu", "qua", "sm", "ssc"];

/// Scans the `songs/` directory and fills the database.
///
/// Note: This scanner now only extracts basic metadata (hash, notes, duration, nps).
/// Difficulty ratings are NOT calculated here - they are computed on-demand
/// when the user selects a beatmap in the song select menu.
pub async fn scan_songs_directory(
    db: &Database,
    songs_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if !songs_path.exists() {
        eprintln!("The songs/ directory does not exist");
        return Ok(());
    }

    // Walk every sub-folder under songs/.
    let entries = fs::read_dir(songs_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let chart_files = match collect_chart_files(&path) {
            Some(files) if !files.is_empty() => files,
            _ => continue,
        };

        if let Err(e) = process_beatmapset(db, &path, &chart_files).await {
            eprintln!("Error processing beatmapset {:?}: {}", path, e);
        }
    }

    Ok(())
}

/// Collect all supported chart files from a directory.
fn collect_chart_files(path: &Path) -> Option<Vec<PathBuf>> {
    let entries = fs::read_dir(path).ok()?;
    let files = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|s| s.to_str())
                .is_some_and(|ext| SUPPORTED_EXTENSIONS.contains(&ext))
        })
        .collect::<Vec<_>>();
    Some(files)
}

async fn process_beatmapset(
    db: &Database,
    folder: &Path,
    chart_files: &[PathBuf],
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(first_chart) = chart_files.first() else {
        return Ok(());
    };

    // Use ROX to decode the first chart for metadata
    let chart = auto_decode(first_chart)?;

    let title = chart.metadata.title.clone();
    let artist = chart.metadata.artist.clone();
    let image_path = chart
        .metadata
        .background_file
        .as_ref()
        .and_then(|bg| find_background_image(folder, Some(bg.as_str())));

    let Some(path_str) = folder.to_str() else {
        return Ok(());
    };

    let beatmapset_id = db
        .insert_beatmapset(
            path_str,
            image_path.as_deref(),
            Some(artist.as_str()),
            Some(title.as_str()),
        )
        .await?;

    for chart_file in chart_files {
        if let Err(e) = process_chart_file(db, beatmapset_id, chart_file).await {
            eprintln!("Error processing {:?}: {}", chart_file, e);
        }
    }

    Ok(())
}

async fn process_chart_file(
    db: &Database,
    beatmapset_id: i64,
    chart_file: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let hash = calculate_file_hash(chart_file)?;

    // Use ROX to decode the chart
    let chart = auto_decode(chart_file)?;

    // Extract basic info from ROX chart
    let note_count = chart.notes.len() as i32;

    // Calculate duration from first to last note
    let first_time = chart.notes.first().map(|n| n.time_us).unwrap_or(0);
    let last_time = chart
        .notes
        .iter()
        .map(|n| n.end_time_us())
        .max()
        .unwrap_or(first_time);
    let duration_us = last_time - first_time;
    let duration_ms = (duration_us / 1000) as i32;
    let duration_secs = duration_ms as f64 / 1000.0;
    let nps = if duration_secs > 0.0 {
        note_count as f64 / duration_secs
    } else {
        0.0
    };

    let difficulty_name = chart.metadata.difficulty_name.clone();

    if let Some(chart_str) = chart_file.to_str() {
        insert_beatmap(
            db.pool(),
            beatmapset_id,
            &hash,
            chart_str,
            Some(&difficulty_name),
            note_count,
            duration_ms,
            nps,
        )
        .await?;

        // NOTE: We no longer calculate ratings here!
        // Ratings are computed on-demand when the user selects a beatmap.
        // This dramatically speeds up the scan process.
    }

    Ok(())
}

fn find_background_image(beatmapset_path: &Path, filename: Option<&str>) -> Option<String> {
    filename.and_then(|fname| {
        let image_path = beatmapset_path.join(fname);
        if image_path.exists() {
            image_path.to_str().map(|s| s.to_string())
        } else {
            None
        }
    })
}

/// Computes the MD5 hash for a chart file.
fn calculate_file_hash(file_path: &Path) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(file_path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;

    let mut context = Context::new();
    context.consume(buffer.as_bytes());
    let result = context.finalize();
    let hash_string = format!("{:x}", result);

    Ok(hash_string)
}
