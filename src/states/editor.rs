use super::{GameState, MenuStateController, StateContext, StateTransition};
use crate::core::input::actions::{GameAction, KeyAction, UIAction}; // Ajout de GameAction
use crate::models::menu::MenuState;
use crate::shared::messages::MainToLogic;
use std::sync::{Arc, Mutex};
use winit::event::WindowEvent;

/// Contrôleur pour l'état "Éditeur".
///
/// Dans la nouvelle architecture (v2), la logique d'édition est gérée directement
/// par l'interface Egui dans le `Renderer`. Ce contrôleur sert principalement
/// de "placeholder" pour maintenir l'état actif dans la boucle principale
/// et gérer la sortie de l'éditeur.
pub struct EditorStateController {
    menu_state: Arc<Mutex<MenuState>>,
}

impl EditorStateController {
    pub fn new(menu_state: Arc<Mutex<MenuState>>) -> Self {
        Self { menu_state }
    }
}

impl GameState for EditorStateController {
    fn on_enter(&mut self, _ctx: &mut StateContext) {
        // On signale au menu state qu'on est en mode éditeur
        if let Ok(mut state) = self.menu_state.lock() {
            state.in_menu = false;
            state.in_editor = true;
        }

        log::info!("EDITOR: Entered state controller");
    }

    fn on_exit(&mut self, _ctx: &mut StateContext) {
        if let Ok(mut state) = self.menu_state.lock() {
            state.in_editor = false;
        }
        log::info!("EDITOR: Exited state controller");
    }

    fn handle_input(
        &mut self,
        _event: &WindowEvent,
        action: Option<KeyAction>,
        ctx: &mut StateContext,
    ) -> StateTransition {
        // La plupart des inputs (clics, drag, texte) sont capturés par Egui dans le Renderer.
        // Ici, on écoute seulement les commandes globales "système" comme Retour/Quitter.

        if let Some(key_action) = action {
            match key_action {
                // Echap ou Retour pour quitter l'éditeur
                KeyAction::UI(UIAction::Back) => {
                    // 1. On nettoie l'interface côté Renderer (optionnel, mais propre)
                    ctx.with_renderer(|_r| {
                        // On pourrait reset des états visuels ici si nécessaire
                    });

                    // 2. On signale au thread Logic de quitter l'état Éditeur
                    // (Le thread Logic va alors renvoyer un StateUpdate avec RenderState::Menu)
                    ctx.send_to_logic(MainToLogic::Input(KeyAction::UI(UIAction::Back)));

                    // 3. Transition immédiate côté Main Thread pour réactivité
                    return StateTransition::Replace(Box::new(MenuStateController::new(
                        Arc::clone(&self.menu_state),
                    )));
                }

                // On gère aussi la bascule directe via la touche d'éditeur (ex: F2 ou E)
                // Si on appuie dessus alors qu'on y est déjà -> on sort
                KeyAction::Game(GameAction::ToggleEditor) => {
                    // On envoie un Back à la logique pour qu'elle sache qu'on sort proprement
                    ctx.send_to_logic(MainToLogic::Input(KeyAction::UI(UIAction::Back)));

                    return StateTransition::Replace(Box::new(MenuStateController::new(
                        Arc::clone(&self.menu_state),
                    )));
                }

                _ => {}
            }
        }

        StateTransition::None
    }

    fn update(&mut self, _ctx: &mut StateContext) -> StateTransition {
        // Rien à faire ici, le Renderer gère l'affichage et l'interaction.
        StateTransition::None
    }
}
