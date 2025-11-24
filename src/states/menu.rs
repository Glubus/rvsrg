use super::{GameState, PlayStateController, StateContext, StateTransition};
use crate::models::menu::MenuState;
use std::sync::{Arc, Mutex};
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

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

    fn trigger_rescan(&self, ctx: &mut StateContext) {
        let _ = ctx.with_db_manager(|db| db.rescan());
    }

    fn load_selected_map(&self, ctx: &mut StateContext) -> bool {
        let map_path = {
            if let Ok(state) = self.menu_state.lock() {
                state.get_selected_beatmap_path()
            } else {
                None
            }
        };

        if let Some(path) = map_path {
            let loaded = ctx
                .with_renderer(|renderer| {
                    if let Ok(mut menu_state) = self.menu_state.lock() {
                        menu_state.in_menu = false;
                    }
                    renderer.load_map(path);
                })
                .is_some();
            return loaded;
        }
        false
    }
}

impl GameState for MenuStateController {
    fn on_enter(&mut self, ctx: &mut StateContext) {
        self.with_menu_state(|state| state.in_menu = true);
        
        // Réinitialiser le flag de chargement des scores pour forcer le rechargement
        ctx.with_renderer(|renderer| {
            renderer.leaderboard_scores_loaded = false;
            renderer.current_leaderboard_hash = None;
        });
    }

    fn handle_input(&mut self, event: &WindowEvent, ctx: &mut StateContext) -> StateTransition {
        if let WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    state,
                    physical_key: PhysicalKey::Code(key_code),
                    repeat,
                    ..
                },
            ..
        } = event
        {
            if *repeat {
                return StateTransition::None;
            }

            let key_code = *key_code;
            if let ElementState::Pressed = state {
                match key_code {
                    KeyCode::Escape => {
                        return StateTransition::Exit;
                    }
                    KeyCode::F8 => {
                        self.trigger_rescan(ctx);
                    }
                    KeyCode::ArrowUp => {
                        self.with_menu_state(|state| state.move_up());
                        // Réinitialiser le leaderboard pour recharger avec la nouvelle map
                        ctx.with_renderer(|renderer| {
                            renderer.leaderboard_scores_loaded = false;
                            renderer.current_leaderboard_hash = None;
                        });
                    }
                    KeyCode::ArrowDown => {
                        self.with_menu_state(|state| state.move_down());
                        // Réinitialiser le leaderboard pour recharger avec la nouvelle map
                        ctx.with_renderer(|renderer| {
                            renderer.leaderboard_scores_loaded = false;
                            renderer.current_leaderboard_hash = None;
                        });
                    }
                    KeyCode::ArrowLeft => {
                        self.with_menu_state(|state| state.previous_difficulty());
                        // Réinitialiser le leaderboard pour recharger avec la nouvelle difficulté
                        ctx.with_renderer(|renderer| {
                            renderer.leaderboard_scores_loaded = false;
                            renderer.current_leaderboard_hash = None;
                        });
                    }
                    KeyCode::ArrowRight => {
                        self.with_menu_state(|state| state.next_difficulty());
                        // Réinitialiser le leaderboard pour recharger avec la nouvelle difficulté
                        ctx.with_renderer(|renderer| {
                            renderer.leaderboard_scores_loaded = false;
                            renderer.current_leaderboard_hash = None;
                        });
                    }
                    KeyCode::Enter | KeyCode::NumpadEnter => {
                        if self.load_selected_map(ctx) {
                            return StateTransition::Replace(Box::new(PlayStateController::new(
                                Arc::clone(&self.menu_state),
                            )));
                        }
                    }
                    KeyCode::PageUp => {
                        self.with_menu_state(|state| {
                            state.increase_rate();
                            println!("Rate: {:.1}x", state.rate);
                        });
                    }
                    KeyCode::PageDown => {
                        self.with_menu_state(|state| {
                            state.decrease_rate();
                            println!("Rate: {:.1}x", state.rate);
                        });
                    }
                    _ => {}
                }
            }
        }
        StateTransition::None
    }
}
