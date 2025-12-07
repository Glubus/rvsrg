//! Render snapshots for inter-thread communication.
//!
//! Snapshots are immutable captures of game state sent from the logic thread
//! to the render thread. This decouples game logic from rendering.

use crate::input::events::{EditMode, EditorTarget};
use crate::models::engine::NoteData;
use crate::models::stats::{HitStats, Judgement};
use crate::state::{GameResultData, MenuState};
use std::time::Instant;

/// High-level render state representing the current game mode.
#[derive(Clone, Debug)]
pub enum RenderState {
    /// Initial empty state.
    Empty,
    /// Song select menu.
    Menu(MenuState),
    /// Active gameplay.
    InGame(GameplaySnapshot),
    /// Beatmap editor.
    Editor(EditorSnapshot),
    /// Post-game result screen.
    Result(GameResultData),
}

/// Snapshot of editor state for rendering.
#[derive(Clone, Debug)]
pub struct EditorSnapshot {
    /// Underlying gameplay state.
    pub game: GameplaySnapshot,
    /// Currently selected editor target.
    pub target: Option<EditorTarget>,
    /// Current edit mode (Resize/Move).
    pub mode: EditMode,
    /// Status text for the editor UI.
    pub status_text: String,
    /// Pending modification command: (target, mode, dx, dy).
    pub modification: Option<(EditorTarget, EditMode, f32, f32)>,
    /// Whether save was requested.
    pub save_requested: bool,
}

/// Snapshot of gameplay state for rendering.
#[derive(Clone, Debug)]
pub struct GameplaySnapshot {
    /// Current audio time in milliseconds.
    pub audio_time: f64,
    /// Wall-clock time when snapshot was created (for interpolation).
    pub timestamp: Instant,
    /// Playback rate multiplier.
    pub rate: f64,
    /// Scroll speed in milliseconds.
    pub scroll_speed: f64,

    /// Notes currently visible on screen.
    pub visible_notes: Vec<NoteData>,
    /// Per-column key held state.
    pub keys_held: Vec<bool>,

    /// Current score.
    pub score: u32,
    /// Current accuracy percentage.
    pub accuracy: f64,
    /// Current combo.
    pub combo: u32,
    /// Hit statistics.
    pub hit_stats: HitStats,
    /// Number of remaining notes.
    pub remaining_notes: usize,

    /// Last hit judgement (for flash display).
    pub last_hit_judgement: Option<Judgement>,
    /// Last hit timing offset in ms.
    pub last_hit_timing: Option<f64>,

    /// Current notes per second.
    pub nps: f64,

    /// Whether practice mode is enabled.
    pub practice_mode: bool,
    /// Timestamps of placed checkpoints.
    pub checkpoints: Vec<f64>,
    /// Total map duration (for progress graph).
    pub map_duration: f64,
}
