//! Core gameplay engine for rhythm game mechanics.
//!
//! The `GameEngine` handles all real-time gameplay logic including:
//! - Note timing and hit detection
//! - Score and combo tracking
//! - Audio synchronization
//! - Practice mode with checkpoints

mod input;
mod notes;
mod practice;
mod snapshot;

pub mod actions;

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
pub(crate) const CHECKPOINT_RETRY_OFFSET_MS: f64 = 1000.0;

/// Saved state at a checkpoint for restoration.
#[derive(Clone)]
pub(crate) struct CheckpointState {
    pub timestamp_ms: f64,
    pub head_index: usize,
    pub score: u32,
    pub combo: u32,
    pub max_combo: u32,
    pub hit_stats: HitStats,
    pub notes_passed: u32,
    /// Hit state of each note at checkpoint time.
    pub note_hit_states: Vec<bool>,
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
    pub(crate) has_audio: bool,

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
    pub(crate) started_audio: bool,

    /// Timestamps of recent inputs for NPS calculation.
    pub(crate) input_timestamps: VecDeque<f64>,
    /// Current notes per second.
    pub(crate) current_nps: f64,

    /// Whether practice mode is enabled.
    pub practice_mode: bool,
    /// Saved state at the last checkpoint.
    pub(crate) checkpoint_state: Option<CheckpointState>,
    /// Timestamp of the last checkpoint (for cooldown enforcement).
    pub(crate) last_checkpoint_time: f64,
}

impl GameEngine {
    /// Pre-roll time before the first note (in ms).
    const PRE_ROLL_MS: f64 = 3000.0;

    /// Creates a new `GameEngine` by loading the map from a file.
    /// Returns `None` if the map cannot be loaded.
    pub fn new(
        bus: &SystemBus,
        map_path: PathBuf,
        rate: f64,
        beatmap_hash: Option<String>,
        hit_window_mode: HitWindowMode,
        hit_window_value: f64,
    ) -> Option<Self> {
        match load_map(map_path.clone()) {
            Ok((audio_path, chart)) => Some(Self::from_cached(
                bus,
                chart,
                audio_path,
                rate,
                beatmap_hash,
                hit_window_mode,
                hit_window_value,
            )),
            Err(e) => {
                log::error!("ENGINE: Failed to load map {:?}: {}", map_path, e);
                None
            }
        }
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
