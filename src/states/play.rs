use super::{GameState, StateContext, StateTransition};
use crate::core::input::actions::{KeyAction, UIAction};
use crate::models::menu::MenuState;
use crate::shared::messages::MainToLogic; // Hook into logic thread signaling
use std::sync::{Arc, Mutex};
use winit::event::WindowEvent;

pub struct PlayStateController {
    menu_state: Arc<Mutex<MenuState>>,
}

impl PlayStateController {
    pub fn new(menu_state: Arc<Mutex<MenuState>>) -> Self {
        Self { menu_state }
    }
}

impl GameState for PlayStateController {
    fn on_enter(&mut self, _ctx: &mut StateContext) {
        // Mark that we are no longer in the menu so the UI can react.
        if let Ok(mut state) = self.menu_state.lock() {
            state.in_menu = false;
        }
    }

    fn handle_input(
        &mut self,
        _event: &WindowEvent,
        action: Option<KeyAction>,
        ctx: &mut StateContext,
    ) -> StateTransition {
        // Gameplay inputs (Hit, etc.) go straight to the logic thread via App.
        // This layer only cares about forced exits (Escape).

        if let Some(KeyAction::UI(UIAction::Back)) = action {
            // Ask the logic thread to stop the engine.
            ctx.send_to_logic(MainToLogic::Input(KeyAction::UI(UIAction::Back)));

            // Move back to the menu (the visual state will follow the snapshot).
            // App handles transitions when logic sends TransitionToMenu, but we can
            // preemptively request it for faster UI feedback. Logic keeps owning the
            // audio shutdown for now.
        }
        StateTransition::None
    }

    fn update(&mut self, _ctx: &mut StateContext) -> StateTransition {
        // Nothing to do each frame here—the logic thread drives the flow.
        // Transition to the result screen happens via a message from App.
        StateTransition::None
    }
}
