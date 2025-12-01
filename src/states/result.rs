use super::{GameState, MenuStateController, StateContext, StateTransition};
use crate::core::input::actions::{KeyAction, UIAction};
use crate::models::menu::{GameResultData, MenuState};
use crate::models::replay::{ReplayData, ReplayResult};
use crate::models::stats::HitStats;
use std::sync::{Arc, Mutex};
use winit::event::WindowEvent;

pub struct ResultStateController {
    menu_state: Arc<Mutex<MenuState>>,
}

impl ResultStateController {
    pub fn new(
        menu_state: Arc<Mutex<MenuState>>,
        hit_stats: HitStats,
        replay_data: ReplayData,
        replay_result: ReplayResult,
        score: u32,
        accuracy: f64,
        max_combo: u32,
    ) -> Self {
        if let Ok(mut state) = menu_state.lock() {
            let hash = state.get_selected_beatmap_hash();
            let rate = state.rate;
            let judge_text = "Result".to_string();

            state.last_result = Some(GameResultData {
                hit_stats,
                replay_data,
                replay_result,
                score,
                accuracy,
                max_combo,
                beatmap_hash: hash,
                rate,
                judge_text,
            });
            state.should_close_result = false;
        }

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
}

impl GameState for ResultStateController {
    fn on_enter(&mut self, _ctx: &mut StateContext) {
        self.with_menu_state(|state| {
            state.in_menu = true;
            state.show_result = true;
            state.should_close_result = false;
        });

        _ctx.with_renderer(|renderer| {
            let settings_text = match renderer.resources.settings.hit_window_mode {
                crate::models::settings::HitWindowMode::OsuOD => {
                    format!("OD {:.1}", renderer.resources.settings.hit_window_value)
                }
                crate::models::settings::HitWindowMode::EtternaJudge => {
                    format!(
                        "Judge {}",
                        renderer.resources.settings.hit_window_value as u8
                    )
                }
            };

            if let Ok(mut state) = self.menu_state.lock() {
                if let Some(res) = &mut state.last_result {
                    res.judge_text = settings_text;
                }
            }
        });
    }

    fn on_exit(&mut self, _ctx: &mut StateContext) {
        self.with_menu_state(|state| {
            state.show_result = false;
            state.should_close_result = false;
        });
    }

    fn update(&mut self, _ctx: &mut StateContext) -> StateTransition {
        // Close the screen if the UI requested it (e.g. via button click).
        let should_close = if let Ok(state) = self.menu_state.lock() {
            state.should_close_result
        } else {
            false
        };

        if should_close {
            return StateTransition::Replace(Box::new(MenuStateController::new(Arc::clone(
                &self.menu_state,
            ))));
        }

        StateTransition::None
    }

    fn handle_input(
        &mut self,
        _event: &WindowEvent,
        action: Option<KeyAction>,
        _ctx: &mut StateContext,
    ) -> StateTransition {
        if let Some(KeyAction::UI(ui_action)) = action {
            match ui_action {
                // Either Select (Enter) or Back (Esc) exits this screen.
                UIAction::Select | UIAction::Back => {
                    return StateTransition::Replace(Box::new(MenuStateController::new(
                        Arc::clone(&self.menu_state),
                    )));
                }
                _ => {}
            }
        }
        StateTransition::None
    }
}
