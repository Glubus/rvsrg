use super::{GameState, MenuStateController, ResultStateController, StateContext, StateTransition};
use crate::models::menu::MenuState;
use crate::models::engine::NUM_COLUMNS; // Import nécessaire pour les keybinds
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
                        let new_speed = (renderer.engine.scroll_speed_ms - 50.0).max(100.0);
                        renderer.engine.scroll_speed_ms = new_speed;
                        // Synchroniser avec les settings pour la persistance/UI
                        renderer.settings.scroll_speed = new_speed;
                    });
                }
                KeyCode::F4 => {
                    ctx.with_renderer(|renderer| {
                        let new_speed = (renderer.engine.scroll_speed_ms + 50.0).min(2000.0);
                        renderer.engine.scroll_speed_ms = new_speed;
                        // Synchroniser avec les settings pour la persistance/UI
                        renderer.settings.scroll_speed = new_speed;
                    });
                }
                KeyCode::F5 => {
                    ctx.with_renderer(|renderer| {
                        renderer.engine.reset_time();
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
                        // Récupérer les binds pour le nombre de colonnes actuel
                        // On clone pour éviter les soucis d'emprunt avec renderer.engine.process_input
                        let current_binds = renderer.settings.keybinds
                            .get(NUM_COLUMNS.to_string().as_str())
                            .cloned()
                            .unwrap_or_default();

                        // 1. Chercher par nom de touche exact (ex: "KeyD")
                        let mut column = current_binds.iter().position(|k| k == &key_name);

                        // 2. Si pas trouvé, essayer le mapping AZERTY / Caractères spéciaux
                        if column.is_none() {
                            let mut char_keys = Vec::new();
                            match *key_code {
                                KeyCode::Digit0 => char_keys.push("à"),
                                KeyCode::Digit1 => char_keys.push("&"),
                                KeyCode::Digit2 => char_keys.push("é"),
                                KeyCode::Digit3 => char_keys.push("\""),
                                KeyCode::Digit4 => char_keys.push("'"),
                                KeyCode::Digit5 => char_keys.push("("),
                                KeyCode::Digit6 => char_keys.push("-"),
                                KeyCode::Digit7 => char_keys.push("è"),
                                KeyCode::Digit8 => char_keys.push("_"),
                                KeyCode::Digit9 => char_keys.push("ç"),
                                KeyCode::KeyQ => char_keys.push("a"),
                                KeyCode::KeyW => char_keys.push("z"),
                                KeyCode::KeyA => char_keys.push("q"),
                                KeyCode::KeyM => char_keys.push("?"),
                                KeyCode::Comma => char_keys.push(";"),
                                KeyCode::Period => char_keys.push(":"),
                                KeyCode::Semicolon => char_keys.push("m"),
                                KeyCode::Slash => char_keys.push("!"),
                                KeyCode::Backquote => char_keys.push("²"),
                                _ => {}
                            }

                            // Chercher si un des caractères correspond à un bind
                            for ch in char_keys {
                                // On cherche insensible à la casse pour être sympa
                                if let Some(col) = current_binds.iter().position(|k| k.eq_ignore_ascii_case(ch)) {
                                    column = Some(col);
                                    break;
                                }
                            }
                        }

                        if let Some(col) = column {
                            renderer.engine.process_input(col);
                        }
                    });
                }
            }
        }
        StateTransition::None
    }

    fn update(&mut self, ctx: &mut StateContext) -> StateTransition {
        // Vérifier si la partie est terminée
        let game_finished = ctx
            .with_renderer(|renderer| renderer.engine.is_game_finished())
            .unwrap_or(false);

        if game_finished {
            // Sauvegarder le replay dans la base de données
            let _ = ctx.with_renderer(|renderer| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let db_path = std::path::PathBuf::from("main.db");

                if let Ok(db) = rt.block_on(crate::database::connection::Database::new(&db_path)) {
                    if let Err(e) = rt.block_on(renderer.engine.save_replay(&db)) {
                        eprintln!("Erreur lors de la sauvegarde du replay: {}", e);
                    }
                }
            });

            // Récupérer les stats et le replay avant de passer à l'écran de résultats
            if let Some((hit_stats, replay_data, score, accuracy, max_combo)) =
                ctx.with_renderer(|renderer| {
                    let hit_stats = renderer.engine.hit_stats.clone();
                    let replay_data = renderer.engine.replay_data.clone();
                    let score = renderer.engine.notes_passed;
                    let accuracy = hit_stats.calculate_accuracy();
                    let max_combo = renderer.engine.max_combo;
                    (hit_stats, replay_data, score, accuracy, max_combo)
                })
            {
                // Arrêter l'audio
                ctx.with_renderer(|renderer| renderer.stop_audio());

                return StateTransition::Replace(Box::new(ResultStateController::new(
                    Arc::clone(&self.menu_state),
                    hit_stats,
                    replay_data,
                    score,
                    accuracy,
                    max_combo,
                )));
            }
        }
        StateTransition::None
    }
}