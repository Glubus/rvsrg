use crate::core::input::actions::{GameAction, KeyAction, UIAction};
use crate::models::engine::GameEngine;
use crate::shared::messages::LogicToMain;
use super::{InputContext, InputHandler};

pub struct MenuInputHandler;

impl InputHandler for MenuInputHandler {
    fn handle(&mut self, action: KeyAction, ctx: &mut InputContext) -> bool {
        // 1. Si les Settings sont ouverts, on intercepte TOUT
        if ctx.menu_state.show_settings {
            match action {
                // Seules ces touches permettent de fermer les settings
                KeyAction::UI(UIAction::ToggleSettings) | KeyAction::UI(UIAction::Back) => {
                    println!("MENU: Closing Settings");
                    ctx.menu_state.show_settings = false;
                    return true;
                }
                // Tout le reste est consommé (ignoré) pour ne pas bouger le menu derrière
                _ => return true, 
            }
        }

        // 2. Gestion standard du Menu (quand settings fermés)
        match action {
            KeyAction::UI(UIAction::Up) => ctx.menu_state.move_up(),
            KeyAction::UI(UIAction::Down) => ctx.menu_state.move_down(),
            KeyAction::UI(UIAction::Left) => ctx.menu_state.previous_difficulty(),
            KeyAction::UI(UIAction::Right) => ctx.menu_state.next_difficulty(),
            KeyAction::UI(UIAction::TabNext) => ctx.menu_state.increase_rate(),
            KeyAction::UI(UIAction::TabPrev) => ctx.menu_state.decrease_rate(),
            
            // NOUVEAU : Gestion des clics absolus
            KeyAction::UI(UIAction::SetSelection(idx)) => {
                if idx < ctx.menu_state.beatmapsets.len() {
                    ctx.menu_state.selected_index = idx;
                    ctx.menu_state.selected_difficulty_index = 0;
                    
                    // Mise à jour du scroll visible pour suivre la sélection
                    if ctx.menu_state.selected_index >= ctx.menu_state.end_index {
                        ctx.menu_state.end_index = (ctx.menu_state.selected_index + 1).min(ctx.menu_state.beatmapsets.len());
                        ctx.menu_state.start_index = ctx.menu_state.end_index.saturating_sub(ctx.menu_state.visible_count);
                    } else if ctx.menu_state.selected_index < ctx.menu_state.start_index {
                        ctx.menu_state.start_index = ctx.menu_state.selected_index;
                        ctx.menu_state.end_index = (ctx.menu_state.start_index + ctx.menu_state.visible_count).min(ctx.menu_state.beatmapsets.len());
                    }
                }
            }
            KeyAction::UI(UIAction::SetDifficulty(idx)) => {
                if let Some((_, maps)) = ctx.menu_state.get_selected_beatmapset() {
                    if idx < maps.len() {
                        ctx.menu_state.selected_difficulty_index = idx;
                    }
                }
            }
            
            // Lancement du jeu
            KeyAction::UI(UIAction::Select) => {
                if let Some(path) = ctx.menu_state.get_selected_beatmap_path() {
                    println!("MENU: Loading Map {:?}", path);
                    let rate = ctx.menu_state.rate;
                    let mut engine = GameEngine::from_map(path, rate);
                    // Config moteur par défaut
                    engine.scroll_speed_ms = 500.0;
                    engine.update_hit_window(crate::models::settings::HitWindowMode::OsuOD, 5.0);
                    
                    *ctx.game_engine = Some(engine);
                    ctx.menu_state.in_menu = false;
                    ctx.menu_state.in_editor = false;
                }
            }

            KeyAction::Game(GameAction::Rescan) => {
                ctx.db_manager.rescan();
            }

            // Ouverture Settings
            KeyAction::UI(UIAction::ToggleSettings) => {
                println!("MENU: Opening Settings");
                ctx.menu_state.show_settings = true;
                let _ = ctx.tx.send(LogicToMain::ToggleSettings);
            }

            // Ouverture Éditeur sur la map sélectionnée
            KeyAction::Game(GameAction::ToggleEditor) => {
                if let Some(path) = ctx.menu_state.get_selected_beatmap_path() {
                    println!("MENU: Opening Editor");
                    let rate = 1.0; 
                    let mut engine = GameEngine::from_map(path, rate);
                    engine.scroll_speed_ms = 500.0;
                    
                    *ctx.game_engine = Some(engine);
                    ctx.menu_state.in_menu = false;
                    ctx.menu_state.in_editor = true;
                    let _ = ctx.tx.send(LogicToMain::TransitionToEditor);
                }
            }

            _ => return false,
        }
        true
    }
}