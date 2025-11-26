use crate::core::input::actions::{GameAction, KeyAction, UIAction};
use crate::shared::messages::LogicToMain;
use super::{InputContext, InputHandler};

pub struct GameInputHandler;

impl InputHandler for GameInputHandler {
    fn handle(&mut self, action: KeyAction, ctx: &mut InputContext) -> bool {
        let Some(engine) = ctx.game_engine else { return false };

        match action {
            KeyAction::Game(GameAction::Hit(col)) => {
                engine.set_key_held(col, true);
                engine.process_input(col);
            }
            KeyAction::Game(GameAction::Release(col)) => {
                engine.set_key_held(col, false);
            }
            KeyAction::Game(GameAction::SkipIntro) => engine.skip_intro(),
            KeyAction::Game(GameAction::Restart) => engine.reset_time(),
            KeyAction::Game(GameAction::ChangeSpeed(delta)) => {
                engine.scroll_speed_ms = (engine.scroll_speed_ms + delta as f64).clamp(100.0, 2000.0);
            }
            
            // Retour au menu
            KeyAction::UI(UIAction::Back) => {
                println!("GAME: Back to Menu");
                engine.stop_audio();
                *ctx.game_engine = None;
                ctx.menu_state.in_menu = true;
                ctx.menu_state.in_editor = false;
                ctx.menu_state.show_settings = false; // Sécurité
                let _ = ctx.tx.send(LogicToMain::TransitionToMenu);
            }
            
            _ => return false,
        }
        true
    }
}