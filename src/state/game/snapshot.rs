//! Snapshot creation for GameEngine - get_snapshot
//!
//! All times are in microseconds internally, converted to ms for GameplaySnapshot.

use super::GameEngine;
use crate::models::engine::NoteData;
use crate::models::engine::US_PER_MS;
use crate::shared::snapshot::GameplaySnapshot;

impl GameEngine {
    /// Creates a snapshot of the current game state for rendering.
    pub fn get_snapshot(&self) -> GameplaySnapshot {
        // Apply audio offset for visual synchronization
        let offset_clock_us = self.audio_clock_us + self.audio_offset_us;

        let scroll_speed_us = (self.scroll_speed_ms * US_PER_MS as f64 * self.rate) as i64;
        let max_visible_time_us = offset_clock_us + scroll_speed_us;
        let buffer_us = 2_000_000; // 2 seconds buffer

        // For notes with duration (Hold/Burst), we need to keep them visible
        // until their end time has passed, not just their start time
        let visible_notes: Vec<NoteData> = self
            .chart
            .iter()
            .skip(self.head_index)
            .take_while(|n| n.time_us() <= max_visible_time_us + buffer_us)
            .filter(|n| {
                if n.state.hit {
                    return false;
                }
                // For notes with duration, keep visible until end time passes
                if n.has_duration() {
                    // Keep visible if end hasn't passed yet
                    n.end_time_us() > offset_clock_us - 100_000 // 100ms
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        // Convert checkpoints from i64 µs to f64 ms for compatibility
        let checkpoints_ms: Vec<f64> = self
            .replay_data
            .checkpoints
            .iter()
            .map(|&us| us as f64 / US_PER_MS as f64)
            .collect();

        GameplaySnapshot {
            audio_time: offset_clock_us as f64 / US_PER_MS as f64,
            timestamp: std::time::Instant::now(),
            rate: self.rate,
            scroll_speed: self.scroll_speed_ms,
            visible_notes,
            keys_held: self.keys_held.clone(),
            score: self.score,
            accuracy: self.hit_stats.calculate_accuracy(),
            combo: self.combo,
            hit_stats: self.hit_stats.clone(),
            remaining_notes: self.chart.len().saturating_sub(self.notes_passed as usize),
            last_hit_judgement: self.last_hit_judgement,
            last_hit_timing: self
                .last_hit_timing_us
                .map(|us| us as f64 / US_PER_MS as f64),
            nps: self.current_nps,
            practice_mode: self.practice_mode,
            checkpoints: checkpoints_ms,
            map_duration: self.get_map_duration_us() as f64 / US_PER_MS as f64,
        }
    }
}
