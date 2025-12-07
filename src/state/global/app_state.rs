//! Application state enum for the state machine.

use crate::state::editor::EditorState;
use crate::state::{GameEngine, GameResultData, MenuState};

/// High-level application states driven by `GlobalState`.
pub(super) enum AppState {
    /// Song select and menu browsing.
    Menu(MenuState),
    /// Live gameplay.
    Game(GameEngine),
    /// Beatmap/skin editor.
    Editor(EditorState),
    /// Post-game result screen.
    Result(GameResultData),
}
