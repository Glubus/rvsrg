use crate::database::{DbManager, DbStatus, SaveReplayCommand};
use crate::input::events::{EditMode, EditorTarget, GameAction, InputCommand};
use crate::logic::engine::GameEngine;
use crate::models::menu::{GameResultData, MenuState};
use crate::models::settings::{HitWindowMode, SettingsState};
use crate::shared::snapshot::{EditorSnapshot, RenderState};
use crossbeam_channel::Sender;
use serde_json;
use std::time::{SystemTime, UNIX_EPOCH};

enum AppState {
    Menu(MenuState),
    Game(GameEngine),
    Editor {
        engine: GameEngine,
        target: Option<EditorTarget>,
        mode: EditMode, // On stocke le mode courant ici
        modification_buffer: Option<(f32, f32)>,
        save_requested: bool,
    },
    Result(GameResultData),
}

pub struct GlobalState {
    current_state: AppState,
    db_manager: DbManager,
    last_db_version: u64,
    last_leaderboard_version: u64,
    requested_leaderboard_hash: Option<String>,
    settings: SettingsState,
    input_cmd_tx: Sender<InputCommand>,
}

impl GlobalState {
    pub fn new(db_manager: DbManager, input_cmd_tx: Sender<InputCommand>) -> Self {
        log::info!("LOGIC: Initializing Global State");
        let settings = SettingsState::load();
        let menu = MenuState::new();

        Self {
            current_state: AppState::Menu(menu),
            db_manager,
            last_db_version: 0,
            last_leaderboard_version: 0,
            requested_leaderboard_hash: None,
            settings,
            input_cmd_tx,
        }
    }

    pub fn resize(&mut self, _w: u32, _h: u32) {}
    pub fn shutdown(&mut self) {}

    pub fn update(&mut self, dt: f64) {
        self.sync_db_to_menu();

        let mut next_state = None;

        match &mut self.current_state {
            AppState::Menu(menu) => {
                menu.ensure_selected_rate_cache();
            }
            AppState::Game(engine) => {
                engine.update(dt);
                if engine.is_finished() {
                    let accuracy = engine.hit_stats.calculate_accuracy();
                    if let Some(payload) = Self::build_replay_payload(engine, accuracy) {
                        self.db_manager.save_replay(payload);
                    }
                    let result = GameResultData {
                        hit_stats: engine.hit_stats.clone(),
                        replay_data: engine.replay_data.clone(),
                        score: engine.score,
                        accuracy,
                        max_combo: engine.max_combo,
                        beatmap_hash: engine.beatmap_hash.clone(),
                        rate: engine.rate,
                        judge_text: self.get_hit_window_text(),
                    };
                    next_state = Some(AppState::Result(result));
                }
            }
            AppState::Editor {
                engine: _,
                modification_buffer: _,
                save_requested,
                ..
            } => {
                *save_requested = false;
            }
            AppState::Result(_) => {}
        }

        if let Some(state) = next_state {
            self.current_state = state;
        }
    }

    fn sync_db_to_menu(&mut self) {
        let db_state_arc = self.db_manager.get_state();
        if let Ok(guard) = db_state_arc.try_lock() {
            if matches!(guard.status, DbStatus::Idle) {
                if guard.version != self.last_db_version {
                    if let AppState::Menu(menu) = &mut self.current_state {
                        menu.beatmapsets = guard.beatmapsets.clone();
                        menu.start_index = 0;
                        menu.end_index = menu.visible_count.min(menu.beatmapsets.len());
                        menu.selected_index = 0;
                        menu.selected_difficulty_index = 0;
                        let request_hash = menu.get_selected_beatmap_hash();
                        self.request_leaderboard_for_hash(request_hash);
                    }
                    self.last_db_version = guard.version;
                }

                if guard.leaderboard_version != self.last_leaderboard_version {
                    if let AppState::Menu(menu) = &mut self.current_state {
                        menu.set_leaderboard(
                            guard.leaderboard_hash.clone(),
                            guard.leaderboard.clone(),
                        );
                    }
                    self.last_leaderboard_version = guard.leaderboard_version;
                    if let Some(hash) = &guard.leaderboard_hash {
                        if self.requested_leaderboard_hash.as_deref() == Some(hash.as_str()) {
                            self.requested_leaderboard_hash = None;
                        }
                    }
                }
            }
        }
    }

    fn get_hit_window_text(&self) -> String {
        match self.settings.hit_window_mode {
            HitWindowMode::OsuOD => format!("OD {:.1}", self.settings.hit_window_value),
            HitWindowMode::EtternaJudge => format!("Judge {:.0}", self.settings.hit_window_value),
        }
    }

    fn request_leaderboard_for_hash(&mut self, hash: Option<String>) {
        if let Some(hash) = hash {
            if self.requested_leaderboard_hash.as_deref() != Some(hash.as_str()) {
                self.db_manager.fetch_leaderboard(&hash);
                self.requested_leaderboard_hash = Some(hash);
            }
        }
    }

    fn build_replay_payload(engine: &GameEngine, accuracy: f64) -> Option<SaveReplayCommand> {
        let hash = engine.beatmap_hash.clone()?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let data = serde_json::to_string(&engine.replay_data).ok()?;

        Some(SaveReplayCommand {
            beatmap_hash: hash,
            timestamp,
            score: engine.score.min(i32::MAX as u32) as i32,
            accuracy,
            max_combo: engine.max_combo.min(i32::MAX as u32) as i32,
            rate: engine.rate,
            data,
        })
    }

    pub fn handle_action(&mut self, action: GameAction) {
        if let GameAction::ReloadKeybinds = action {
            self.reload_keybinds_from_disk();
            return;
        }

        let mut next_state = None;

        match &mut self.current_state {
            AppState::Menu(menu) => {
                match action {
                    GameAction::Navigation { x, y } => {
                        let request_hash = {
                            if y < 0 {
                                menu.move_up();
                            }
                            if y > 0 {
                                menu.move_down();
                            }
                            if x < 0 {
                                menu.previous_difficulty();
                            }
                            if x > 0 {
                                menu.next_difficulty();
                            }
                            menu.get_selected_beatmap_hash()
                        };
                        self.request_leaderboard_for_hash(request_hash);
                    }
                    GameAction::SetSelection(idx) => {
                        let request_hash = {
                            if idx < menu.beatmapsets.len() {
                                menu.selected_index = idx;
                                menu.selected_difficulty_index = 0;
                                if idx < menu.start_index {
                                    menu.start_index = idx;
                                    menu.end_index = (menu.start_index + menu.visible_count)
                                        .min(menu.beatmapsets.len());
                                } else if idx >= menu.end_index {
                                    menu.end_index = (idx + 1).min(menu.beatmapsets.len());
                                    menu.start_index =
                                        menu.end_index.saturating_sub(menu.visible_count);
                                }
                                menu.get_selected_beatmap_hash()
                            } else {
                                None
                            }
                        };
                        self.request_leaderboard_for_hash(request_hash);
                    }
                    GameAction::SetDifficulty(idx) => {
                        let request_hash = {
                            menu.selected_difficulty_index = idx;
                            menu.get_selected_beatmap_hash()
                        };
                        self.request_leaderboard_for_hash(request_hash);
                    }
                    GameAction::Confirm => {
                        if let Some(path) = menu.get_selected_beatmap_path() {
                            let beatmap_hash = menu.get_selected_beatmap_hash();
                            let mut engine = GameEngine::new(path, menu.rate, beatmap_hash.clone());
                            engine.scroll_speed_ms = self.settings.scroll_speed;
                            engine.update_hit_window(
                                self.settings.hit_window_mode,
                                self.settings.hit_window_value,
                            );
                            engine.audio_manager.set_volume(self.settings.master_volume);
                            next_state = Some(AppState::Game(engine));
                        }
                    }
                    GameAction::ToggleEditor => {
                        if let Some(path) = menu.get_selected_beatmap_path() {
                            let mut engine = GameEngine::new(path, 1.0, None);
                            engine.scroll_speed_ms = self.settings.scroll_speed;
                            engine.update_hit_window(
                                self.settings.hit_window_mode,
                                self.settings.hit_window_value,
                            );
                            engine.audio_manager.set_volume(self.settings.master_volume);

                            next_state = Some(AppState::Editor {
                                engine,
                                target: None,
                                mode: EditMode::Move, // Mode par défaut
                                modification_buffer: None,
                                save_requested: false,
                            });
                        }
                    }
                    GameAction::TabNext => menu.increase_rate(),
                    GameAction::TabPrev => menu.decrease_rate(),
                    GameAction::ToggleSettings => menu.show_settings = !menu.show_settings,
                    GameAction::UpdateVolume(value) => {
                        self.settings.master_volume = value;
                    }
                    GameAction::Rescan => {
                        self.db_manager.rescan();
                        self.last_db_version = u64::MAX;
                    }
                    GameAction::ApplySearch(filters) => {
                        menu.search_filters = filters.clone();
                        self.db_manager.search(filters);
                        self.requested_leaderboard_hash = None;
                        self.last_leaderboard_version = 0;
                    }
                    _ => {}
                }
            }
            AppState::Game(engine) => match action {
                GameAction::Back => {
                    engine.audio_manager.stop();
                    let new_menu = MenuState::new();
                    self.last_db_version = u64::MAX;
                    self.last_leaderboard_version = 0;
                    self.requested_leaderboard_hash = None;
                    next_state = Some(AppState::Menu(new_menu));
                }
                GameAction::UpdateVolume(value) => {
                    self.settings.master_volume = value;
                    engine.audio_manager.set_volume(value);
                }
                _ => {
                    engine.handle_input(action);
                }
            },
            AppState::Editor {
                engine,
                target,
                mode,
                modification_buffer,
                save_requested,
            } => {
                match action {
                    GameAction::Back => {
                        engine.audio_manager.stop();
                        let new_menu = MenuState::new();
                        self.last_db_version = u64::MAX;
                        self.last_leaderboard_version = 0;
                        self.requested_leaderboard_hash = None;
                        next_state = Some(AppState::Menu(new_menu));
                    }
                    GameAction::EditorSelect(t) => {
                        if *target == Some(t) {
                            // Si même cible, on bascule le mode
                            *mode = match *mode {
                                EditMode::Resize => EditMode::Move,
                                EditMode::Move => EditMode::Resize,
                            };
                        } else {
                            // Nouvelle cible, mode par défaut intelligent
                            *target = Some(t);
                            *mode = match t {
                                EditorTarget::Notes
                                | EditorTarget::Receptors
                                | EditorTarget::HitBar => EditMode::Resize,
                                _ => EditMode::Move,
                            };
                        }
                    }
                    GameAction::Navigation { x, y } => {
                        if target.is_some() {
                            *modification_buffer = Some((x as f32, y as f32));
                        }
                    }
                    GameAction::EditorSave => *save_requested = true,
                    GameAction::UpdateVolume(value) => {
                        self.settings.master_volume = value;
                        engine.audio_manager.set_volume(value);
                    }
                    GameAction::Hit { column } => engine.handle_input(GameAction::Hit { column }),
                    GameAction::Release { column } => {
                        engine.handle_input(GameAction::Release { column })
                    }
                    _ => {}
                }
            }
            AppState::Result(_) => match action {
                GameAction::Back | GameAction::Confirm => {
                    let new_menu = MenuState::new();
                    self.last_db_version = u64::MAX;
                    self.last_leaderboard_version = 0;
                    self.requested_leaderboard_hash = None;
                    next_state = Some(AppState::Menu(new_menu));
                }
                _ => {}
            },
        }

        if let Some(state) = next_state {
            self.current_state = state;
        }
    }

    pub fn frame_end(&mut self) {
        if let AppState::Editor {
            modification_buffer,
            save_requested,
            ..
        } = &mut self.current_state
        {
            *modification_buffer = None;
            *save_requested = false;
        }
    }

    pub fn create_snapshot(&self) -> RenderState {
        match &self.current_state {
            AppState::Menu(menu) => RenderState::Menu(menu.clone()),
            AppState::Game(engine) => RenderState::InGame(engine.get_snapshot()),
            AppState::Editor {
                engine,
                target,
                mode,
                modification_buffer,
                save_requested,
            } => {
                let modification = if let (Some(t), Some((dx, dy))) = (target, modification_buffer)
                {
                    Some((*t, *mode, *dx, *dy))
                } else {
                    None
                };

                let status_text = if let Some(t) = target {
                    format!("EDIT: {:?} [{}]", t, mode)
                } else {
                    "SELECT: W(Note) X(Rec) C(Cmb) V(Scr) B(Acc) N(Judg) K(Bar) | S(Save)"
                        .to_string()
                };

                RenderState::Editor(EditorSnapshot {
                    game: engine.get_snapshot(),
                    target: *target,
                    mode: *mode, // On passe le mode au renderer
                    status_text,
                    modification,
                    save_requested: *save_requested,
                })
            }
            AppState::Result(res) => RenderState::Result(res.clone()),
        }
    }
}

impl GlobalState {
    fn reload_keybinds_from_disk(&mut self) {
        let disk_settings = SettingsState::load();
        self.settings.keybinds = disk_settings.keybinds.clone();
        if let Err(e) = self
            .input_cmd_tx
            .send(InputCommand::ReloadKeybinds(self.settings.keybinds.clone()))
        {
            log::error!("LOGIC: Failed to forward keybinds to input thread: {}", e);
        }
    }
}
