//! Data structures mirroring the SQLite tables.

#![allow(dead_code)]

use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Beatmapset {
    pub id: i64,
    pub path: String,
    pub image_path: Option<String>,
    pub artist: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Beatmap {
    pub hash: String, // MD5 hash acting as primary key
    pub beatmapset_id: i64,
    pub path: String,
    pub difficulty_name: Option<String>,
    pub note_count: i32,
    pub duration_ms: i32,
    pub nps: f64,
}

/// Lightweight beatmap info for pagination (no ratings loaded).
#[derive(Debug, Clone)]
pub struct BeatmapLight {
    pub hash: String,
    pub difficulty_name: Option<String>,
    pub note_count: i32,
    pub duration_ms: i32,
    pub nps: f64,
    pub path: String,
}

impl From<Beatmap> for BeatmapLight {
    fn from(beatmap: Beatmap) -> Self {
        Self {
            hash: beatmap.hash,
            difficulty_name: beatmap.difficulty_name,
            note_count: beatmap.note_count,
            duration_ms: beatmap.duration_ms,
            nps: beatmap.nps,
            path: beatmap.path,
        }
    }
}

impl From<&Beatmap> for BeatmapLight {
    fn from(beatmap: &Beatmap) -> Self {
        Self {
            hash: beatmap.hash.clone(),
            difficulty_name: beatmap.difficulty_name.clone(),
            note_count: beatmap.note_count,
            duration_ms: beatmap.duration_ms,
            nps: beatmap.nps,
            path: beatmap.path.clone(),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct BeatmapRating {
    pub id: i64,
    pub beatmap_hash: String,
    pub name: String,
    pub overall: f64,
    pub stream: f64,
    pub jumpstream: f64,
    pub handstream: f64,
    pub stamina: f64,
    pub jackspeed: f64,
    pub chordjack: f64,
    pub technical: f64,
}

/// New rating structure with calculator_id and rate support.
#[derive(Debug, Clone, FromRow)]
pub struct BeatmapRatingV2 {
    pub id: i64,
    pub beatmap_hash: String,
    pub calculator_id: String,
    pub rate: f64,
    pub overall: f64,
    pub stream: f64,
    pub jumpstream: f64,
    pub handstream: f64,
    pub stamina: f64,
    pub jackspeed: f64,
    pub chordjack: f64,
    pub technical: f64,
}

impl From<BeatmapRatingV2> for BeatmapRating {
    fn from(v2: BeatmapRatingV2) -> Self {
        Self {
            id: v2.id,
            beatmap_hash: v2.beatmap_hash,
            name: v2.calculator_id,
            overall: v2.overall,
            stream: v2.stream,
            jumpstream: v2.jumpstream,
            handstream: v2.handstream,
            stamina: v2.stamina,
            jackspeed: v2.jackspeed,
            chordjack: v2.chordjack,
            technical: v2.technical,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BeatmapWithRatings {
    pub beatmap: Beatmap,
    pub ratings: Vec<BeatmapRating>,
}

impl BeatmapWithRatings {
    pub fn new(beatmap: Beatmap, ratings: Vec<BeatmapRating>) -> Self {
        Self { beatmap, ratings }
    }
}

/// Beatmapset with lightweight beatmaps (no ratings).
/// Used for pagination to reduce memory usage.
#[derive(Debug, Clone)]
pub struct BeatmapsetLight {
    pub beatmapset: Beatmapset,
    pub beatmaps: Vec<BeatmapLight>,
}

impl BeatmapsetLight {
    pub fn new(beatmapset: Beatmapset, beatmaps: Vec<BeatmapLight>) -> Self {
        Self {
            beatmapset,
            beatmaps,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct Replay {
    pub hash: String,
    pub beatmap_hash: String, // Reference to beatmap.hash
    pub timestamp: i64,       // Unix timestamp recorded at completion
    pub score: i32,
    pub accuracy: f64,
    pub max_combo: i32,
    pub rate: f64,    // Playback rate (1.0 = normal, 1.5 = 1.5x, etc.)
    pub data: String, // JSON or other encoded replay payload
}

/// Pagination info for song select.
#[derive(Debug, Clone, Default)]
pub struct PaginationState {
    pub total_count: usize,
    pub page_size: usize,
    pub current_offset: usize,
}

impl PaginationState {
    pub fn new(total_count: usize, page_size: usize) -> Self {
        Self {
            total_count,
            page_size,
            current_offset: 0,
        }
    }

    /// Returns the range of items currently loaded.
    pub fn loaded_range(&self) -> std::ops::Range<usize> {
        self.current_offset..(self.current_offset + self.page_size).min(self.total_count)
    }

    /// Checks if an index is within the currently loaded range.
    pub fn is_loaded(&self, index: usize) -> bool {
        self.loaded_range().contains(&index)
    }

    /// Calculates a new offset to center around a given index.
    pub fn offset_for_index(&self, index: usize) -> usize {
        if index < self.page_size / 2 {
            0
        } else {
            (index - self.page_size / 2).min(self.total_count.saturating_sub(self.page_size))
        }
    }
}
