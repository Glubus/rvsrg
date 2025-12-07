//! Snapshot creation for GameEngine - get_snapshot

use super::GameEngine;
use crate::models::engine::NoteData;
use crate::shared::snapshot::GameplaySnapshot;

impl GameEngine {
    /// Creates a snapshot of the current game state for rendering.
    pub fn get_snapshot(&self) -> GameplaySnapshot {
        let effective_speed = self.scroll_speed_ms * self.rate;
        let max_visible_time = self.audio_clock + effective_speed;

        // For notes with duration (Hold/Burst), we need to keep them visible
        // until their end time has passed, not just their start time
        let visible_notes: Vec<NoteData> = self
            .chart
            .iter()
            .skip(self.head_index)
            .take_while(|n| n.timestamp_ms <= max_visible_time + 2000.0)
            .filter(|n| {
                if n.hit {
                    return false;
                }
                // For notes with duration, keep visible until end time passes
                if n.note_type.has_duration() {
                    // Keep visible if end hasn't passed yet
                    n.end_time_ms() > self.audio_clock - 100.0
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        GameplaySnapshot {
            audio_time: self.audio_clock,
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
            last_hit_timing: self.last_hit_timing,
            nps: self.current_nps,
            practice_mode: self.practice_mode,
            checkpoints: self.replay_data.checkpoints.clone(),
            map_duration: self.get_map_duration(),
        }
    }
}
