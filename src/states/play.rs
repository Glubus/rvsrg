use super::{GameState, MenuStateController, StateContext, StateTransition};
use crate::models::menu::MenuState;
use std::sync::{Arc, Mutex};
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct PlayStateController {
    menu_state: Arc<Mutex<MenuState>>,
}

impl PlayStateController {
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

    fn keycode_to_string(key_code: KeyCode) -> String {
        format!("{:?}", key_code)
    }
}

impl GameState for PlayStateController {
    fn on_enter(&mut self, _ctx: &mut StateContext) {
        self.with_menu_state(|state| state.in_menu = false);
    }

    fn handle_input(&mut self, event: &WindowEvent, ctx: &mut StateContext) -> StateTransition {
        if let WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    state: ElementState::Pressed,
                    physical_key: PhysicalKey::Code(key_code),
                    ..
                },
            ..
        } = event
        {
            match key_code {
                KeyCode::Escape => {
                    ctx.with_renderer(|renderer| renderer.stop_audio());
                    self.with_menu_state(|state| state.in_menu = true);
                    return StateTransition::Replace(Box::new(MenuStateController::new(
                        Arc::clone(&self.menu_state),
                    )));
                }
                KeyCode::F3 => {
                    ctx.with_renderer(|renderer| {
                        renderer.engine.scroll_speed_ms =
                            (renderer.engine.scroll_speed_ms - 50.0).max(100.0);
                        println!("Scroll speed: {:.1} ms", renderer.engine.scroll_speed_ms);
                    });
                }
                KeyCode::F4 => {
                    ctx.with_renderer(|renderer| {
                        renderer.engine.scroll_speed_ms =
                            (renderer.engine.scroll_speed_ms + 50.0).min(2000.0);
                        println!("Scroll speed: {:.1} ms", renderer.engine.scroll_speed_ms);
                    });
                }
                KeyCode::F5 => {
                    ctx.with_renderer(|renderer| {
                        renderer.engine.reset_time();
                        println!("Map restarted from the beginning");
                    });
                }
                KeyCode::F8 => {
                    let _ = ctx.with_db_manager(|db| db.rescan());
                }
                KeyCode::F11 => {
                    ctx.with_renderer(|renderer| renderer.decrease_note_size());
                }
                KeyCode::F12 => {
                    ctx.with_renderer(|renderer| renderer.increase_note_size());
                }
                _ => {
                    let key_name = Self::keycode_to_string(*key_code);
                    ctx.with_renderer(|renderer| {
                        if let Some(column) = renderer.skin.get_column_for_key(&key_name) {
                            if let Some(judgement) = renderer.engine.process_input(column) {
                                println!("Hit column {} ({}): {:?}", column, key_name, judgement);
                            }
                        }
                    });
                }
            }
        }
        StateTransition::None
    }
}
