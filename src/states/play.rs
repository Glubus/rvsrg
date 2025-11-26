use super::{GameState, StateContext, StateTransition};
use crate::core::input::actions::{KeyAction, UIAction};
use crate::models::menu::MenuState;
use crate::shared::messages::MainToLogic; // NOUVEAU
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
        // On signale juste qu'on est plus dans le menu (pour l'UI)
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
        // Les inputs de jeu (Hit, etc.) sont envoyés directement au Logic Thread par App.
        // On ne gère ici que la sortie forcée (Echap).
        
        if let Some(KeyAction::UI(UIAction::Back)) = action {
            // Demander au moteur de s'arrêter
            ctx.send_to_logic(MainToLogic::Input(KeyAction::UI(UIAction::Back)));
            
            // Revenir au menu (le state visuel suivra via le snapshot)
            // Note : App gère la transition si Logic renvoie TransitionToMenu, 
            // mais on peut forcer ici pour réactivité immédiate de l'UI.
            // Pour l'instant, on laisse Logic gérer le shutdown audio.
        }
        StateTransition::None
    }

    fn update(&mut self, _ctx: &mut StateContext) -> StateTransition {
        // Plus de logique ici !
        // La transition vers ResultScreen se fera via un message reçu dans App
        StateTransition::None
    }
}