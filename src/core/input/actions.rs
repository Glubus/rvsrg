//! Canonical action enums shared between input layers.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameAction {
    Hit(usize),
    Release(usize),
    TogglePause,
    SkipIntro,
    ChangeSpeed(i32),
    ChangeOffset(i32),
    Restart,
    ToggleEditor,
    Rescan,
    DecreaseNoteSize,
    IncreaseNoteSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UIAction {
    Up,
    Down,
    Left,
    Right,
    Select,
    Back,
    TabNext,
    TabPrev,
    Screenshot,
    ToggleFullscreen,
    ToggleSettings,
    // Absolute cursor-driven actions (e.g., mouse selection).
    SetSelection(usize),
    SetDifficulty(usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyAction {
    Game(GameAction),
    UI(UIAction),
    None,
}
