use std::collections::HashMap;
use winit::keyboard::KeyCode;
use super::actions::{GameAction, UIAction, KeyAction};
use crate::models::settings::SettingsState;

#[derive(Clone)]
pub struct KeyBindings {
    game_binds: HashMap<KeyCode, GameAction>,
    ui_binds: HashMap<KeyCode, UIAction>,
    column_maps: HashMap<usize, Vec<KeyCode>>,
}

impl KeyBindings {
    pub fn new() -> Self {
        let mut bindings = Self {
            game_binds: HashMap::new(),
            ui_binds: Self::default_ui_binds(),
            column_maps: Self::default_column_maps(),
        };
        bindings.load_default_game_binds();
        bindings
    }

    pub fn reload_from_settings(&mut self, settings: &SettingsState, current_col_count: usize) {
        let col_str = current_col_count.to_string();
        if let Some(custom_keys_str) = settings.keybinds.get(&col_str) {
            let mut new_keys = Vec::new();
            for key_str in custom_keys_str {
                if let Some(keycode) = parse_keycode(key_str) {
                    new_keys.push(keycode);
                } else {
                    eprintln!("Warning: Unknown keycode in settings: {}", key_str);
                }
            }
            if !new_keys.is_empty() {
                self.column_maps.insert(current_col_count, new_keys);
            }
        }
        self.apply_column_bindings(current_col_count);
    }

    pub fn apply_column_bindings(&mut self, column_count: usize) {
        // On nettoie seulement les actions de HIT/RELEASE
        self.game_binds.retain(|_, action| !matches!(action, GameAction::Hit(_) | GameAction::Release(_)));
        
        if let Some(keys) = self.column_maps.get(&column_count) {
            for (i, &keycode) in keys.iter().enumerate() {
                // Si l'utilisateur bind une touche qui était déjà prise (ex: 'E'), ça l'écrase ici.
                // C'est le comportement voulu pour le jeu, mais c'est pour ça qu'on a ajouté F2 en backup.
                self.game_binds.insert(keycode, GameAction::Hit(i));
            }
        }
    }

    pub fn resolve(&self, key: KeyCode) -> KeyAction {
        if let Some(action) = self.game_binds.get(&key) {
            KeyAction::Game(*action)
        } else if let Some(action) = self.ui_binds.get(&key) {
            KeyAction::UI(*action)
        } else {
            KeyAction::None
        }
    }

    fn load_default_game_binds(&mut self) {
        self.game_binds.insert(KeyCode::Space, GameAction::SkipIntro);
        self.game_binds.insert(KeyCode::F3, GameAction::ChangeSpeed(-50)); 
        self.game_binds.insert(KeyCode::F4, GameAction::ChangeSpeed(50)); 
        self.game_binds.insert(KeyCode::F5, GameAction::Restart);
        self.game_binds.insert(KeyCode::F8, GameAction::Rescan);
        self.game_binds.insert(KeyCode::F11, GameAction::DecreaseNoteSize);
        self.game_binds.insert(KeyCode::F12, GameAction::IncreaseNoteSize);
        
        // KeyE est le raccourci standard
        self.game_binds.insert(KeyCode::KeyE, GameAction::ToggleEditor);
        // F2 est le raccourci de secours (si E est bindé)
        self.game_binds.insert(KeyCode::F2, GameAction::ToggleEditor);
    }

    fn default_ui_binds() -> HashMap<KeyCode, UIAction> {
        let mut map = HashMap::new();
        map.insert(KeyCode::ArrowUp, UIAction::Up);
        map.insert(KeyCode::ArrowDown, UIAction::Down);
        map.insert(KeyCode::ArrowLeft, UIAction::Left);
        map.insert(KeyCode::ArrowRight, UIAction::Right);
        map.insert(KeyCode::Enter, UIAction::Select);
        map.insert(KeyCode::NumpadEnter, UIAction::Select);
        map.insert(KeyCode::Escape, UIAction::Back);
        map.insert(KeyCode::PageUp, UIAction::TabNext);
        map.insert(KeyCode::PageDown, UIAction::TabPrev);
        // F12 est aussi utilisé pour Screenshot dans certains contextes
        // map.insert(KeyCode::F12, UIAction::Screenshot); 
        map
    }

    fn default_column_maps() -> HashMap<usize, Vec<KeyCode>> {
        let mut map = HashMap::new();
        map.insert(4, vec![KeyCode::KeyD, KeyCode::KeyF, KeyCode::KeyJ, KeyCode::KeyK]);
        map.insert(5, vec![KeyCode::KeyD, KeyCode::KeyF, KeyCode::Space, KeyCode::KeyJ, KeyCode::KeyK]);
        map.insert(6, vec![KeyCode::KeyS, KeyCode::KeyD, KeyCode::KeyF, KeyCode::KeyJ, KeyCode::KeyK, KeyCode::KeyL]);
        map.insert(7, vec![KeyCode::KeyS, KeyCode::KeyD, KeyCode::KeyF, KeyCode::Space, KeyCode::KeyJ, KeyCode::KeyK, KeyCode::KeyL]);
        map
    }
}

fn parse_keycode(s: &str) -> Option<KeyCode> {
    match s {
        "KeyA" => Some(KeyCode::KeyA), "KeyB" => Some(KeyCode::KeyB), "KeyC" => Some(KeyCode::KeyC),
        "KeyD" => Some(KeyCode::KeyD), "KeyE" => Some(KeyCode::KeyE), "KeyF" => Some(KeyCode::KeyF),
        "KeyG" => Some(KeyCode::KeyG), "KeyH" => Some(KeyCode::KeyH), "KeyI" => Some(KeyCode::KeyI),
        "KeyJ" => Some(KeyCode::KeyJ), "KeyK" => Some(KeyCode::KeyK), "KeyL" => Some(KeyCode::KeyL),
        "KeyM" => Some(KeyCode::KeyM), "KeyN" => Some(KeyCode::KeyN), "KeyO" => Some(KeyCode::KeyO),
        "KeyP" => Some(KeyCode::KeyP), "KeyQ" => Some(KeyCode::KeyQ), "KeyR" => Some(KeyCode::KeyR),
        "KeyS" => Some(KeyCode::KeyS), "KeyT" => Some(KeyCode::KeyT), "KeyU" => Some(KeyCode::KeyU),
        "KeyV" => Some(KeyCode::KeyV), "KeyW" => Some(KeyCode::KeyW), "KeyX" => Some(KeyCode::KeyX),
        "KeyY" => Some(KeyCode::KeyY), "KeyZ" => Some(KeyCode::KeyZ),
        "Digit0" => Some(KeyCode::Digit0), "Digit1" => Some(KeyCode::Digit1), "Digit2" => Some(KeyCode::Digit2),
        "Digit3" => Some(KeyCode::Digit3), "Digit4" => Some(KeyCode::Digit4), "Digit5" => Some(KeyCode::Digit5),
        "Digit6" => Some(KeyCode::Digit6), "Digit7" => Some(KeyCode::Digit7), "Digit8" => Some(KeyCode::Digit8),
        "Digit9" => Some(KeyCode::Digit9),
        "Space" => Some(KeyCode::Space), "Enter" => Some(KeyCode::Enter), "Escape" => Some(KeyCode::Escape),
        "Backspace" => Some(KeyCode::Backspace), "Tab" => Some(KeyCode::Tab),
        "ShiftLeft" => Some(KeyCode::ShiftLeft), "ShiftRight" => Some(KeyCode::ShiftRight),
        "ControlLeft" => Some(KeyCode::ControlLeft), "ControlRight" => Some(KeyCode::ControlRight),
        "AltLeft" => Some(KeyCode::AltLeft), "AltRight" => Some(KeyCode::AltRight),
        "Semicolon" => Some(KeyCode::Semicolon), "Quote" => Some(KeyCode::Quote),
        "Comma" => Some(KeyCode::Comma), "Period" => Some(KeyCode::Period), "Slash" => Some(KeyCode::Slash),
        "Backslash" => Some(KeyCode::Backslash), "BracketLeft" => Some(KeyCode::BracketLeft), "BracketRight" => Some(KeyCode::BracketRight),
        "Minus" => Some(KeyCode::Minus), "Equal" => Some(KeyCode::Equal),
        _ => None,
    }
}