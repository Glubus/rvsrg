use crate::core::input::actions::KeyAction;
use crate::shared::snapshot::RenderState;
use std::path::PathBuf;

#[derive(Debug)]
pub enum MainToLogic {
    Input(KeyAction),
    Resize { width: u32, height: u32 },
    SettingsChanged,
    Shutdown,
    LoadMap { path: PathBuf, is_editor: bool },
    EditorCommand(EditorCommand),
    // AJOUT DES VARIANTES MANQUANTES
    TransitionToResult(crate::models::menu::GameResultData),
    TransitionToMenu,
}

#[derive(Debug)]
pub enum LogicToMain {
    StateUpdate(RenderState),
    AudioCommand(AudioCommand),
    ExitApp,
    TransitionToResult(crate::models::menu::GameResultData),
    TransitionToMenu,
    TransitionToEditor, 
    ToggleSettings,     
}

#[derive(Debug)]
pub enum AudioCommand {
    PlaySample(String),
    StopMusic,
}

#[derive(Debug)]
pub enum EditorCommand {
    SaveConfig,
    UpdateConfig(String, f32),
}