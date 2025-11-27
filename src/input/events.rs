use crate::models::search::MenuSearchFilters;
use std::collections::HashMap;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Debug, Clone, Copy)]
pub struct RawInputEvent {
    pub keycode: KeyCode,
    pub state: ElementState,
}

impl RawInputEvent {
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

// Editor mode toggles between resize/move behaviors.
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

#[derive(Debug, Clone, PartialEq)]
pub enum GameAction {
    // Gameplay
    Hit { column: usize },
    Release { column: usize },
    Restart,

    // System / UI
    TogglePause,
    Back,
    Confirm,
    Navigation { x: i32, y: i32 },

    // Mouse interactions
    SetSelection(usize),
    SetDifficulty(usize),

    // Tabs / Settings
    TabNext,
    TabPrev,
    ToggleSettings,
    UpdateVolume(f32),
    ReloadKeybinds,

    // Editor
    ToggleEditor,
    EditorSelect(EditorTarget),
    EditorModify { x: f32, y: f32 },
    EditorSave,

    // DB
    Rescan,
    ApplySearch(MenuSearchFilters),
}

#[derive(Debug, Clone)]
pub enum InputCommand {
    ReloadKeybinds(HashMap<String, Vec<String>>),
}
