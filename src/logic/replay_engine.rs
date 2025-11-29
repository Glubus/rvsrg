//! Replay engine for frame-perfect replay reproduction.
//! 
//! This engine can replay a saved ReplayData by feeding the inputs back into
//! a GameEngine at the exact same timestamps, ensuring frame-perfect reproduction
//! with identical accuracy and score.

use crate::input::events::GameAction;
use crate::logic::engine::GameEngine;
use crate::models::replay::ReplayData;
use crate::models::stats::HitStats;
use std::path::PathBuf;

/// Engine that can replay a saved replay by feeding inputs into a GameEngine.
pub struct ReplayEngine {
    /// The game engine that processes the replay
    engine: GameEngine,
    /// Replay inputs sorted by timestamp
    inputs: Vec<crate::models::replay::ReplayInput>,
    /// Current index in the inputs array
    input_index: usize,
    /// Whether the replay has finished
    finished: bool,
    /// Collected hits during replay: (note_index, timing_ms)
    collected_hits: Vec<(usize, f64)>,
}

impl ReplayEngine {
    /// Creates a new ReplayEngine from a replay data and beatmap path.
    /// 
    /// # Arguments
    /// * `map_path` - Path to the beatmap file
    /// * `replay_data` - The replay data containing inputs
    /// * `rate` - Playback rate (should match the original play)
    /// * `beatmap_hash` - Optional beatmap hash
    pub fn new(
        map_path: PathBuf,
        replay_data: ReplayData,
        rate: f64,
        beatmap_hash: Option<String>,
    ) -> Self {
        let mut engine = GameEngine::new(map_path, rate, beatmap_hash.clone());
        
        // Clear the replay_data in the engine since we'll be feeding inputs manually
        engine.replay_data = crate::models::replay::ReplayData::new();
        
        // Sort inputs to ensure chronological order
        let mut inputs = replay_data.inputs;
        inputs.sort_by(|a, b| {
            a.timestamp_ms
                .partial_cmp(&b.timestamp_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Self {
            engine,
            inputs,
            input_index: 0,
            finished: false,
            collected_hits: Vec::new(),
        }
    }

    /// Updates the replay engine by one frame.
    /// This processes all inputs that should occur at the current time,
    /// then updates the game engine.
    /// 
    /// # Arguments
    /// * `dt_seconds` - Delta time in seconds since last frame
    pub fn update(&mut self, dt_seconds: f64) {
        if self.finished {
            return;
        }

        // Store which notes were hit before this frame
        let notes_hit_before: std::collections::HashSet<usize> = self
            .engine
            .chart
            .iter()
            .enumerate()
            .filter(|(_, note)| note.hit)
            .map(|(idx, _)| idx)
            .collect();

        // Update the engine first to advance time (this handles misses)
        self.engine.update(dt_seconds);

        // Capture any misses that were processed in update()
        for (note_index, note) in self.engine.chart.iter().enumerate() {
            if note.hit && !notes_hit_before.contains(&note_index) {
                // This note was just marked as hit (likely a miss)
                // For misses, the timing is the miss threshold
                let note_timestamp = note.timestamp_ms;
                let current_time = self.engine.get_time();
                let timing_ms = note_timestamp - current_time;
                self.collected_hits.push((note_index, timing_ms));
            }
        }

        // Process all inputs that should occur at or before the current time
        let current_time = self.engine.get_time();
        
        while self.input_index < self.inputs.len() {
            let input = &self.inputs[self.input_index];
            
            // Only process inputs that are at or before current time
            // We use a small epsilon to account for floating point precision
            if input.timestamp_ms <= current_time + 0.1 {
                // Store state before processing this input
                let notes_hit_before_input: std::collections::HashSet<usize> = self
                    .engine
                    .chart
                    .iter()
                    .enumerate()
                    .filter(|(_, note)| note.hit)
                    .map(|(idx, _)| idx)
                    .collect();
                
                let action = if input.is_press {
                    GameAction::Hit { column: input.column }
                } else {
                    GameAction::Release { column: input.column }
                };
                
                // Feed the input to the engine without recording it to replay_data
                self.engine.handle_input_internal(action, false);
                
                // Check if a new note was hit by this input and collect its timing
                if input.is_press {
                    for (note_index, note) in self.engine.chart.iter().enumerate() {
                        if note.hit && !notes_hit_before_input.contains(&note_index) {
                            // This note was just hit by this input
                            // Capture the exact timing from the engine
                            if let Some(timing_ms) = self.engine.last_hit_timing {
                                self.collected_hits.push((note_index, timing_ms));
                            }
                        }
                    }
                }
                
                self.input_index += 1;
            } else {
                // Input is in the future, wait for next frame
                break;
            }
        }

        // Check if replay is finished
        if self.engine.is_finished() && self.input_index >= self.inputs.len() {
            self.finished = true;
        }
    }

    /// Gets a snapshot of the current game state.
    pub fn get_snapshot(&self) -> crate::shared::snapshot::GameplaySnapshot {
        self.engine.get_snapshot()
    }

    /// Returns true if the replay has finished.
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Gets the current audio time.
    pub fn get_time(&self) -> f64 {
        self.engine.get_time()
    }

    /// Gets the current score.
    pub fn get_score(&self) -> u32 {
        self.engine.score
    }

    /// Gets the current accuracy.
    pub fn get_accuracy(&self) -> f64 {
        self.engine.hit_stats.calculate_accuracy()
    }

    /// Gets the current hit stats.
    pub fn get_hit_stats(&self) -> HitStats {
        self.engine.hit_stats.clone()
    }

    /// Gets the max combo.
    pub fn get_max_combo(&self) -> u32 {
        self.engine.max_combo
    }

    /// Replays the entire replay as fast as possible (without rendering).
    /// This is useful for quickly computing stats without visual playback.
    /// 
    /// # Returns
    /// `(HitStats, accuracy, score, max_combo)`
    pub fn replay_fast(
        map_path: PathBuf,
        replay_data: ReplayData,
        rate: f64,
        beatmap_hash: Option<String>,
        hit_window_mode: crate::models::settings::HitWindowMode,
        hit_window_value: f64,
        scroll_speed: f64,
    ) -> (HitStats, f64, u32, u32) {
        let mut replay_engine = Self::new(map_path, replay_data, rate, beatmap_hash);
        
        // Configure hit window and scroll speed
        replay_engine.engine.update_hit_window(hit_window_mode, hit_window_value);
        replay_engine.engine.scroll_speed_ms = scroll_speed;

        // Fast replay: use a fixed small timestep for accuracy
        // This ensures frame-perfect reproduction
        const DT: f64 = 1.0 / 1000.0; // 1ms timestep for precision
        
        while !replay_engine.is_finished() {
            replay_engine.update(DT);
        }

        let stats = replay_engine.get_hit_stats();
        let accuracy = replay_engine.get_accuracy();
        let score = replay_engine.get_score();
        let max_combo = replay_engine.get_max_combo();

        (stats, accuracy, score, max_combo)
    }

    /// Extracts hit information from the replayed chart.
    /// This should be called after the replay is finished.
    /// Returns the hits collected during the replay with their exact timings.
    /// 
    /// # Returns
    /// Vector of (note_index, timing_ms) tuples for all hit notes
    /// timing_ms is the offset from the note's timestamp (negative = early, positive = late)
    pub fn extract_hits(&self) -> Vec<(usize, f64)> {
        // Return the hits collected during replay
        // These were captured at the exact moment they occurred in the GameEngine
        self.collected_hits.clone()
    }

    /// Replays and extracts hits for visualization.
    /// This is a convenience method that combines replay_fast and extract_hits.
    /// 
    /// # Returns
    /// `(hits, stats, accuracy, score, max_combo)` where hits is Vec<(note_index, timing_ms)>
    pub fn replay_and_extract_hits(
        map_path: PathBuf,
        replay_data: ReplayData,
        rate: f64,
        beatmap_hash: Option<String>,
        hit_window_mode: crate::models::settings::HitWindowMode,
        hit_window_value: f64,
        scroll_speed: f64,
    ) -> (Vec<(usize, f64)>, HitStats, f64, u32, u32) {
        let mut replay_engine = Self::new(map_path, replay_data, rate, beatmap_hash);
        
        // Configure hit window and scroll speed
        replay_engine.engine.update_hit_window(hit_window_mode, hit_window_value);
        replay_engine.engine.scroll_speed_ms = scroll_speed;

        // Fast replay: use a larger timestep for speed, but still accurate enough
        // We can use 5ms steps which is still frame-perfect for hit detection
        const DT: f64 = 5.0 / 1000.0; // 5ms timestep for better performance
        
        while !replay_engine.is_finished() {
            replay_engine.update(DT);
        }

        let hits = replay_engine.extract_hits();
        let stats = replay_engine.get_hit_stats();
        let accuracy = replay_engine.get_accuracy();
        let score = replay_engine.get_score();
        let max_combo = replay_engine.get_max_combo();

        (hits, stats, accuracy, score, max_combo)
    }
}

