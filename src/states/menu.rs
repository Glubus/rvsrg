use super::{GameState, PlayStateController, EditorStateController, StateContext, StateTransition};
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

    fn load_selected_map(&self, ctx: &mut StateContext, is_editor: bool) -> bool {
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
                        menu_state.in_editor = is_editor; // On définit le mode ici
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
        self.with_menu_state(|state| {
            state.in_menu = true;
            state.in_editor = false;
        });

        ctx.with_renderer(|renderer| {
            renderer.leaderboard_scores_loaded = false;
            renderer.current_leaderboard_hash = None;
        });
    }

    fn handle_input(&mut self, event: &WindowEvent, ctx: &mut StateContext) -> StateTransition {
        if let WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    state: ElementState::Pressed,
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

            match key_code {
                // MODIFICATION : Escape ne quitte plus le jeu
                KeyCode::Escape => {
                    // return StateTransition::Exit; // DÉSACTIVÉ
                }
                KeyCode::F8 => {
                    self.trigger_rescan(ctx);
                }
                KeyCode::ArrowUp => {
                    self.with_menu_state(|state| state.move_up());
                    ctx.with_renderer(|renderer| {
                        renderer.leaderboard_scores_loaded = false;
                        renderer.current_leaderboard_hash = None;
                    });
                }
                KeyCode::ArrowDown => {
                    self.with_menu_state(|state| state.move_down());
                    ctx.with_renderer(|renderer| {
                        renderer.leaderboard_scores_loaded = false;
                        renderer.current_leaderboard_hash = None;
                    });
                }
                KeyCode::ArrowLeft => {
                    self.with_menu_state(|state| state.previous_difficulty());
                    ctx.with_renderer(|renderer| {
                        renderer.leaderboard_scores_loaded = false;
                        renderer.current_leaderboard_hash = None;
                    });
                }
                KeyCode::ArrowRight => {
                    self.with_menu_state(|state| state.next_difficulty());
                    ctx.with_renderer(|renderer| {
                        renderer.leaderboard_scores_loaded = false;
                        renderer.current_leaderboard_hash = None;
                    });
                }
                // PLAY MODE
                KeyCode::Enter | KeyCode::NumpadEnter => {
                    if self.load_selected_map(ctx, false) {
                        return StateTransition::Replace(Box::new(PlayStateController::new(
                            Arc::clone(&self.menu_state),
                        )));
                    }
                }
                // EDITOR MODE (Touche E)
                KeyCode::KeyE => {
                    if self.load_selected_map(ctx, true) {
                        return StateTransition::Replace(Box::new(EditorStateController::new(
                            Arc::clone(&self.menu_state),
                        )));
                    }
                }
                KeyCode::PageUp => {
                    self.with_menu_state(|state| state.increase_rate());
                }
                KeyCode::PageDown => {
                    self.with_menu_state(|state| state.decrease_rate());
                }
                _ => {}
            }
        }
        StateTransition::None
    }
}