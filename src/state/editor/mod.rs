//! Editor state module.
//!
//! Contains the `EditorState` struct that manages the skin editor state.

pub mod actions;

use crate::input::events::{EditMode, EditorTarget};
use crate::state::GameEngine;

/// State for the skin editor mode.
pub struct EditorState {
    /// The game engine used for preview playback.
    pub engine: GameEngine,
    /// Currently selected editor target (Notes, Receptors, etc.).
    pub target: Option<EditorTarget>,
    /// Current edit mode (Move or Resize).
    pub mode: EditMode,
    /// Accumulated modification buffer for input handling.
    pub modification_buffer: Option<(f32, f32)>,
    /// Whether a save was requested this frame.
    pub save_requested: bool,
}

impl EditorState {
    /// Creates a new editor state with the given engine.
    pub fn new(engine: GameEngine) -> Self {
        Self {
            engine,
            target: None,
            mode: EditMode::Move,
            modification_buffer: None,
            save_requested: false,
        }
    }
}
