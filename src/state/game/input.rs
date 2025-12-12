//! Input handling for GameEngine - process_hit, process_release, handle_input
//!
//! All times are in microseconds (i64).

use super::GameEngine;
use crate::input::events::GameAction;
use crate::models::engine::NoteType;
use crate::models::stats::Judgement;

impl GameEngine {
    /// Handles a gameplay input action.
    pub fn handle_input(&mut self, action: GameAction) {
        match action {
            GameAction::Hit { column } => {
                if column < self.keys_held.len() {
                    self.keys_held[column] = true;
                }

                // Record the raw PRESS input in the replay (in µs)
                self.replay_data.add_press(self.audio_clock_us, column);

                // Record input timestamp for NPS calculation
                self.input_timestamps.push_back(self.audio_clock_us);
                self.process_hit(column);
            }
            GameAction::Release { column } => {
                if column < self.keys_held.len() {
                    self.keys_held[column] = false;
                }

                // Record the raw RELEASE input in the replay
                self.replay_data.add_release(self.audio_clock_us, column);

                // Check if releasing a hold note
                self.process_release(column);
            }
            GameAction::TogglePause => { /* TODO */ }
            GameAction::PracticeCheckpoint => {
                if self.practice_mode {
                    self.set_checkpoint();
                }
            }
            GameAction::PracticeRetry => {
                if self.practice_mode {
                    self.goto_checkpoint();
                }
            }
            _ => {}
        }
    }

    /// Processes a hit input on the given column.
    ///
    /// Finds the closest unhit note within the hit window and applies
    /// the appropriate judgement based on note type.
    pub(crate) fn process_hit(&mut self, column: usize) {
        // Apply global audio offset to compensate for audio latency
        // Positive offset = notes appear later (audio late), Negative = notes appear earlier (audio early)
        let current_time_us = self.audio_clock_us + self.audio_offset_us;
        let miss_us = self.hit_window.miss_us;
        let mut best_note_idx = None;
        let mut min_diff: i64 = i64::MAX;
        let search_limit = current_time_us + miss_us;

        // Find the best matching note (immutable borrow)
        for (i, note) in self.chart.iter().enumerate().skip(self.head_index) {
            if note.time_us() > search_limit {
                break;
            }
            if note.column() == column && !note.state.hit {
                let diff = (note.time_us() - current_time_us).abs();
                if diff <= miss_us && diff < min_diff {
                    min_diff = diff;
                    best_note_idx = Some(i);
                }
            }
        }

        // Apply judgement based on note type
        if let Some(idx) = best_note_idx {
            let diff_us = self.chart[idx].time_us() - current_time_us;

            if self.chart[idx].is_tap() {
                let (judgement, _) = self.hit_window.judge(diff_us);
                self.chart[idx].state.hit = true;
                self.last_hit_timing_us = Some(diff_us);
                self.last_hit_judgement = Some(judgement);
                self.apply_judgement(judgement);
            } else if self.chart[idx].is_hold() {
                // Start holding - judgement comes when hold is complete
                let (judgement, _) = self.hit_window.judge(diff_us);
                self.chart[idx].state.hold.start_time_us = Some(current_time_us);
                self.chart[idx].state.hold.is_held = true;
                self.last_hit_timing_us = Some(diff_us);
                self.last_hit_judgement = Some(judgement);
                // Don't mark as hit yet - wait for release/completion
            } else if self.chart[idx].is_mine() {
                // Hit a mine = bad!
                self.chart[idx].state.hit = true;
                self.last_hit_timing_us = Some(diff_us);
                self.last_hit_judgement = Some(Judgement::Miss);
                self.apply_judgement(Judgement::Miss);
            } else if self.chart[idx].is_burst() {
                // Increment hit count
                self.chart[idx].state.burst.current_hits += 1;
                if self.chart[idx].state.burst.current_hits
                    >= self.chart[idx].state.burst.required_hits
                {
                    // Burst complete!
                    self.chart[idx].state.hit = true;
                    let (judgement, _) = self.hit_window.judge(diff_us);
                    self.last_hit_timing_us = Some(diff_us);
                    self.last_hit_judgement = Some(judgement);
                    self.apply_judgement(judgement);
                }
            }
        } else {
            self.last_hit_timing_us = None;
            self.last_hit_judgement = Some(Judgement::GhostTap);
            self.apply_judgement(Judgement::GhostTap);
        }
    }

    /// Processes a release input on the given column (for hold notes).
    pub(crate) fn process_release(&mut self, column: usize) {
        // Apply global audio offset for consistency with process_hit
        let current_time_us = self.audio_clock_us + self.audio_offset_us;

        // Find active hold in this column
        for note in self.chart.iter_mut().skip(self.head_index) {
            if note.column() != column || note.state.hit {
                continue;
            }

            if !note.is_hold() || !note.state.hold.is_held {
                continue;
            }

            if let Some(start_us) = note.state.hold.start_time_us {
                let end_time_us = note.end_time_us();
                let hold_duration_us = current_time_us - start_us;
                let expected_duration_us = note.duration_us();

                note.state.hold.is_held = false;
                note.state.hit = true;

                // Calculate how well they held (percentage of required duration)
                let hold_ratio = hold_duration_us as f64 / expected_duration_us as f64;

                let judgement = if hold_ratio >= 0.9 {
                    Judgement::Marv
                } else if hold_ratio >= 0.8 {
                    Judgement::Perfect
                } else if hold_ratio >= 0.6 {
                    Judgement::Great
                } else if hold_ratio >= 0.4 {
                    Judgement::Good
                } else if hold_ratio >= 0.2 {
                    Judgement::Bad
                } else {
                    Judgement::Miss
                };

                self.last_hit_judgement = Some(judgement);
                self.apply_judgement(judgement);
                break;
            }
        }
    }
}
