use super::{GameState, PlayStateController, StateContext, StateTransition};
use crate::core::input::actions::{KeyAction, UIAction};
use crate::models::menu::MenuState;
use crate::shared::messages::MainToLogic; // NOUVEAU
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

    // NOUVEAU : On envoie juste le chemin, la logique se débrouille pour charger
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
            return true; // On assume que ça va charger
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
        
        // Reset visuel local (optionnel, le snapshot écrasera ça vite)
        ctx.with_renderer(|renderer| {
            renderer.leaderboard_scores_loaded = false;
            renderer.current_leaderboard_hash = None;
        });
    }

    fn handle_input(
        &mut self,
        _event: &WindowEvent,
        action: Option<KeyAction>,
        ctx: &mut StateContext,
    ) -> StateTransition {
        // La logique reçoit déjà les inputs via App.
        // Ici on gère UNIQUEMENT les transitions d'état locales (Main Thread).
        
        if let Some(KeyAction::UI(action)) = action {
            match action {
                UIAction::Select => {
                    if self.request_load_map(ctx, false) {
                        // On passe en PlayState. Note : PlayState n'a plus de logique lourde.
                        return StateTransition::Replace(Box::new(PlayStateController::new(
                            Arc::clone(&self.menu_state),
                        )));
                    }
                }
                _ => {}
            }
        }
        
        // Le thread logique met à jour MenuState, le Renderer l'affiche.
        // On laisse le MenuStateController minimaliste.
        StateTransition::None
    }
}