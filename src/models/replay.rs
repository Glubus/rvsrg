//! Serializable replay structures and replay simulation engine.
//!
//! This module handles recording and playback of user inputs for replays,
//! as well as deterministic simulation to recalculate scores.
//!
//! All timestamps are in **microseconds (i64)** for precision.

use crate::models::engine::hit_window::HitWindow;
use crate::models::engine::note::{NoteData, US_PER_MS};
use crate::models::settings::HitWindowMode;
use crate::models::stats::{HitStats, Judgement};
use serde::{Deserialize, Serialize};

/// Current replay format version for compatibility.
pub const REPLAY_FORMAT_VERSION: u8 = 4; // Bumped for µs transition

/// A single user input (press or release).
/// Uses microseconds for precision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayInput {
    /// Absolute time in microseconds since map start.
    pub time_us: i64,
    /// Packed data: (column << 1) | is_press
    /// Bit 0: is_press (1 = press, 0 = release)
    /// Bits 1-7: column index
    pub payload: u8,
}

impl ReplayInput {
    /// Unpack column and is_press from payload.
    #[inline]
    pub fn unpack(&self) -> (usize, bool) {
        let is_press = (self.payload & 1) != 0;
        let column = (self.payload >> 1) as usize;
        (column, is_press)
    }
}

/// Minimal replay data containing only raw inputs.
///
/// This design allows replays to be re-simulated with different hit windows
/// to see how scores would change with different judging parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayData {
    /// Format version for future compatibility.
    pub version: u8,
    /// All user inputs in chronological order.
    pub inputs: Vec<ReplayInput>,
    /// Playback rate used during the play.
    pub rate: f64,
    /// Hit window mode used.
    pub hit_window_mode: HitWindowMode,
    /// Hit window value (OD or judge level).
    pub hit_window_value: f64,
    /// Whether practice mode was enabled (scores labeled differently).
    #[serde(default)]
    pub is_practice_mode: bool,
    /// Checkpoints placed by the user (timestamps in µs).
    /// Maximum 1 checkpoint every 15 seconds.
    #[serde(default)]
    pub checkpoints: Vec<i64>,
}

/// Minimum interval between checkpoints (in µs).
pub const CHECKPOINT_MIN_INTERVAL_US: i64 = 15_000_000; // 15 seconds

impl ReplayData {
    /// Creates a new replay data structure.
    pub fn new(rate: f64, hit_window_mode: HitWindowMode, hit_window_value: f64) -> Self {
        Self {
            version: REPLAY_FORMAT_VERSION,
            inputs: Vec::new(),
            rate,
            hit_window_mode,
            hit_window_value,
            is_practice_mode: false,
            checkpoints: Vec::new(),
        }
    }

    /// Creates a new replay data structure in practice mode.
    pub fn new_practice(rate: f64, hit_window_mode: HitWindowMode, hit_window_value: f64) -> Self {
        let mut data = Self::new(rate, hit_window_mode, hit_window_value);
        data.is_practice_mode = true;
        data
    }

    /// Adds a checkpoint if the minimum interval is respected.
    ///
    /// Returns `true` if the checkpoint was successfully added.
    pub fn add_checkpoint(&mut self, time_us: i64) -> bool {
        // Check interval with last checkpoint
        if let Some(&last) = self.checkpoints.last() {
            if time_us - last < CHECKPOINT_MIN_INTERVAL_US {
                return false;
            }
        }
        self.checkpoints.push(time_us);
        true
    }

    /// Returns the last checkpoint timestamp, if any.
    pub fn get_last_checkpoint(&self) -> Option<i64> {
        self.checkpoints.last().copied()
    }

    /// Removes all inputs after the given timestamp.
    ///
    /// Used when retrying from a checkpoint.
    pub fn truncate_inputs_after(&mut self, time_us: i64) {
        self.inputs.retain(|input| input.time_us < time_us);
    }

    /// Adds an input (press or release).
    pub fn add_input(&mut self, time_us: i64, column: usize, is_press: bool) {
        let payload = ((column as u8) << 1) | (is_press as u8);
        self.inputs.push(ReplayInput { time_us, payload });
    }

    /// Adds a key press input.
    #[inline]
    pub fn add_press(&mut self, time_us: i64, column: usize) {
        self.add_input(time_us, column, true);
    }

    /// Adds a key release input.
    #[inline]
    pub fn add_release(&mut self, time_us: i64, column: usize) {
        self.add_input(time_us, column, false);
    }

    /// Serializes to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserializes from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Rebuilds the hit window from saved parameters.
    pub fn build_hit_window(&self) -> HitWindow {
        match self.hit_window_mode {
            HitWindowMode::OsuOD => HitWindow::from_osu_od(self.hit_window_value),
            HitWindowMode::EtternaJudge => {
                HitWindow::from_etterna_judge(self.hit_window_value as u8)
            }
        }
    }
}

impl Default for ReplayData {
    fn default() -> Self {
        Self {
            version: REPLAY_FORMAT_VERSION,
            inputs: Vec::new(),
            rate: 1.0,
            hit_window_mode: HitWindowMode::OsuOD,
            hit_window_value: 5.0,
            is_practice_mode: false,
            checkpoints: Vec::new(),
        }
    }
}

impl ReplayData {
    /// Creates an empty replay data (for fallback/tests).
    pub fn empty() -> Self {
        Self::default()
    }
}

/// Individual hit timing for graphs and analysis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HitTiming {
    /// Index of the hit note.
    pub note_index: usize,
    /// Timing offset in µs (negative = early, positive = late).
    pub timing_us: i64,
    /// Assigned judgement.
    pub judgement: Judgement,
    /// Timestamp of the note in the map (µs).
    pub note_time_us: i64,
}

impl HitTiming {
    /// Timing offset in milliseconds (for display).
    pub fn timing_ms(&self) -> f64 {
        self.timing_us as f64 / US_PER_MS as f64
    }
}

/// Ghost tap (press without a corresponding note).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GhostTap {
    /// Timestamp of the ghost tap (µs).
    pub time_us: i64,
    /// Column index.
    pub column: u8,
}

/// Complete result of a replay simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayResult {
    /// Hit statistics.
    pub hit_stats: HitStats,
    /// Calculated accuracy (0-100).
    pub accuracy: f64,
    /// Total score.
    pub score: u32,
    /// Maximum combo achieved.
    pub max_combo: u32,
    /// Hit timing details for graphs.
    pub hit_timings: Vec<HitTiming>,
    /// List of ghost taps.
    pub ghost_taps: Vec<GhostTap>,
}

impl ReplayResult {
    pub fn new() -> Self {
        Self {
            hit_stats: HitStats::new(),
            accuracy: 0.0,
            score: 0,
            max_combo: 0,
            hit_timings: Vec::new(),
            ghost_taps: Vec::new(),
        }
    }
}

impl Default for ReplayResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Recalculates stats from hit timings of a `ReplayResult`.
///
/// Useful for re-judging an already simulated result with a new hit window
/// WITHOUT access to the original chart (approximation).
pub fn rejudge_hit_timings(hit_timings: &[HitTiming], hit_window: &HitWindow) -> (HitStats, f64) {
    let mut stats = HitStats::new();

    for hit in hit_timings {
        let (judgement, _) = hit_window.judge(hit.timing_us);

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

    let accuracy = stats.calculate_accuracy();
    (stats, accuracy)
}

/// Simulates a replay on a chart with the given hit window.
///
/// This function replays recorded inputs on the map to deterministically
/// recalculate all statistics.
pub fn simulate_replay(
    replay_data: &ReplayData,
    chart: &[NoteData],
    hit_window: &HitWindow,
) -> ReplayResult {
    let mut result = ReplayResult::new();
    let mut combo: u32 = 0;

    // Use hit window miss threshold directly (already in µs)
    let miss_us = hit_window.miss_us;

    // Track hit notes (index -> hit)
    let mut note_hit: Vec<bool> = vec![false; chart.len()];

    // Head index to optimize search
    let mut head_index: usize = 0;

    for input in &replay_data.inputs {
        let (input_column, is_press) = input.unpack();
        let input_time_us = input.time_us;

        // Before processing this input, check for missed notes
        while head_index < chart.len() {
            if note_hit[head_index] {
                head_index += 1;
                continue;
            }

            let note = &chart[head_index];
            let miss_deadline = note.time_us() + miss_us;

            if input_time_us > miss_deadline {
                // Miss!
                note_hit[head_index] = true;
                result.hit_stats.miss += 1;
                combo = 0;

                result.hit_timings.push(HitTiming {
                    note_index: head_index,
                    timing_us: miss_us,
                    judgement: Judgement::Miss,
                    note_time_us: note.time_us(),
                });

                head_index += 1;
            } else {
                break;
            }
        }

        // Only process presses (releases are ignored for scoring)
        if !is_press {
            continue;
        }

        // Find the best note to hit in this column
        let current_time = input_time_us;
        let mut best_match: Option<(usize, i64)> = None;
        let search_limit = current_time + miss_us;

        for i in head_index..chart.len() {
            let note = &chart[i];

            if note.time_us() > search_limit {
                break;
            }

            if note.column() == input_column && !note_hit[i] {
                let diff = (note.time_us() - current_time).abs();
                if diff <= miss_us && best_match.is_none_or(|(_, best_diff)| diff < best_diff) {
                    best_match = Some((i, diff));
                }
            }
        }

        if let Some((idx, _)) = best_match {
            let note = &chart[idx];
            let diff_us = note.time_us() - current_time; // Signed: negative = early
            let (judgement, _) = hit_window.judge(diff_us);

            note_hit[idx] = true;

            // Apply judgement
            match judgement {
                Judgement::Miss => {
                    result.hit_stats.miss += 1;
                    combo = 0;
                }
                Judgement::GhostTap => {
                    result.hit_stats.ghost_tap += 1;
                }
                _ => {
                    match judgement {
                        Judgement::Marv => result.hit_stats.marv += 1,
                        Judgement::Perfect => result.hit_stats.perfect += 1,
                        Judgement::Great => result.hit_stats.great += 1,
                        Judgement::Good => result.hit_stats.good += 1,
                        Judgement::Bad => result.hit_stats.bad += 1,
                        _ => {}
                    }
                    combo += 1;
                    result.max_combo = result.max_combo.max(combo);
                    result.score += match judgement {
                        Judgement::Marv | Judgement::Perfect => 300,
                        Judgement::Great => 200,
                        Judgement::Good => 100,
                        Judgement::Bad => 50,
                        _ => 0,
                    };
                }
            }

            result.hit_timings.push(HitTiming {
                note_index: idx,
                timing_us: diff_us,
                judgement,
                note_time_us: note.time_us(),
            });
        } else {
            // Ghost tap - no corresponding note
            result.hit_stats.ghost_tap += 1;
            result.ghost_taps.push(GhostTap {
                time_us: input_time_us,
                column: input_column as u8,
            });
        }
    }

    // After all inputs, check remaining unhit notes (final misses)
    for (idx, note) in chart.iter().enumerate() {
        if !note_hit[idx] {
            result.hit_stats.miss += 1;
            result.hit_timings.push(HitTiming {
                note_index: idx,
                timing_us: miss_us,
                judgement: Judgement::Miss,
                note_time_us: note.time_us(),
            });
        }
    }

    // Calculate final accuracy
    result.accuracy = result.hit_stats.calculate_accuracy();

    result
}

/// Re-judges a replay with a new hit window.
///
/// Useful for seeing how the score would change with different parameters.
pub fn rejudge_replay(
    replay_data: &ReplayData,
    chart: &[NoteData],
    new_hit_window: &HitWindow,
) -> ReplayResult {
    simulate_replay(replay_data, chart, new_hit_window)
}
