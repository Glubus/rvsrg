//! Input event types and game actions.
//!
//! This module defines all input-related structures used for communication
//! between the window, input thread, and game logic.

use crate::models::search::MenuSearchFilters;
use std::collections::HashMap;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

/// A raw keyboard input event from the window.
#[derive(Debug, Clone, Copy)]
pub struct RawInputEvent {
    /// The physical key code.
    pub keycode: KeyCode,
    /// Whether the key was pressed or released.
    pub state: ElementState,
}

impl RawInputEvent {
    /// Creates a `RawInputEvent` from a winit window event.
    ///
    /// Returns `None` if the event is not a keyboard input or is a key repeat.
    pub fn from_winit(event: &WindowEvent) -> Option<Self> {
        if let WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    physical_key: PhysicalKey::Code(keycode),
                    state,
                    repeat: false,
                    ..
                },
            ..
        } = event
        {
            Some(Self {
                keycode: *keycode,
                state: *state,
            })
        } else {
            None
        }
    }
}

/// Target element for editor modifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTarget {
    Notes,
    Receptors,
    Combo,
    Score,
    Accuracy,
    Judgement,
    Counter,
    HitBar,
    Lanes,
}

/// Editor mode: resize or move elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMode {
    Resize,
    Move,
}

impl std::fmt::Display for EditMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditMode::Resize => write!(f, "RESIZE"),
            EditMode::Move => write!(f, "MOVE"),
        }
    }
}

/// High-level game actions processed by the logic thread.
#[derive(Debug, Clone, PartialEq)]
pub enum GameAction {
    // Gameplay
    /// Key press on a column.
    Hit { column: usize },
    /// Key release on a column.
    Release { column: usize },
    /// Restart the current map.
    Restart,

    // Practice Mode (in-game)
    /// Place a checkpoint (max 1 every 15 seconds).
    PracticeCheckpoint,
    /// Return to the last checkpoint (minus 1 second).
    PracticeRetry,

    // Menu
    /// Launch the game in practice mode (F3).
    LaunchPractice,

    // System / UI
    /// Toggle pause state.
    TogglePause,
    /// Go back (escape).
    Back,
    /// Confirm selection (enter).
    Confirm,
    /// Navigation input (arrow keys).
    Navigation { x: i32, y: i32 },

    // Mouse interactions
    /// Set the selected beatmapset by index.
    SetSelection(usize),
    /// Set the selected difficulty by index.
    SetDifficulty(usize),

    // Tabs / Settings
    /// Switch to next tab.
    TabNext,
    /// Switch to previous tab.
    TabPrev,
    /// Toggle settings panel.
    ToggleSettings,
    /// Update master volume.
    UpdateVolume(f32),
    /// Reload keybinds from disk.
    ReloadKeybinds,

    // Editor
    /// Toggle editor mode.
    ToggleEditor,
    /// Select an editor target element.
    EditorSelect(EditorTarget),
    /// Modify the selected element position/size.
    EditorModify { x: f32, y: f32 },
    /// Save editor changes.
    EditorSave,

    // Database
    /// Trigger a full beatmap rescan.
    Rescan,
    /// Apply search filters.
    ApplySearch(MenuSearchFilters),

    // Difficulty
    /// Set the active difficulty calculator.
    SetCalculator(String),
    /// Update the hit window (live re-judging).
    UpdateHitWindow {
        mode: crate::models::settings::HitWindowMode,
        value: f64,
    },

    // Result screen
    /// Navigate to result screen with data.
    SetResult(crate::models::menu::GameResultData),

    // Debug
    /// Launch a debug map with all note types for testing.
    LaunchDebugMap,
}

/// Commands sent to the input thread.
#[derive(Debug, Clone)]
pub enum InputCommand {
    /// Reload keybind configuration.
    ReloadKeybinds(HashMap<String, Vec<String>>),
}
