//! Practice mode - checkpoints, restore functionality
//!
//! All times are in microseconds (i64).

use super::{CheckpointState, GameEngine};
use crate::models::engine::US_PER_MS;
use crate::models::replay::CHECKPOINT_MIN_INTERVAL_US;

/// Offset applied when retrying from a checkpoint (in µs).
/// The player starts 1 second before the checkpoint to prepare.
pub(crate) const CHECKPOINT_RETRY_OFFSET_US: i64 = 1_000_000; // 1 second

impl GameEngine {
    /// Enables practice mode (called at engine creation).
    pub fn enable_practice_mode(&mut self) {
        self.practice_mode = true;
        self.replay_data.is_practice_mode = true;
        log::info!("PRACTICE MODE: Enabled");
    }

    /// Places a checkpoint at the current position.
    ///
    /// Respects a 15-second cooldown between checkpoints.
    /// Returns `true` if the checkpoint was successfully placed.
    pub fn set_checkpoint(&mut self) -> bool {
        let current_time_us = self.audio_clock_us;

        // Check cooldown
        if current_time_us - self.last_checkpoint_time_us < CHECKPOINT_MIN_INTERVAL_US {
            log::debug!(
                "PRACTICE: Checkpoint cooldown ({:.1}s remaining)",
                (CHECKPOINT_MIN_INTERVAL_US - (current_time_us - self.last_checkpoint_time_us))
                    as f64
                    / 1_000_000.0
            );
            return false;
        }

        // Save current state
        let note_hit_states: Vec<bool> = self.chart.iter().map(|n| n.state.hit).collect();

        self.checkpoint_state = Some(CheckpointState {
            time_us: current_time_us,
            head_index: self.head_index,
            score: self.score,
            combo: self.combo,
            max_combo: self.max_combo,
            hit_stats: self.hit_stats.clone(),
            notes_passed: self.notes_passed,
            note_hit_states,
        });

        // Record the checkpoint in replay data
        self.replay_data.add_checkpoint(current_time_us);
        self.last_checkpoint_time_us = current_time_us;

        log::info!(
            "PRACTICE: Checkpoint set at {:.1}s",
            current_time_us as f64 / 1_000_000.0
        );
        true
    }

    /// Returns to the last checkpoint (minus 1 second for preparation).
    ///
    /// Returns `true` if a checkpoint was available and restored.
    pub fn goto_checkpoint(&mut self) -> bool {
        log::info!("PRACTICE: goto_checkpoint START");

        let Some(state) = self.checkpoint_state.clone() else {
            log::debug!("PRACTICE: No checkpoint to return to");
            return false;
        };

        // Calculate retry time (checkpoint - 1 second)
        let retry_time_us = (state.time_us - CHECKPOINT_RETRY_OFFSET_US).max(0);

        // Restore game state
        self.head_index = state.head_index;
        self.score = state.score;
        self.combo = state.combo;
        self.hit_stats = state.hit_stats;
        self.notes_passed = state.notes_passed;

        log::info!(
            "PRACTICE: Restoring {} notes state",
            state.note_hit_states.len()
        );

        // Restore note states
        for (i, &was_hit) in state.note_hit_states.iter().enumerate() {
            if i < self.chart.len() {
                self.chart[i].state.hit = was_hit;
            }
        }

        let miss_us = self.hit_window.miss_us;

        // Recalculate head_index for notes after retry_time
        for (i, note) in self.chart.iter_mut().enumerate() {
            if note.time_us() >= retry_time_us
                && i >= state.head_index
                && !state.note_hit_states.get(i).copied().unwrap_or(false)
            {
                note.state.hit = false;
            }
        }

        self.head_index = self
            .chart
            .iter()
            .position(|n| !n.state.hit && n.time_us() >= retry_time_us - miss_us)
            .unwrap_or(state.head_index);

        log::info!("PRACTICE: Notes restored, truncating replay");

        // Truncate replay inputs after the checkpoint
        self.replay_data.truncate_inputs_after(state.time_us);

        log::info!(
            "PRACTICE: Seeking audio to {:.1}s",
            retry_time_us as f64 / 1_000_000.0
        );

        // Seek audio (async)
        self.audio_clock_us = retry_time_us;
        let seek_seconds = retry_time_us as f32 / 1_000_000.0;
        self.audio_manager.seek(seek_seconds);

        log::info!("PRACTICE: Audio seek initiated");

        // Reset held keys
        self.keys_held.fill(false);
        self.input_timestamps.clear();
        self.current_nps = 0.0;

        log::info!(
            "PRACTICE: Returned to checkpoint at {:.1}s (retry from {:.1}s)",
            state.time_us as f64 / 1_000_000.0,
            retry_time_us as f64 / 1_000_000.0
        );
        true
    }

    /// Returns the timestamps of all checkpoints for UI display (in µs).
    pub fn get_checkpoints(&self) -> &[i64] {
        &self.replay_data.checkpoints
    }

    /// Returns the total duration of the map in µs (last note timestamp).
    pub fn get_map_duration_us(&self) -> i64 {
        self.chart.last().map_or(0, |n| n.time_us())
    }
}
