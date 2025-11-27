//! Serializable replay structures and helpers.
use serde::{Deserialize, Serialize};

/// Represents a hit on a specific note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayHit {
    pub note_index: usize, // Sequential note index
    pub timing_ms: f64,    // Offset in ms (negative means early)
}

/// Represents a raw key press (no specific note).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayKeyPress {
    pub timestamp_ms: f64, // Absolute time in ms since song start
    pub column: usize,     // Column index
}

/// Full replay payload including hits and key presses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayData {
    pub hits: Vec<ReplayHit>,             // Hits in chronological order
    pub key_presses: Vec<ReplayKeyPress>, // Raw key presses
    pub hit_stats: Option<crate::models::stats::HitStats>, // Cached stats for fast display
}

impl ReplayData {
    pub fn new() -> Self {
        Self {
            hits: Vec::new(),
            key_presses: Vec::new(),
            hit_stats: None,
        }
    }

    pub fn with_hit_stats(mut self, stats: crate::models::stats::HitStats) -> Self {
        self.hit_stats = Some(stats);
        self
    }

    /// Appends a hit to the replay.
    pub fn add_hit(&mut self, note_index: usize, timing_ms: f64) {
        self.hits.push(ReplayHit {
            note_index,
            timing_ms,
        });
    }

    /// Appends a raw key press entry.
    pub fn add_key_press(&mut self, timestamp_ms: f64, column: usize) {
        self.key_presses.push(ReplayKeyPress {
            timestamp_ms,
            column,
        });
    }

    /// Serializes to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserializes from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl Default for ReplayData {
    fn default() -> Self {
        Self::new()
    }
}

/// Recomputes hit stats and accuracy using the provided hit window.
///
/// # Arguments
/// * `replay_data` - Replay payload containing hit timings
/// * `total_notes` - Total number of notes in the chart
/// * `hit_window` - Hit window configuration used for rejudging
///
/// # Returns
/// `(HitStats, accuracy_percentage)`
pub fn recalculate_accuracy_with_hit_window(
    replay_data: &ReplayData,
    total_notes: usize,
    hit_window: &crate::models::engine::hit_window::HitWindow,
) -> (crate::models::stats::HitStats, f64) {
    use crate::models::stats::{HitStats, Judgement};

    let mut stats = HitStats::new();

    // Track which notes were hit.
    let mut hit_notes = std::collections::HashSet::new();
    for hit in &replay_data.hits {
        hit_notes.insert(hit.note_index);

        // Rejudge using the new hit window (timing_ms is already rate-normalized).
        let (judgement, _) = hit_window.judge(hit.timing_ms);

        match judgement {
            Judgement::Marv => stats.marv += 1,
            Judgement::Perfect => stats.perfect += 1,
            Judgement::Great => stats.great += 1,
            Judgement::Good => stats.good += 1,
            Judgement::Bad => stats.bad += 1,
            Judgement::Miss => stats.miss += 1,
            Judgement::GhostTap => stats.ghost_tap += 1,
        }
    }

    // Notes never hit count as misses.
    for note_index in 0..total_notes {
        if !hit_notes.contains(&note_index) {
            stats.miss += 1;
        }
    }

    let accuracy = stats.calculate_accuracy();
    (stats, accuracy)
}
