pub mod audio;
pub mod engine;
pub mod state;

use crate::database::DbManager;
use crate::logic::state::GlobalState;
use crate::system::bus::{SystemBus, SystemEvent};
use std::thread;
use std::time::{Duration, Instant};

const TPS: u64 = 200;

pub fn start_thread(bus: SystemBus, db_manager: DbManager) {
    thread::Builder::new()
        .name("Logic Thread".to_string())
        .spawn(move || {
            log::info!("LOGIC: Thread started");

            // 1. Connexion DB
            db_manager.init();
            // 2. Charger les beatmaps existantes (sans vider la DB)
            db_manager.load();

            let input_cmd_tx = bus.input_cmd_tx.clone();
            let mut state = GlobalState::new(db_manager, input_cmd_tx);

            let mut accumulator = Duration::new(0, 0);
            let mut last_time = Instant::now();
            let target_dt = Duration::from_secs_f64(1.0 / TPS as f64);

            loop {
                // 1. Inputs
                while let Ok(action) = bus.action_rx.try_recv() {
                    state.handle_action(action);
                }

                // 2. Système
                while let Ok(sys_evt) = bus.sys_rx.try_recv() {
                    match sys_evt {
                        SystemEvent::Quit => {
                            log::info!("LOGIC: Quit received...");
                            state.shutdown();
                            return;
                        }
                        SystemEvent::Resize { width, height } => {
                            state.resize(width, height);
                        }
                        _ => {}
                    }
                }

                // 3. Physique
                let current_time = Instant::now();
                let delta = current_time - last_time;
                last_time = current_time;
                accumulator += delta;

                let mut updated = false;
                let mut loops = 0;
                while accumulator >= target_dt && loops < 10 {
                    state.update(target_dt.as_secs_f64());
                    accumulator -= target_dt;
                    loops += 1;
                    updated = true;
                }

                // 4. Rendu - Envoyer un snapshot seulement si on a fait un update
                // Cela évite d'envoyer des snapshots avec le même temps audio
                if updated {
                    let snapshot = state.create_snapshot();
                    let _ = bus.render_tx.try_send(snapshot);
                }
                state.frame_end();
                
                // Sleep adaptatif : moins de sleep si on a beaucoup de travail
                if loops == 0 {
                    thread::sleep(Duration::from_millis(1));
                }
            }
        })
        .expect("Failed to spawn Logic thread");
}
