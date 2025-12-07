//! Input handling for GameEngine - process_hit, process_release, handle_input

use super::GameEngine;
use crate::input::events::GameAction;
use crate::models::engine::note::NoteType;
use crate::models::stats::Judgement;

impl GameEngine {
    /// Handles a gameplay input action.
    pub fn handle_input(&mut self, action: GameAction) {
        match action {
            GameAction::Hit { column } => {
                if column < self.keys_held.len() {
                    self.keys_held[column] = true;
                }

                // Record the raw PRESS input in the replay
                self.replay_data.add_press(self.audio_clock, column);

                // Record input timestamp for NPS calculation
                self.input_timestamps.push_back(self.audio_clock);
                self.process_hit(column);
            }
            GameAction::Release { column } => {
                if column < self.keys_held.len() {
                    self.keys_held[column] = false;
                }

                // Record the raw RELEASE input in the replay
                self.replay_data.add_release(self.audio_clock, column);

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
        let current_time = self.audio_clock;
        let mut best_note_idx = None;
        let mut min_diff = f64::MAX;
        let search_limit = current_time + self.hit_window.miss_ms;

        // Find the best matching note (immutable borrow)
        for (i, note) in self.chart.iter().enumerate().skip(self.head_index) {
            if note.timestamp_ms > search_limit {
                break;
            }
            if note.column == column && !note.hit {
                let diff = (note.timestamp_ms - current_time).abs();
                if diff <= self.hit_window.miss_ms && diff < min_diff {
                    min_diff = diff;
                    best_note_idx = Some(i);
                }
            }
        }

        // Apply judgement based on note type
        if let Some(idx) = best_note_idx {
            let diff = self.chart[idx].timestamp_ms - current_time;

            match &mut self.chart[idx].note_type {
                NoteType::Tap => {
                    let (judgement, _) = self.hit_window.judge(diff);
                    self.chart[idx].hit = true;
                    self.last_hit_timing = Some(diff);
                    self.last_hit_judgement = Some(judgement);
                    self.apply_judgement(judgement);
                }

                NoteType::Hold {
                    start_time,
                    is_held,
                    ..
                } => {
                    // Start holding - judgement comes when hold is complete
                    let (judgement, _) = self.hit_window.judge(diff);
                    *start_time = Some(current_time);
                    *is_held = true;
                    self.last_hit_timing = Some(diff);
                    self.last_hit_judgement = Some(judgement);
                    // Don't mark as hit yet - wait for release/completion
                }

                NoteType::Mine => {
                    // Hit a mine = bad!
                    self.chart[idx].hit = true;
                    self.last_hit_timing = Some(diff);
                    self.last_hit_judgement = Some(Judgement::Miss);
                    self.apply_judgement(Judgement::Miss);
                }

                NoteType::Burst {
                    current_hits,
                    required_hits,
                    ..
                } => {
                    // Increment hit count
                    *current_hits += 1;
                    if *current_hits >= *required_hits {
                        // Burst complete!
                        self.chart[idx].hit = true;
                        let (judgement, _) = self.hit_window.judge(diff);
                        self.last_hit_timing = Some(diff);
                        self.last_hit_judgement = Some(judgement);
                        self.apply_judgement(judgement);
                    }
                }
            }
        } else {
            self.last_hit_timing = None;
            self.last_hit_judgement = Some(Judgement::GhostTap);
            self.apply_judgement(Judgement::GhostTap);
        }
    }

    /// Processes a release input on the given column (for hold notes).
    pub(crate) fn process_release(&mut self, column: usize) {
        let current_time = self.audio_clock;

        // Find active hold in this column
        for note in self.chart.iter_mut().skip(self.head_index) {
            if note.column != column || note.hit {
                continue;
            }

            if let NoteType::Hold {
                duration_ms,
                start_time: Some(start),
                is_held,
                ..
            } = &mut note.note_type
            {
                if !*is_held {
                    continue;
                }

                let end_time = note.timestamp_ms + *duration_ms;
                let hold_duration = current_time - *start;
                let expected_duration = end_time - note.timestamp_ms;

                *is_held = false;
                note.hit = true;

                // Calculate how well they held (percentage of required duration)
                let hold_ratio = hold_duration / expected_duration;

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
