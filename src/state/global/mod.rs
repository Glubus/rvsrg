//! Global state management for the game state machine.

mod actions;
mod app_state;
mod helpers;

use actions::editor::apply as apply_to_editor;
use actions::game::apply as apply_to_game;
use actions::menu::apply as apply_to_menu;
use actions::result::apply as apply_to_result;
use app_state::AppState;

use crate::database::{DbManager, DbStatus};
use crate::input::events::{GameAction, InputCommand};
use crate::models::settings::SettingsState;
use crate::shared::snapshot::{EditorSnapshot, RenderState};
use crate::state::MenuState;
use crate::state::traits::{Snapshot, Transition, Update, UpdateContext};
use crate::system::bus::SystemBus;
use crossbeam_channel::Sender;
use std::sync::Arc;

/// Owns the long-lived state machine for gameplay, menu and editor.
pub struct GlobalState {
    pub(super) current_state: AppState,
    pub(super) saved_menu_state: MenuState,
    pub(super) db_manager: DbManager,
    pub(super) last_db_version: u64,
    pub(super) last_leaderboard_version: u64,
    pub(super) requested_leaderboard_hash: Option<String>,
    pub(super) settings: SettingsState,
    pub(super) input_cmd_tx: Sender<InputCommand>,
    pub(super) bus: SystemBus,
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

        // Create the update context with shared resources
        let mut ctx = UpdateContext {
            db_manager: &mut self.db_manager,
            settings: &self.settings,
            bus: &self.bus,
        };

        // Call update on the current state and collect any transition
        let transition = match &mut self.current_state {
            AppState::Menu(menu) => Update::update(menu, dt, &mut ctx),
            AppState::Game(engine) => Update::update(engine, dt, &mut ctx),
            AppState::Result(result) => Update::update(result, dt, &mut ctx),
            AppState::Editor(editor) => {
                // Reset save flag each frame
                editor.save_requested = false;
                None
            }
        };

        // Apply any transition
        if let Some(Transition::ToResult(result)) = transition {
            self.current_state = AppState::Result(result);
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
                    menu.beatmapsets = Arc::new(guard.beatmapsets.clone());
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

    /// Asks the DB thread to refresh leaderboard data for a beatmap hash.
    pub(super) fn request_leaderboard_for_hash(&mut self, hash: Option<String>) {
        if let Some(hash) = hash
            && self.requested_leaderboard_hash.as_deref() != Some(hash.as_str())
        {
            self.db_manager.fetch_leaderboard(&hash);
            self.requested_leaderboard_hash = Some(hash);
        }
    }

    /// Persists the last known menu state so that leaving gameplay restores it.
    pub(super) fn cache_menu_state(&mut self, menu: MenuState) {
        self.saved_menu_state = menu;
    }

    /// Writes current settings to disk.
    pub(super) fn persist_settings(&self) {
        self.settings.save();
    }

    /// Reloads settings from disk (to sync with renderer's changes).
    pub(super) fn reload_settings(&mut self) {
        self.settings = SettingsState::load();
    }

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
                let next = apply_to_menu(self, menu, &action);
                self.cache_menu_state(menu.clone());
                next
            }
            AppState::Game(engine) => apply_to_game(self, engine, &action),
            AppState::Editor(editor) => apply_to_editor(self, editor, &action),
            AppState::Result(result) => apply_to_result(self, result, &action),
        };

        self.current_state = transition.unwrap_or(current_state);
    }

    /// Cleans up transient editor buffers so next frame starts fresh.
    pub fn frame_end(&mut self) {
        if let AppState::Editor(editor) = &mut self.current_state {
            // Don't clear modification_buffer here - it will be processed in create_snapshot
            // and cleared only after being used
            editor.save_requested = false;
        }
    }

    /// Produces a render-ready snapshot for the renderer thread.
    pub fn create_snapshot(&mut self) -> RenderState {
        match &mut self.current_state {
            AppState::Menu(menu) => RenderState::Menu(Snapshot::create_snapshot(menu)),
            AppState::Game(engine) => RenderState::InGame(Snapshot::create_snapshot(engine)),
            AppState::Editor(editor) => {
                let modification = if let (Some(t), Some((dx, dy))) =
                    (editor.target.as_ref(), editor.modification_buffer.as_ref())
                {
                    Some((*t, editor.mode, *dx, *dy))
                } else {
                    None
                };

                // Clear the buffer after using it
                if modification.is_some() {
                    editor.modification_buffer = None;
                }

                let status_text = if let Some(t) = editor.target.as_ref() {
                    format!("EDIT: {:?} [{}]", t, editor.mode)
                } else {
                    "SELECT: W(Note) X(Rec) C(Cmb) V(Scr) B(Acc) N(Judg) K(Bar) | S(Save)"
                        .to_string()
                };

                RenderState::Editor(EditorSnapshot {
                    game: Snapshot::create_snapshot(&editor.engine),
                    target: editor.target,
                    mode: editor.mode,
                    status_text,
                    modification,
                    save_requested: editor.save_requested,
                })
            }
            AppState::Result(res) => RenderState::Result(Snapshot::create_snapshot(res)),
        }
    }
}
