//! Menu state driver for the windowing/state machine layer.

use super::{GameState, PlayStateController, StateContext, StateTransition};
use crate::core::input::actions::{KeyAction, UIAction};
use crate::models::menu::MenuState;
use crate::shared::messages::MainToLogic; // Message channel to the logic thread
use std::sync::{Arc, Mutex};
use winit::event::WindowEvent;

pub struct MenuStateController {
    menu_state: Arc<Mutex<MenuState>>,
}

impl MenuStateController {
    pub fn new(menu_state: Arc<Mutex<MenuState>>) -> Self {
        Self { menu_state }
    }

    fn with_menu_state<F>(&self, mut f: F)
    where
        F: FnMut(&mut MenuState),
    {
        if let Ok(mut state) = self.menu_state.lock() {
            f(&mut state);
        }
    }

    // Request that logic loads the selected map; it handles the heavy lifting.
    fn request_load_map(&self, ctx: &mut StateContext, is_editor: bool) -> bool {
        let map_path = {
            if let Ok(state) = self.menu_state.lock() {
                state.get_selected_beatmap_path()
            } else {
                None
            }
        };

        if let Some(path) = map_path {
            ctx.send_to_logic(MainToLogic::LoadMap { path, is_editor });
            return true; // Assume the load will proceed.
        }
        false
    }
}

impl GameState for MenuStateController {
    fn on_enter(&mut self, ctx: &mut StateContext) {
        self.with_menu_state(|state| {
            state.in_menu = true;
            state.in_editor = false;
        });

        // Local visual reset; snapshots will quickly overwrite it.
        ctx.with_renderer(|renderer| {
            renderer.resources.leaderboard_scores_loaded = false;
            renderer.resources.current_leaderboard_hash = None;
        });
    }

    fn handle_input(
        &mut self,
        _event: &WindowEvent,
        action: Option<KeyAction>,
        ctx: &mut StateContext,
    ) -> StateTransition {
        // Logic already receives inputs via App.
        // This layer only manages local state transitions on the main thread.

        if let Some(KeyAction::UI(action)) = action {
            match action {
                UIAction::Select => {
                    if self.request_load_map(ctx, false) {
                        // Switch to PlayState (which no longer owns heavy logic).
                        return StateTransition::Replace(Box::new(PlayStateController::new(
                            Arc::clone(&self.menu_state),
                        )));
                    }
                }
                _ => {}
            }
        }

        // Logic updates MenuState and the renderer draws it, so keep this controller lean.
        StateTransition::None
    }
}
