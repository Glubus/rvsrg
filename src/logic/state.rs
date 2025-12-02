use crate::database::{DbManager, DbStatus, SaveReplayCommand};
use crate::input::events::{EditMode, EditorTarget, GameAction, InputCommand};
use crate::logic::engine::GameEngine;
use crate::models::menu::{GameResultData, MenuState};
use crate::models::replay::simulate_replay;
use crate::models::settings::{HitWindowMode, SettingsState};
use crate::shared::snapshot::{EditorSnapshot, RenderState};
use crate::system::bus::SystemBus;
use crossbeam_channel::Sender;
use serde_json;
use std::time::{SystemTime, UNIX_EPOCH};

/// High-level application states driven by `GlobalState`.
enum AppState {
    /// Song select and menu browsing.
    Menu(MenuState),
    /// Live gameplay.
    Game(GameEngine),
    /// Beatmap editor wrapper.
    Editor {
        engine: GameEngine,
        target: Option<EditorTarget>,
        /// Tracks the current edit mode so the renderer/input stay in sync.
        mode: EditMode,
        modification_buffer: Option<(f32, f32)>,
        save_requested: bool,
    },
    /// Post-game result screen.
    Result(GameResultData),
}

/// Owns the long-lived state machine for gameplay, menu and editor.
pub struct GlobalState {
    current_state: AppState,
    saved_menu_state: MenuState,
    db_manager: DbManager,
    last_db_version: u64,
    last_leaderboard_version: u64,
    requested_leaderboard_hash: Option<String>,
    settings: SettingsState,
    input_cmd_tx: Sender<InputCommand>,
    bus: SystemBus,
}

impl GlobalState {
    /// Creates a new state machine with default menu/settings and DB plumbing.
    pub fn new(db_manager: DbManager, input_cmd_tx: Sender<InputCommand>, bus: SystemBus) -> Self {
        log::info!("LOGIC: Initializing Global State");
        let settings = SettingsState::load();
        let menu = MenuState::new();

        Self {
            saved_menu_state: menu.clone(),
            current_state: AppState::Menu(menu),
            db_manager,
            last_db_version: 0,
            last_leaderboard_version: 0,
            requested_leaderboard_hash: None,
            settings,
            input_cmd_tx,
            bus,
        }
    }

    pub fn resize(&mut self, _w: u32, _h: u32) {}
    pub fn shutdown(&mut self) {}

    /// Ticks the active state and processes end-of-run transitions.
    pub fn update(&mut self, dt: f64) {
        self.sync_db_to_menu();

        let mut next_state = None;

        match &mut self.current_state {
            AppState::Menu(menu) => {
                menu.ensure_selected_rate_cache();
                menu.ensure_chart_cache();
            }
            AppState::Game(engine) => {
                engine.update(dt);
                if engine.is_finished() {
                    // Simulate replay to get detailed results (hit_timings, etc.)
                    let chart = engine.get_chart();
                    let replay_result =
                        simulate_replay(&engine.replay_data, &chart, &engine.hit_window);

                    let accuracy = replay_result.accuracy;
                    if let Some(payload) = Self::build_replay_payload(engine, accuracy) {
                        self.db_manager.save_replay(payload);
                    }

                    let result = GameResultData {
                        hit_stats: replay_result.hit_stats.clone(),
                        replay_data: engine.replay_data.clone(),
                        replay_result,
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

    /// Mirrors database snapshots into the menu whenever new data is available.
    fn sync_db_to_menu(&mut self) {
        let db_state_arc = self.db_manager.get_state();
        if let Ok(guard) = db_state_arc.try_lock()
            && matches!(guard.status, DbStatus::Idle)
        {
            if guard.version != self.last_db_version {
                let mut request_hash = None;
                let mut cache = None;
                if let AppState::Menu(menu) = &mut self.current_state {
                    menu.beatmapsets = guard.beatmapsets.clone();
                    menu.start_index = 0;
                    menu.end_index = menu.visible_count.min(menu.beatmapsets.len());
                    menu.selected_index = 0;
                    menu.selected_difficulty_index = 0;
                    request_hash = menu.get_selected_beatmap_hash();
                    cache = Some(menu.clone());
                }
                if let Some(menu) = cache {
                    self.cache_menu_state(menu);
                }
                self.request_leaderboard_for_hash(request_hash);
                self.last_db_version = guard.version;
            }

            if guard.leaderboard_version != self.last_leaderboard_version {
                let mut cache = None;
                if let AppState::Menu(menu) = &mut self.current_state {
                    menu.set_leaderboard(guard.leaderboard_hash.clone(), guard.leaderboard.clone());
                    cache = Some(menu.clone());
                }
                if let Some(menu) = cache {
                    self.cache_menu_state(menu);
                }
                self.last_leaderboard_version = guard.leaderboard_version;
                if let Some(hash) = &guard.leaderboard_hash
                    && self.requested_leaderboard_hash.as_deref() == Some(hash.as_str())
                {
                    self.requested_leaderboard_hash = None;
                }
            }
        }
    }

    /// Formats the active judgement window as text for the UI/result screen.
    fn get_hit_window_text(&self) -> String {
        match self.settings.hit_window_mode {
            HitWindowMode::OsuOD => format!("OD {:.1}", self.settings.hit_window_value),
            HitWindowMode::EtternaJudge => format!("Judge {:.0}", self.settings.hit_window_value),
        }
    }

    /// Asks the DB thread to refresh leaderboard data for a beatmap hash.
    fn request_leaderboard_for_hash(&mut self, hash: Option<String>) {
        if let Some(hash) = hash
            && self.requested_leaderboard_hash.as_deref() != Some(hash.as_str())
        {
            self.db_manager.fetch_leaderboard(&hash);
            self.requested_leaderboard_hash = Some(hash);
        }
    }

    /// Persists the last known menu state so that leaving gameplay restores it.
    fn cache_menu_state(&mut self, menu: MenuState) {
        self.saved_menu_state = menu;
    }

    /// Writes current settings to disk.
    fn persist_settings(&self) {
        self.settings.save();
    }

    /// Converts gameplay stats into a DB command for replay persistence.
    fn build_replay_payload(engine: &GameEngine, accuracy: f64) -> Option<SaveReplayCommand> {
        let hash = match engine.beatmap_hash.clone() {
            Some(h) => h,
            None => {
                log::error!("REPLAY: Cannot save - beatmap_hash is None!");
                return None;
            }
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let data = match serde_json::to_string(&engine.replay_data) {
            Ok(d) => d,
            Err(e) => {
                log::error!("REPLAY: Cannot serialize replay_data: {}", e);
                return None;
            }
        };

        log::info!(
            "REPLAY: Building payload for {} (score: {}, accuracy: {:.2}%, inputs: {})",
            hash,
            engine.score,
            accuracy,
            engine.replay_data.inputs.len()
        );

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

    /// Routes a `GameAction` to the current state and applies the resulting transition.
    pub fn handle_action(&mut self, action: GameAction) {
        if let GameAction::ReloadKeybinds = action {
            self.reload_keybinds_from_disk();
            return;
        }

        let mut current_state =
            std::mem::replace(&mut self.current_state, AppState::Menu(MenuState::new()));

        let transition = match &mut current_state {
            AppState::Menu(menu) => {
                let next = action.apply_to_menu(self, menu);
                self.cache_menu_state(menu.clone());
                next
            }
            AppState::Game(engine) => action.apply_to_game(self, engine),
            AppState::Editor {
                engine,
                target,
                mode,
                modification_buffer,
                save_requested,
            } => action.apply_to_editor(
                self,
                engine,
                target,
                mode,
                modification_buffer,
                save_requested,
            ),
            AppState::Result(result) => action.apply_to_result(self, result),
        };

        self.current_state = transition.unwrap_or(current_state);
    }

    /// Cleans up transient editor buffers so next frame starts fresh.
    pub fn frame_end(&mut self) {
        if let AppState::Editor {
            modification_buffer: _,
            save_requested,
            ..
        } = &mut self.current_state
        {
            // Don't clear modification_buffer here - it will be processed in create_snapshot
            // and cleared only after being used
            *save_requested = false;
        }
    }

    /// Produces a render-ready snapshot for the renderer thread.
    pub fn create_snapshot(&mut self) -> RenderState {
        match &mut self.current_state {
            AppState::Menu(menu) => RenderState::Menu(menu.clone()),
            AppState::Game(engine) => RenderState::InGame(engine.get_snapshot()),
            AppState::Editor {
                engine,
                target,
                mode,
                modification_buffer,
                save_requested,
            } => {
                let modification = if let (Some(t), Some((dx, dy))) =
                    (target.as_ref(), modification_buffer.as_ref())
                {
                    Some((*t, *mode, *dx, *dy))
                } else {
                    None
                };

                // Clear the buffer after using it
                if modification.is_some() {
                    *modification_buffer = None;
                }

                let status_text = if let Some(t) = target.as_ref() {
                    format!("EDIT: {:?} [{}]", t, mode)
                } else {
                    "SELECT: W(Note) X(Rec) C(Cmb) V(Scr) B(Acc) N(Judg) K(Bar) | S(Save)"
                        .to_string()
                };

                RenderState::Editor(EditorSnapshot {
                    game: engine.get_snapshot(),
                    target: *target,
                    // Editor mode is forwarded so the renderer can show the right hints.
                    mode: *mode,
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
    /// Reloads bindings from disk and forwards them to the input thread.
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

impl GameAction {
    /// Handles menu-specific behavior for this action.
    fn apply_to_menu(&self, state: &mut GlobalState, menu: &mut MenuState) -> Option<AppState> {
        match self {
            GameAction::Navigation { x, y } => {
                if *y < 0 {
                    menu.move_up();
                }
                if *y > 0 {
                    menu.move_down();
                }
                if *x < 0 {
                    menu.previous_difficulty();
                }
                if *x > 0 {
                    menu.next_difficulty();
                }
                let request_hash = menu.get_selected_beatmap_hash();
                state.request_leaderboard_for_hash(request_hash);
                None
            }
            GameAction::SetSelection(idx) => {
                if *idx < menu.beatmapsets.len() {
                    menu.selected_index = *idx;
                    menu.selected_difficulty_index = 0;
                    if *idx < menu.start_index {
                        menu.start_index = *idx;
                        menu.end_index =
                            (menu.start_index + menu.visible_count).min(menu.beatmapsets.len());
                    } else if *idx >= menu.end_index {
                        menu.end_index = (*idx + 1).min(menu.beatmapsets.len());
                        menu.start_index = menu.end_index.saturating_sub(menu.visible_count);
                    }
                }
                let request_hash = menu.get_selected_beatmap_hash();
                state.request_leaderboard_for_hash(request_hash);
                None
            }
            GameAction::SetDifficulty(idx) => {
                menu.selected_difficulty_index = *idx;
                let request_hash = menu.get_selected_beatmap_hash();
                state.request_leaderboard_for_hash(request_hash);
                None
            }
            GameAction::Confirm => {
                // Use cached chart if available, otherwise load from file
                let engine = if let Some(cache) = menu.get_cached_chart() {
                    // Reset hit notes for new gameplay
                    let chart: Vec<_> = cache
                        .chart
                        .iter()
                        .map(|n| crate::models::engine::NoteData {
                            timestamp_ms: n.timestamp_ms,
                            column: n.column,
                            hit: false,
                            is_hold: n.is_hold,
                            hold_duration_ms: n.hold_duration_ms,
                        })
                        .collect();

                    // Use cache hash (guaranteed consistent with chart)
                    let beatmap_hash = Some(cache.beatmap_hash.clone());

                    log::info!(
                        "GAME: Using cached chart ({} notes, hash: {:?})",
                        chart.len(),
                        beatmap_hash
                    );
                    GameEngine::from_cached(
                        &state.bus,
                        chart,
                        cache.audio_path.clone(),
                        menu.rate,
                        beatmap_hash,
                        state.settings.hit_window_mode,
                        state.settings.hit_window_value,
                    )
                } else if let Some(path) = menu.get_selected_beatmap_path() {
                    let beatmap_hash = menu.get_selected_beatmap_hash();
                    log::info!(
                        "GAME: Loading chart from file (no cache), hash: {:?}",
                        beatmap_hash
                    );
                    GameEngine::new(
                        &state.bus,
                        path,
                        menu.rate,
                        beatmap_hash,
                        state.settings.hit_window_mode,
                        state.settings.hit_window_value,
                    )
                } else {
                    return None;
                };

                let mut engine = engine;
                engine.scroll_speed_ms = state.settings.scroll_speed;
                engine
                    .audio_manager
                    .set_volume(state.settings.master_volume);
                Some(AppState::Game(engine))
            }
            GameAction::LaunchPractice => {
                // Like Confirm, but enables Practice mode
                let engine = if let Some(cache) = menu.get_cached_chart() {
                    let chart: Vec<_> = cache
                        .chart
                        .iter()
                        .map(|n| crate::models::engine::NoteData {
                            timestamp_ms: n.timestamp_ms,
                            column: n.column,
                            hit: false,
                            is_hold: n.is_hold,
                            hold_duration_ms: n.hold_duration_ms,
                        })
                        .collect();

                    let beatmap_hash = Some(cache.beatmap_hash.clone());

                    log::info!(
                        "PRACTICE: Using cached chart ({} notes, hash: {:?})",
                        chart.len(),
                        beatmap_hash
                    );
                    GameEngine::from_cached(
                        &state.bus,
                        chart,
                        cache.audio_path.clone(),
                        menu.rate,
                        beatmap_hash,
                        state.settings.hit_window_mode,
                        state.settings.hit_window_value,
                    )
                } else if let Some(path) = menu.get_selected_beatmap_path() {
                    let beatmap_hash = menu.get_selected_beatmap_hash();
                    log::info!(
                        "PRACTICE: Loading chart from file (no cache), hash: {:?}",
                        beatmap_hash
                    );
                    GameEngine::new(
                        &state.bus,
                        path,
                        menu.rate,
                        beatmap_hash,
                        state.settings.hit_window_mode,
                        state.settings.hit_window_value,
                    )
                } else {
                    return None;
                };

                let mut engine = engine;
                engine.scroll_speed_ms = state.settings.scroll_speed;
                engine
                    .audio_manager
                    .set_volume(state.settings.master_volume);
                engine.enable_practice_mode(); // Active le mode practice
                Some(AppState::Game(engine))
            }
            GameAction::ToggleEditor => {
                // Utiliser la chart cachée si disponible
                let engine = if let Some(cache) = menu.get_cached_chart() {
                    let chart: Vec<_> = cache
                        .chart
                        .iter()
                        .map(|n| crate::models::engine::NoteData {
                            timestamp_ms: n.timestamp_ms,
                            column: n.column,
                            hit: false,
                            is_hold: n.is_hold,
                            hold_duration_ms: n.hold_duration_ms,
                        })
                        .collect();

                    GameEngine::from_cached(
                        &state.bus,
                        chart,
                        cache.audio_path.clone(),
                        1.0,
                        None,
                        state.settings.hit_window_mode,
                        state.settings.hit_window_value,
                    )
                } else if let Some(path) = menu.get_selected_beatmap_path() {
                    GameEngine::new(
                        &state.bus,
                        path,
                        1.0,
                        None,
                        state.settings.hit_window_mode,
                        state.settings.hit_window_value,
                    )
                } else {
                    return None;
                };

                let mut engine = engine;
                engine.scroll_speed_ms = state.settings.scroll_speed;
                engine
                    .audio_manager
                    .set_volume(state.settings.master_volume);

                Some(AppState::Editor {
                    engine,
                    target: None,
                    mode: EditMode::Move,
                    modification_buffer: None,
                    save_requested: false,
                })
            }
            GameAction::TabNext => {
                menu.increase_rate();
                None
            }
            GameAction::TabPrev => {
                menu.decrease_rate();
                None
            }
            GameAction::ToggleSettings => {
                menu.show_settings = !menu.show_settings;
                None
            }
            GameAction::UpdateVolume(value) => {
                state.settings.master_volume = *value;
                state.persist_settings();
                None
            }
            GameAction::Rescan => {
                state.db_manager.rescan();
                state.last_db_version = u64::MAX;
                None
            }
            GameAction::ApplySearch(filters) => {
                menu.search_filters = filters.clone();
                state.db_manager.search(filters.clone());
                state.requested_leaderboard_hash = None;
                state.last_leaderboard_version = 0;
                None
            }
            GameAction::SetCalculator(calc_id) => {
                menu.set_calculator(&calc_id);
                // Recalculate difficulty for current map with new calculator
                menu.ensure_difficulty_calculated();
                None
            }
            GameAction::SetResult(result_data) => Some(AppState::Result(result_data.clone())),
            GameAction::TogglePause
            | GameAction::Hit { .. }
            | GameAction::Release { .. }
            | GameAction::Restart
            | GameAction::Back
            | GameAction::EditorSelect(_)
            | GameAction::EditorModify { .. }
            | GameAction::EditorSave
            | GameAction::ReloadKeybinds
            | GameAction::PracticeCheckpoint
            | GameAction::PracticeRetry => None,
        }
    }

    /// Handles in-game behavior for this action.
    fn apply_to_game(&self, state: &mut GlobalState, engine: &mut GameEngine) -> Option<AppState> {
        match self {
            GameAction::Back => {
                engine.audio_manager.stop();
                state.requested_leaderboard_hash = None;
                let menu = state.saved_menu_state.clone();
                let request_hash = menu.get_selected_beatmap_hash();
                state.request_leaderboard_for_hash(request_hash);
                Some(AppState::Menu(menu))
            }
            GameAction::UpdateVolume(value) => {
                state.settings.master_volume = *value;
                engine.audio_manager.set_volume(*value);
                state.persist_settings();
                None
            }
            GameAction::ReloadKeybinds => None,
            _ => {
                engine.handle_input(self.clone());
                None
            }
        }
    }

    /// Handles editor-specific behavior for this action.
    fn apply_to_editor(
        &self,
        state: &mut GlobalState,
        engine: &mut GameEngine,
        target: &mut Option<EditorTarget>,
        mode: &mut EditMode,
        modification_buffer: &mut Option<(f32, f32)>,
        save_requested: &mut bool,
    ) -> Option<AppState> {
        match self {
            GameAction::Back => {
                engine.audio_manager.stop();
                state.requested_leaderboard_hash = None;
                let menu = state.saved_menu_state.clone();
                let request_hash = menu.get_selected_beatmap_hash();
                state.request_leaderboard_for_hash(request_hash);
                Some(AppState::Menu(menu))
            }
            GameAction::EditorSelect(t) => {
                if *target == Some(*t) {
                    *mode = match *mode {
                        EditMode::Resize => EditMode::Move,
                        EditMode::Move => EditMode::Resize,
                    };
                } else {
                    *target = Some(*t);
                    *mode = match t {
                        EditorTarget::Notes | EditorTarget::Receptors | EditorTarget::HitBar => {
                            EditMode::Resize
                        }
                        _ => EditMode::Move,
                    };
                }
                None
            }
            GameAction::Navigation { x, y } => {
                if target.is_some() {
                    // Accumuler les modifications au lieu de les remplacer
                    // Cela permet de traiter plusieurs inputs même s'ils arrivent rapidement
                    let (dx, dy) = (*x as f32, *y as f32);
                    if let Some((old_dx, old_dy)) = modification_buffer {
                        *modification_buffer = Some((*old_dx + dx, *old_dy + dy));
                    } else {
                        *modification_buffer = Some((dx, dy));
                    }
                }
                None
            }
            GameAction::EditorModify { x, y } => {
                if target.is_some() {
                    // Accumuler les modifications au lieu de les remplacer
                    if let Some((old_dx, old_dy)) = modification_buffer {
                        *modification_buffer = Some((*old_dx + *x, *old_dy + *y));
                    } else {
                        *modification_buffer = Some((*x, *y));
                    }
                }
                None
            }
            GameAction::EditorSave => {
                *save_requested = true;
                None
            }
            GameAction::UpdateVolume(value) => {
                state.settings.master_volume = *value;
                engine.audio_manager.set_volume(*value);
                state.persist_settings();
                None
            }
            GameAction::Hit { column } => {
                engine.handle_input(GameAction::Hit { column: *column });
                None
            }
            GameAction::Release { column } => {
                engine.handle_input(GameAction::Release { column: *column });
                None
            }
            _ => None,
        }
    }

    /// Handles result screen behavior for this action.
    fn apply_to_result(
        &self,
        state: &mut GlobalState,
        _result: &mut GameResultData,
    ) -> Option<AppState> {
        match self {
            GameAction::Back | GameAction::Confirm => {
                state.requested_leaderboard_hash = None;
                let menu = state.saved_menu_state.clone();
                let request_hash = menu.get_selected_beatmap_hash();
                state.request_leaderboard_for_hash(request_hash);
                Some(AppState::Menu(menu))
            }
            _ => None,
        }
    }
}
