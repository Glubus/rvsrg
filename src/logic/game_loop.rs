use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use crate::core::input::actions::KeyAction;
use crate::database::{DbManager, DbStatus};
use crate::models::engine::GameEngine;
use crate::models::menu::MenuState;
use crate::shared::messages::{LogicToMain, MainToLogic, EditorCommand};
use crate::shared::snapshot::{RenderState};

use crate::logic::input::{InputContext, InputHandler};
use crate::logic::input::menu::MenuInputHandler;
use crate::logic::input::game::GameInputHandler;
use crate::logic::input::editor::EditorInputHandler;

const TICK_RATE: u64 = 200; 
const TICK_DURATION: Duration = Duration::from_nanos(1_000_000_000 / TICK_RATE);

pub struct LogicLoop {
    rx: Receiver<MainToLogic>,
    tx: Sender<LogicToMain>,
    db_manager: DbManager,
    menu_state: MenuState,
    game_engine: Option<GameEngine>,
    master_volume: f32,
    last_db_beatmap_count: usize,
    
    menu_handler: MenuInputHandler,
    game_handler: GameInputHandler,
    editor_handler: EditorInputHandler,
}

impl LogicLoop {
    pub fn start(rx: Receiver<MainToLogic>, tx: Sender<LogicToMain>, db_manager: DbManager) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut logic = Self {
                rx, tx, db_manager,
                menu_state: MenuState::new(),
                game_engine: None,
                master_volume: 0.5,
                last_db_beatmap_count: 0,
                menu_handler: MenuInputHandler,
                game_handler: GameInputHandler,
                editor_handler: EditorInputHandler,
            };
            println!("LOGIC: Thread démarré !");
            logic.run();
        })
    }

    fn run(&mut self) {
        self.db_manager.init(); 
        let mut next_tick = Instant::now();

        loop {
            let now = Instant::now();
            while let Ok(msg) = self.rx.try_recv() {
                if !self.handle_message(msg) { return; }
            }

            self.update();
            self.sync_with_db();
            self.broadcast_state();

            next_tick += TICK_DURATION;
            if now < next_tick { thread::sleep(next_tick - now); } else { next_tick = now + TICK_DURATION; }
        }
    }

    fn sync_with_db(&mut self) {
        let db_state = self.db_manager.get_state();
        let guard = db_state.lock().unwrap();
        match guard.status {
            DbStatus::Idle => {
                if guard.beatmapsets.len() != self.last_db_beatmap_count {
                    self.menu_state.beatmapsets = guard.beatmapsets.clone();
                    self.last_db_beatmap_count = guard.beatmapsets.len();
                    self.menu_state.start_index = 0;
                    self.menu_state.end_index = self.menu_state.visible_count.min(self.menu_state.beatmapsets.len());
                    self.menu_state.selected_index = 0;
                    self.menu_state.selected_difficulty_index = 0;
                }
            }
            _ => {}
        }
    }

    fn handle_message(&mut self, msg: MainToLogic) -> bool {
        match msg {
            MainToLogic::Shutdown => return false,
            MainToLogic::Input(action) => self.process_input_with_handlers(action),
            MainToLogic::SettingsChanged => {}, 
            MainToLogic::Resize { .. } => {},
            MainToLogic::LoadMap { path, is_editor } => {
                let rate = self.menu_state.rate;
                let mut engine = GameEngine::from_map(path, rate);
                engine.scroll_speed_ms = 500.0; 
                engine.update_hit_window(crate::models::settings::HitWindowMode::OsuOD, 5.0);
                self.game_engine = Some(engine);
                self.menu_state.in_menu = false;
                self.menu_state.in_editor = is_editor;
                self.menu_state.show_settings = false;
            },
            MainToLogic::EditorCommand(_) => {},
            
            // --- GESTION DES NOUVELLES VARIANTES ---
            MainToLogic::TransitionToMenu => {
                self.menu_state.in_menu = true;
                self.menu_state.in_editor = false;
                self.menu_state.show_result = false;
                // On s'assure que le moteur de jeu est éteint
                if let Some(engine) = &self.game_engine {
                    engine.stop_audio();
                }
                self.game_engine = None;
                
                // On prévient l'App pour qu'elle change l'écran (GameState)
                let _ = self.tx.send(LogicToMain::TransitionToMenu);
            },
            MainToLogic::TransitionToResult(data) => {
                self.menu_state.in_menu = true;
                self.menu_state.show_result = true;
                self.menu_state.last_result = Some(data.clone());
                
                // On prévient l'App pour qu'elle change l'écran (GameState)
                let _ = self.tx.send(LogicToMain::TransitionToResult(data));
            }
        }
        true
    }

    fn process_input_with_handlers(&mut self, action: KeyAction) {
        let mut ctx = InputContext {
            menu_state: &mut self.menu_state,
            game_engine: &mut self.game_engine,
            db_manager: &self.db_manager,
            tx: &self.tx,
        };

        let consumed = if ctx.menu_state.in_editor {
            self.editor_handler.handle(action, &mut ctx)
        } else if ctx.menu_state.in_menu {
            self.menu_handler.handle(action, &mut ctx)
        } else {
            self.game_handler.handle(action, &mut ctx)
        };

        if !consumed {
            // Debug: Input non géré
        }
    }

    fn update(&mut self) {
        if let Some(engine) = &mut self.game_engine {
            engine.update(self.master_volume);
            if engine.is_game_finished() && !self.menu_state.in_editor {
                let result_data = crate::models::menu::GameResultData {
                    hit_stats: engine.hit_stats.clone(),
                    replay_data: engine.replay_data.clone(),
                    score: engine.notes_passed,
                    accuracy: engine.hit_stats.calculate_accuracy(),
                    max_combo: engine.max_combo,
                    beatmap_hash: None,
                    rate: engine.rate,
                    judge_text: "Judge".to_string(),
                };
                
                // Sauvegarde du replay si possible (asynchrone, on lance juste)
                // Note: Ici on n'attend pas la DB, c'est fait en background via db_manager normalement
                // ou on pourrait le faire ici si on avait accès au Runtime Tokio.
                // Pour l'instant, on se contente de la transition.
                
                engine.stop_audio();
                self.game_engine = None;
                
                self.menu_state.in_menu = true;
                self.menu_state.show_result = true;
                self.menu_state.last_result = Some(result_data.clone());
                
                let _ = self.tx.send(LogicToMain::TransitionToResult(result_data));
            }
        }
    }

    fn broadcast_state(&self) {
        let state = if let Some(engine) = &self.game_engine {
            RenderState::InGame(engine.get_snapshot())
        } else {
            RenderState::Menu(self.menu_state.clone())
        };
        let _ = self.tx.send(LogicToMain::StateUpdate(state));
    }
}