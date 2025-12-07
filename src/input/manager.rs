use super::events::{EditorTarget, GameAction, RawInputEvent};
use super::keycode::parse_keycode;
use crate::models::engine::constants::NUM_COLUMNS;
use crate::models::settings::SettingsState;
use std::collections::{HashMap, HashSet};
use winit::event::ElementState;
use winit::keyboard::KeyCode;

pub struct InputManager {
    bindings: HashMap<KeyCode, GameAction>,
    ctrl_left: bool,
    ctrl_right: bool,
    suppressed_keys: HashSet<KeyCode>,
}

impl InputManager {
    pub fn new() -> Self {
        let mut manager = Self {
            bindings: HashMap::new(),
            ctrl_left: false,
            ctrl_right: false,
            suppressed_keys: HashSet::new(),
        };
        manager.load_default_bindings();
        let settings = SettingsState::load();
        manager.reload_keybinds(&settings.keybinds);
        manager
    }

    pub fn process(&mut self, event: RawInputEvent) -> Option<GameAction> {
        match event.keycode {
            KeyCode::ControlLeft => {
                self.ctrl_left = event.state == ElementState::Pressed;
                return None;
            }
            KeyCode::ControlRight => {
                self.ctrl_right = event.state == ElementState::Pressed;
                return None;
            }
            _ => {}
        }

        if self.suppressed_keys.contains(&event.keycode) {
            if event.state == ElementState::Released {
                self.suppressed_keys.remove(&event.keycode);
            }
            return None;
        }

        if event.state == ElementState::Pressed
            && event.keycode == KeyCode::KeyO
            && (self.ctrl_left || self.ctrl_right)
        {
            self.suppressed_keys.insert(KeyCode::KeyO);
            return Some(GameAction::ToggleSettings);
        }

        if let Some(base_action) = self.bindings.get(&event.keycode) {
            match (event.state, base_action.clone()) {
                (ElementState::Pressed, GameAction::Hit { column }) => {
                    Some(GameAction::Hit { column })
                }
                (ElementState::Released, GameAction::Hit { column }) => {
                    Some(GameAction::Release { column })
                }
                (ElementState::Pressed, action) => Some(action),
                _ => None,
            }
        } else {
            match (event.state, event.keycode) {
                (ElementState::Pressed, KeyCode::Escape) => Some(GameAction::Back),
                (ElementState::Pressed, KeyCode::Enter) => Some(GameAction::Confirm),
                _ => None,
            }
        }
    }

    pub fn reload_keybinds(&mut self, keybinds: &HashMap<String, Vec<String>>) {
        let key = NUM_COLUMNS.to_string();
        let Some(entries) = keybinds.get(&key) else {
            return;
        };

        let mut parsed = Vec::new();
        for (idx, label) in entries.iter().enumerate() {
            if idx >= NUM_COLUMNS {
                break;
            }
            if let Some(code) = parse_keycode(label) {
                parsed.push((idx, code));
            }
        }

        if parsed.is_empty() {
            return;
        }

        let to_remove: Vec<KeyCode> = self
            .bindings
            .iter()
            .filter_map(|(code, action)| matches!(action, GameAction::Hit { .. }).then_some(*code))
            .collect();
        for code in to_remove {
            self.bindings.remove(&code);
        }

        for (idx, code) in parsed {
            self.bindings.insert(code, GameAction::Hit { column: idx });
        }
    }

    fn load_default_bindings(&mut self) {
        // Gameplay 4K
        self.bindings
            .insert(KeyCode::KeyD, GameAction::Hit { column: 0 });
        self.bindings
            .insert(KeyCode::KeyF, GameAction::Hit { column: 1 });
        self.bindings
            .insert(KeyCode::KeyJ, GameAction::Hit { column: 2 });
        self.bindings
            .insert(KeyCode::KeyK, GameAction::Hit { column: 3 });
        self.bindings.insert(KeyCode::F5, GameAction::Restart);

        // Practice Mode
        self.bindings
            .insert(KeyCode::F3, GameAction::LaunchPractice); // Menu: launch practice
        self.bindings
            .insert(KeyCode::BracketLeft, GameAction::PracticeCheckpoint); // In-game: checkpoint
        self.bindings
            .insert(KeyCode::BracketRight, GameAction::PracticeRetry); // In-game: retry

        // UI navigation (mirrored inside the editor).
        self.bindings
            .insert(KeyCode::ArrowUp, GameAction::Navigation { x: 0, y: -1 });
        self.bindings
            .insert(KeyCode::ArrowDown, GameAction::Navigation { x: 0, y: 1 });
        self.bindings
            .insert(KeyCode::ArrowLeft, GameAction::Navigation { x: -1, y: 0 });
        self.bindings
            .insert(KeyCode::ArrowRight, GameAction::Navigation { x: 1, y: 0 });

        // Tab / settings controls.
        self.bindings.insert(KeyCode::PageUp, GameAction::TabPrev);
        self.bindings.insert(KeyCode::PageDown, GameAction::TabNext);
        self.bindings
            .insert(KeyCode::KeyO, GameAction::ToggleSettings);

        // System / DB
        self.bindings
            .insert(KeyCode::KeyE, GameAction::ToggleEditor); // F2 ou E
        self.bindings.insert(KeyCode::F2, GameAction::ToggleEditor);
        self.bindings.insert(KeyCode::F8, GameAction::Rescan);

        // Editor Selection Shortcuts
        self.bindings
            .insert(KeyCode::KeyW, GameAction::EditorSelect(EditorTarget::Notes));
        self.bindings.insert(
            KeyCode::KeyX,
            GameAction::EditorSelect(EditorTarget::Receptors),
        );
        self.bindings
            .insert(KeyCode::KeyC, GameAction::EditorSelect(EditorTarget::Combo));
        self.bindings
            .insert(KeyCode::KeyV, GameAction::EditorSelect(EditorTarget::Score));
        self.bindings.insert(
            KeyCode::KeyB,
            GameAction::EditorSelect(EditorTarget::Accuracy),
        );
        self.bindings.insert(
            KeyCode::KeyN,
            GameAction::EditorSelect(EditorTarget::Judgement),
        );
        self.bindings.insert(
            KeyCode::KeyK,
            GameAction::EditorSelect(EditorTarget::HitBar),
        );
        self.bindings
            .insert(KeyCode::KeyL, GameAction::EditorSelect(EditorTarget::Lanes));
        self.bindings.insert(KeyCode::KeyS, GameAction::EditorSave);

        // Debug
        self.bindings
            .insert(KeyCode::F10, GameAction::LaunchDebugMap);
    }
}

