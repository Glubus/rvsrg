//! Converts raw key events into high-level `KeyAction`s using bindings.

pub mod actions;
pub mod bindings;

use self::actions::{GameAction, KeyAction, UIAction};
use self::bindings::KeyBindings;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{Key, NamedKey, PhysicalKey};

pub struct InputManager {
    pub bindings: KeyBindings,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            bindings: KeyBindings::new(),
        }
    }

    pub fn update_key_count(&mut self, count: usize) {
        self.bindings.apply_column_bindings(count);
    }

    pub fn process_event(&self, event: &WindowEvent) -> Option<KeyAction> {
        if let WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    state,
                    physical_key: PhysicalKey::Code(keycode),
                    logical_key,
                    repeat: false, // Ignore auto-repeat events.
                    ..
                },
            ..
        } = event
        {
            // Resolve against the configured bindings.
            let action = self.bindings.resolve(*keycode);

            match (*state, action) {
                // Actions Jeu (Hit/Release)
                (ElementState::Pressed, KeyAction::Game(GameAction::Hit(col))) => {
                    Some(KeyAction::Game(GameAction::Hit(col)))
                }
                (ElementState::Released, KeyAction::Game(GameAction::Hit(col))) => {
                    Some(KeyAction::Game(GameAction::Release(col)))
                }

                // Autres Actions Jeu (Pause, etc.) - Pressed only
                (ElementState::Pressed, KeyAction::Game(other)) => Some(KeyAction::Game(other)),

                // Actions UI - Pressed only
                (ElementState::Pressed, KeyAction::UI(ui_action)) => Some(KeyAction::UI(ui_action)),

                // Provide Enter/Escape fallbacks if not mapped.
                (ElementState::Pressed, KeyAction::None) => match logical_key {
                    Key::Named(NamedKey::Enter) => Some(KeyAction::UI(UIAction::Select)),
                    Key::Named(NamedKey::Escape) => Some(KeyAction::UI(UIAction::Back)),
                    _ => None,
                },
                _ => None,
            }
        } else {
            None
        }
    }
}
