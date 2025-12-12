//! Filesystem scanner that imports beatmapsets into the database.
//!
//! This scanner uses rhythm-open-exchange (ROX) to support multiple chart formats:
//! .osu (mania/taiko), .qua, .sm, .ssc, .json
//!
//! Difficulty ratings are calculated on-demand when a map is selected.

use crate::database::connection::Database;
use crate::database::query::insert_beatmap;
use rhythm_open_exchange::codec::auto_decode;
use std::collections::HashMap;
use std::fs;
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
    // Use ROX to decode the chart
    let chart = auto_decode(chart_file)?;

    // Use ROX's blake3 hash instead of MD5
    let hash = chart.hash();

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

    // Extract dominant BPM (the one that lasts the longest, ignoring SV changes)
    let bpm = calculate_dominant_bpm(&chart.timing_points, last_time);

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
            bpm,
        )
        .await?;

        // Calculate and save difficulty ratings during scan
        calculate_and_save_ratings(db, &hash, &chart).await;
    }

    Ok(())
}

/// Calculate difficulty ratings using available calculators and save to DB.
async fn calculate_and_save_ratings(
    db: &Database,
    hash: &str,
    chart: &rhythm_open_exchange::RoxChart,
) {
    use crate::difficulty::{calculate_on_demand, rox_chart_to_rosu};

    // Convert RoxChart to rosu Beatmap format
    let rosu_beatmap = match rox_chart_to_rosu(chart) {
        Ok(bm) => bm,
        Err(e) => {
            log::warn!("Failed to convert chart {} for rating: {}", hash, e);
            return;
        }
    };

    // Calculate Etterna rating
    if let Ok(ssr) = calculate_on_demand(&rosu_beatmap, "etterna", 1.0) {
        if let Err(e) = crate::database::query::insert_beatmap_rating(
            db.pool(),
            hash,
            "etterna",
            ssr.overall,
            ssr.stream,
            ssr.jumpstream,
            ssr.handstream,
            ssr.stamina,
            ssr.jackspeed,
            ssr.chordjack,
            ssr.technical,
        )
        .await
        {
            log::warn!("Failed to save Etterna rating for {}: {}", hash, e);
        }
    }

    // Calculate Osu rating
    if let Ok(ssr) = calculate_on_demand(&rosu_beatmap, "osu", 1.0) {
        if let Err(e) = crate::database::query::insert_beatmap_rating(
            db.pool(),
            hash,
            "osu",
            ssr.overall,
            ssr.stream,
            ssr.jumpstream,
            ssr.handstream,
            ssr.stamina,
            ssr.jackspeed,
            ssr.chordjack,
            ssr.technical,
        )
        .await
        {
            log::warn!("Failed to save Osu rating for {}: {}", hash, e);
        }
    }
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

/// Calculates the dominant BPM (the one that lasts the longest).
/// Only considers timing points where is_inherited is false (actual BPM changes, not SV).
fn calculate_dominant_bpm(
    timing_points: &[rhythm_open_exchange::TimingPoint],
    chart_end_time_us: i64,
) -> f64 {
    // Filter to only BPM timing points (not SV changes)
    let bpm_points: Vec<_> = timing_points.iter().filter(|tp| !tp.is_inherited).collect();

    if bpm_points.is_empty() {
        return 0.0;
    }

    // If only one BPM point, return it
    if bpm_points.len() == 1 {
        return bpm_points[0].bpm as f64;
    }

    // Calculate duration for each BPM segment
    let mut bpm_durations: HashMap<u32, i64> = HashMap::new();

    for (i, tp) in bpm_points.iter().enumerate() {
        let start_time = tp.time_us;
        let end_time = if i + 1 < bpm_points.len() {
            bpm_points[i + 1].time_us
        } else {
            chart_end_time_us
        };

        let duration = (end_time - start_time).max(0);
        // Round BPM to integer for grouping (handles floating point variations)
        let bpm_key = (tp.bpm * 10.0) as u32; // Keep 1 decimal precision
        *bpm_durations.entry(bpm_key).or_insert(0) += duration;
    }

    // Find the BPM with the longest total duration
    bpm_durations
        .into_iter()
        .max_by_key(|(_, duration)| *duration)
        .map(|(bpm_key, _)| bpm_key as f64 / 10.0)
        .unwrap_or(0.0)
}
