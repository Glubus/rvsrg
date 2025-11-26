use crate::core::input::actions::{GameAction, KeyAction, UIAction};
use crate::shared::messages::LogicToMain;
use super::{InputContext, InputHandler};

pub struct EditorInputHandler;

impl InputHandler for EditorInputHandler {
    fn handle(&mut self, action: KeyAction, ctx: &mut InputContext) -> bool {
        let Some(engine) = ctx.game_engine else { return false };

        match action {
            KeyAction::Game(GameAction::TogglePause) => {
                // TODO: Implémenter pause
            }
            
            // C'est ICI que les touches de jeu (binds) sont traitées en mode éditeur
            KeyAction::Game(GameAction::Hit(col)) => {
                engine.set_key_held(col, true);
                // On joue le son/feedback pour tester
                engine.process_input(col);
            }
            KeyAction::Game(GameAction::Release(col)) => {
                engine.set_key_held(col, false);
            }

            // Quitter l'éditeur -> Retour Menu
            KeyAction::UI(UIAction::Back) => {
                println!("EDITOR: Back to Menu");
                engine.stop_audio();
                *ctx.game_engine = None;
                ctx.menu_state.in_menu = true;
                ctx.menu_state.in_editor = false;
                let _ = ctx.tx.send(LogicToMain::TransitionToMenu);
            }

            // Toggle Editor -> Retour au Jeu (Play Mode)
            KeyAction::Game(GameAction::ToggleEditor) => {
                println!("EDITOR: Switching to Play Mode");
                ctx.menu_state.in_editor = false;
            }

            _ => return false,
        }
        true
    }
}