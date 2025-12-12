//! Data structures mirroring the SQLite tables.

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
    pub hash: String, // blake3 hash from ROX
    pub beatmapset_id: i64,
    pub path: String,
    pub difficulty_name: Option<String>,
    pub note_count: i32,
    pub duration_ms: i32,
    pub nps: f64,
    pub bpm: f64, // Dominant BPM (longest duration in chart)
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

#[derive(Debug, Clone, FromRow)]
pub struct Replay {
    pub hash: String,
    pub beatmap_hash: String, // Reference to beatmap.hash
    pub timestamp: i64,       // Unix timestamp recorded at completion
    pub score: i32,
    pub accuracy: f64,
    pub max_combo: i32,
    pub rate: f64,         // Playback rate (1.0 = normal, 1.5 = 1.5x, etc.)
    pub file_path: String, // Path to Brotli-compressed replay file (data/r/{hash}.r)
}
