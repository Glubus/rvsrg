//! Core gameplay engine for rhythm game mechanics.
//!
//! The `GameEngine` handles all real-time gameplay logic including:
//! - Note timing and hit detection
//! - Score and combo tracking
//! - Audio synchronization
//! - Practice mode with checkpoints

use crate::input::events::GameAction;
use crate::logic::audio::AudioManager;
use crate::models::engine::{HitWindow, NUM_COLUMNS, NoteData, load_map};
use crate::models::replay::{CHECKPOINT_MIN_INTERVAL_MS, ReplayData};
use crate::models::settings::HitWindowMode;
use crate::models::stats::{HitStats, Judgement};
use crate::shared::snapshot::GameplaySnapshot;
use crate::system::bus::SystemBus;
use std::collections::VecDeque;
use std::path::PathBuf;

/// Offset applied when retrying from a checkpoint (in ms).
/// The player starts 1 second before the checkpoint to prepare.
const CHECKPOINT_RETRY_OFFSET_MS: f64 = 1000.0;

/// Saved state at a checkpoint for restoration.
#[derive(Clone)]
struct CheckpointState {
    timestamp_ms: f64,
    head_index: usize,
    score: u32,
    combo: u32,
    max_combo: u32,
    hit_stats: HitStats,
    notes_passed: u32,
    /// Hit state of each note at checkpoint time.
    note_hit_states: Vec<bool>,
}

/// Main gameplay engine handling note timing, scoring, and audio sync.
pub struct GameEngine {
    /// The chart data (all notes in the map).
    pub chart: Vec<NoteData>,
    /// Index of the first unhit note to check.
    pub head_index: usize,

    /// Current score.
    pub score: u32,
    /// Current combo count.
    pub combo: u32,
    /// Maximum combo achieved.
    pub max_combo: u32,
    /// Hit statistics (marv, perfect, etc.).
    pub hit_stats: HitStats,
    /// Number of notes that have been judged.
    pub notes_passed: u32,

    /// Currently held keys per column.
    pub keys_held: Vec<bool>,
    /// Timing offset of the last hit (for hit error display).
    pub last_hit_timing: Option<f64>,
    /// Judgement of the last hit.
    pub last_hit_judgement: Option<Judgement>,

    /// Audio manager for music playback.
    pub audio_manager: AudioManager,
    /// Smoothed audio clock in milliseconds.
    pub audio_clock: f64,
    /// Whether audio is loaded (false for debug mode).
    has_audio: bool,

    /// Playback rate multiplier.
    pub rate: f64,
    /// Scroll speed in milliseconds (time visible on screen).
    pub scroll_speed_ms: f64,
    /// Hit window configuration.
    pub hit_window: HitWindow,
    /// Hit window mode (osu! OD or Etterna judge).
    pub hit_window_mode: HitWindowMode,
    /// Hit window value (OD value or judge level).
    pub hit_window_value: f64,

    /// Replay data for recording inputs.
    pub replay_data: ReplayData,
    /// Hash of the beatmap being played.
    pub beatmap_hash: Option<String>,
    /// Whether audio has started playing.
    started_audio: bool,

    /// Timestamps of recent inputs for NPS calculation.
    input_timestamps: VecDeque<f64>,
    /// Current notes per second.
    current_nps: f64,

    /// Whether practice mode is enabled.
    pub practice_mode: bool,
    /// Saved state at the last checkpoint.
    checkpoint_state: Option<CheckpointState>,
    /// Timestamp of the last checkpoint (for cooldown enforcement).
    last_checkpoint_time: f64,
}

impl GameEngine {
    /// Pre-roll time before the first note (in ms).
    const PRE_ROLL_MS: f64 = 3000.0;

    /// Creates a new `GameEngine` by loading the map from a file.
    pub fn new(
        bus: &SystemBus,
        map_path: PathBuf,
        rate: f64,
        beatmap_hash: Option<String>,
        hit_window_mode: HitWindowMode,
        hit_window_value: f64,
    ) -> Self {
        let (audio_path, chart) = load_map(map_path);
        Self::from_cached(
            bus,
            chart,
            audio_path,
            rate,
            beatmap_hash,
            hit_window_mode,
            hit_window_value,
        )
    }

    /// Creates a `GameEngine` from pre-loaded chart and audio path.
    ///
    /// Used when the chart is already cached to avoid redundant file I/O.
    pub fn from_cached(
        bus: &SystemBus,
        chart: Vec<NoteData>,
        audio_path: PathBuf,
        rate: f64,
        beatmap_hash: Option<String>,
        hit_window_mode: HitWindowMode,
        hit_window_value: f64,
    ) -> Self {
        let mut audio_manager = AudioManager::new(bus);
        audio_manager.load_music(&audio_path);
        audio_manager.set_speed(rate as f32);

        let hit_window = match hit_window_mode {
            HitWindowMode::OsuOD => HitWindow::from_osu_od(hit_window_value),
            HitWindowMode::EtternaJudge => HitWindow::from_etterna_judge(hit_window_value as u8),
        };

        Self {
            chart,
            head_index: 0,
            score: 0,
            combo: 0,
            max_combo: 0,
            hit_stats: HitStats::new(),
            notes_passed: 0,
            keys_held: vec![false; NUM_COLUMNS],
            last_hit_timing: None,
            last_hit_judgement: None,
            audio_manager,
            audio_clock: -Self::PRE_ROLL_MS,
            has_audio: true,
            replay_data: ReplayData::new(rate, hit_window_mode, hit_window_value),
            beatmap_hash,
            started_audio: false,
            rate,
            scroll_speed_ms: 500.0,
            hit_window,
            hit_window_mode,
            hit_window_value,
            input_timestamps: VecDeque::new(),
            current_nps: 0.0,
            // Practice Mode
            practice_mode: false,
            checkpoint_state: None,
            last_checkpoint_time: f64::NEG_INFINITY,
        }
    }

    /// Creates a `GameEngine` from a debug chart (no audio, for testing).
    pub fn from_debug_chart(
        bus: &SystemBus,
        chart: Vec<NoteData>,
        hit_window_mode: HitWindowMode,
        hit_window_value: f64,
    ) -> Self {
        let audio_manager = AudioManager::new(bus);
        // No audio loaded - we'll run in silent mode

        let hit_window = match hit_window_mode {
            HitWindowMode::OsuOD => HitWindow::from_osu_od(hit_window_value),
            HitWindowMode::EtternaJudge => HitWindow::from_etterna_judge(hit_window_value as u8),
        };

        Self {
            chart,
            head_index: 0,
            score: 0,
            combo: 0,
            max_combo: 0,
            hit_stats: HitStats::new(),
            notes_passed: 0,
            keys_held: vec![false; NUM_COLUMNS],
            last_hit_timing: None,
            last_hit_judgement: None,
            audio_manager,
            audio_clock: -Self::PRE_ROLL_MS,
            has_audio: false, // Debug mode - no audio
            replay_data: ReplayData::new(1.0, hit_window_mode, hit_window_value),
            beatmap_hash: Some("debug_map".to_string()),
            started_audio: true, // No audio, but consider it "started" for gameplay
            rate: 1.0,
            scroll_speed_ms: 500.0,
            hit_window,
            hit_window_mode,
            hit_window_value,
            input_timestamps: VecDeque::new(),
            current_nps: 0.0,
            // Practice Mode
            practice_mode: false,
            checkpoint_state: None,
            last_checkpoint_time: f64::NEG_INFINITY,
        }
    }

    /// Updates the game state for one tick.
    ///
    /// This method:
    /// 1. Advances the audio clock
    /// 2. Synchronizes with the audio device
    /// 3. Processes missed notes
    /// 4. Updates NPS tracking
    pub fn update(&mut self, dt_seconds: f64) {
        // 1. Advance the smoothed clock
        self.audio_clock += dt_seconds * 1000.0 * self.rate;

        if !self.started_audio {
            if self.audio_clock >= 0.0 {
                self.audio_manager.play();
                self.started_audio = true;
            } else {
                return;
            }
        }

        // 2. Re-synchronize with the audio device if drifted
        // Skip sync if audio is seeking (loading in background) or no audio (debug mode)
        if self.has_audio && !self.audio_manager.is_seeking() {
            let raw_audio_time = self.audio_manager.get_position_seconds() * 1000.0;
            let drift = raw_audio_time - self.audio_clock;

            if drift.abs() > 80.0 {
                self.audio_clock = raw_audio_time;
            } else if drift.abs() > 5.0 {
                // Use a much smaller correction factor to avoid "sawtooth" velocity changes
                // causing visual stutter
                self.audio_clock += drift * 0.05;
            }
        }

        let current_time = self.audio_clock;

        // 3. Note state updates and miss handling
        self.update_notes(current_time);

        // 4. Update NPS tracking
        self.update_nps();
    }

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
        let current_time = self.audio_clock;

        // Check cooldown
        if current_time - self.last_checkpoint_time < CHECKPOINT_MIN_INTERVAL_MS {
            log::debug!(
                "PRACTICE: Checkpoint cooldown ({:.1}s remaining)",
                (CHECKPOINT_MIN_INTERVAL_MS - (current_time - self.last_checkpoint_time)) / 1000.0
            );
            return false;
        }

        // Save current state
        let note_hit_states: Vec<bool> = self.chart.iter().map(|n| n.hit).collect();

        self.checkpoint_state = Some(CheckpointState {
            timestamp_ms: current_time,
            head_index: self.head_index,
            score: self.score,
            combo: self.combo,
            max_combo: self.max_combo,
            hit_stats: self.hit_stats.clone(),
            notes_passed: self.notes_passed,
            note_hit_states,
        });

        // Record the checkpoint in replay data
        self.replay_data.add_checkpoint(current_time);
        self.last_checkpoint_time = current_time;

        log::info!("PRACTICE: Checkpoint set at {:.1}s", current_time / 1000.0);
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
        let retry_time = (state.timestamp_ms - CHECKPOINT_RETRY_OFFSET_MS).max(0.0);

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
                self.chart[i].hit = was_hit;
            }
        }

        // Recalculate head_index for notes after retry_time
        for (i, note) in self.chart.iter_mut().enumerate() {
            if note.timestamp_ms >= retry_time
                && i >= state.head_index
                && !state.note_hit_states.get(i).copied().unwrap_or(false)
            {
                note.hit = false;
            }
        }

        self.head_index = self
            .chart
            .iter()
            .position(|n| !n.hit && n.timestamp_ms >= retry_time - self.hit_window.miss_ms)
            .unwrap_or(state.head_index);

        log::info!("PRACTICE: Notes restored, truncating replay");

        // Truncate replay inputs after the checkpoint
        self.replay_data.truncate_inputs_after(state.timestamp_ms);

        log::info!("PRACTICE: Seeking audio to {:.1}s", retry_time / 1000.0);

        // Seek audio (async)
        self.audio_clock = retry_time;
        let seek_seconds = retry_time / 1000.0;
        self.audio_manager.seek(seek_seconds as f32);

        log::info!("PRACTICE: Audio seek initiated");

        // Reset held keys
        self.keys_held.fill(false);
        self.input_timestamps.clear();
        self.current_nps = 0.0;

        log::info!(
            "PRACTICE: Returned to checkpoint at {:.1}s (retry from {:.1}s)",
            state.timestamp_ms / 1000.0,
            retry_time / 1000.0
        );
        true
    }

    /// Returns the timestamps of all checkpoints for UI display.
    pub fn get_checkpoints(&self) -> &[f64] {
        &self.replay_data.checkpoints
    }

    /// Returns the total duration of the map (last note timestamp).
    pub fn get_map_duration(&self) -> f64 {
        self.chart.last().map_or(0.0, |n| n.timestamp_ms)
    }

    /// Processes a hit input on the given column.
    ///
    /// Finds the closest unhit note within the hit window and applies
    /// the appropriate judgement based on note type.
    fn process_hit(&mut self, column: usize) {
        use crate::models::engine::note::NoteType;

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

    /// Updates note states and handles misses for all note types.
    fn update_notes(&mut self, current_time: f64) {
        use crate::models::engine::NoteType;

        let miss_threshold = self.hit_window.miss_ms;
        let mut new_head = self.head_index;

        // Collect judgements to apply (to avoid borrow conflicts)
        let mut judgements: Vec<Judgement> = Vec::new();
        let keys_held = self.keys_held.clone();

        while new_head < self.chart.len() {
            let note = &mut self.chart[new_head];

            // Skip already completed notes
            if note.hit {
                new_head += 1;
                continue;
            }

            let note_timestamp = note.timestamp_ms;
            let note_end_time = note.end_time_ms();

            match &mut note.note_type {
                NoteType::Tap => {
                    if current_time > note_timestamp + miss_threshold {
                        note.hit = true;
                        judgements.push(Judgement::Miss);
                        new_head += 1;
                    } else {
                        break;
                    }
                }

                NoteType::Hold {
                    is_held,
                    start_time,
                    ..
                } => {
                    if *is_held {
                        // Check if hold completed (reached end time)
                        if current_time >= note_end_time {
                            note.hit = true;
                            *is_held = false;
                            judgements.push(Judgement::Marv);
                            new_head += 1;
                        }
                        // Don't advance head_index while holding - note is still active!
                        // Break to stop processing further notes
                        break;
                    } else if start_time.is_none() && current_time > note_timestamp + miss_threshold
                    {
                        // Never started holding - miss
                        note.hit = true;
                        judgements.push(Judgement::Miss);
                        new_head += 1;
                    } else {
                        break;
                    }
                }

                NoteType::Mine => {
                    if current_time > note_timestamp + miss_threshold {
                        note.hit = true;
                        // No judgement - mines that pass are good!
                        new_head += 1;
                    } else {
                        break;
                    }
                }

                NoteType::Burst {
                    duration_ms,
                    required_hits,
                    current_hits,
                } => {
                    if current_time > note_timestamp + *duration_ms {
                        note.hit = true;
                        if *current_hits < *required_hits {
                            let ratio = *current_hits as f64 / *required_hits as f64;
                            let judgement = if ratio >= 0.8 {
                                Judgement::Great
                            } else if ratio >= 0.5 {
                                Judgement::Good
                            } else if ratio > 0.0 {
                                Judgement::Bad
                            } else {
                                Judgement::Miss
                            };
                            judgements.push(judgement);
                        }
                        new_head += 1;
                    } else {
                        break;
                    }
                }
            }
        }

        self.head_index = new_head;

        // Apply collected judgements
        for j in judgements {
            self.apply_judgement(j);
        }
    }

    /// Processes a release input on the given column (for hold notes).
    fn process_release(&mut self, column: usize) {
        use crate::models::engine::note::NoteType;

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

    /// Applies a judgement to the game state (score, combo, stats).
    fn apply_judgement(&mut self, j: Judgement) {
        match j {
            Judgement::Miss => {
                self.hit_stats.miss += 1;
                self.combo = 0;
                self.notes_passed += 1;
            }
            Judgement::GhostTap => {
                self.hit_stats.ghost_tap += 1;
            }
            _ => {
                match j {
                    Judgement::Marv => self.hit_stats.marv += 1,
                    Judgement::Perfect => self.hit_stats.perfect += 1,
                    Judgement::Great => self.hit_stats.great += 1,
                    Judgement::Good => self.hit_stats.good += 1,
                    Judgement::Bad => self.hit_stats.bad += 1,
                    _ => {}
                }
                self.combo += 1;
                self.max_combo = self.max_combo.max(self.combo);
                self.notes_passed += 1;
                self.score += match j {
                    Judgement::Marv | Judgement::Perfect => 300,
                    Judgement::Great => 200,
                    Judgement::Good => 100,
                    Judgement::Bad => 50,
                    _ => 0,
                };
            }
        }
    }

    /// Updates the notes-per-second tracking.
    fn update_nps(&mut self) {
        let current_time = self.audio_clock;
        let window_start = current_time - 1000.0;

        // Remove timestamps older than 1 second
        while let Some(&oldest) = self.input_timestamps.front() {
            if oldest < window_start {
                self.input_timestamps.pop_front();
            } else {
                break;
            }
        }

        // NPS = number of inputs in the last second
        self.current_nps = self.input_timestamps.len() as f64;
    }

    /// Returns the current audio clock time in milliseconds.
    pub fn get_time(&self) -> f64 {
        self.audio_clock
    }

    /// Returns `true` if the map has finished (2 seconds after last note).
    pub fn is_finished(&self) -> bool {
        self.chart
            .last()
            .is_none_or(|n| self.audio_clock > n.timestamp_ms + 2000.0)
    }

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

    /// Updates the hit window configuration.
    pub fn update_hit_window(&mut self, mode: HitWindowMode, value: f64) {
        self.hit_window = match mode {
            HitWindowMode::OsuOD => HitWindow::from_osu_od(value),
            HitWindowMode::EtternaJudge => HitWindow::from_etterna_judge(value as u8),
        };
        self.hit_window_mode = mode;
        self.hit_window_value = value;
    }

    /// Returns a copy of the chart (for replay simulation).
    pub fn get_chart(&self) -> Vec<NoteData> {
        self.chart.clone()
    }
}
