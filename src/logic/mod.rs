//! Logic thread module for game state management and updates.
//!
//! This module contains the main game loop that runs at a fixed tick rate
//! and coordinates between input, audio, and rendering subsystems.

pub mod audio;
pub mod audio_thread;

use crate::database::DbManager;
use crate::state::GlobalState;
use crate::system::bus::{SystemBus, SystemEvent};
use std::thread;
use std::time::{Duration, Instant};

/// Target ticks per second for the logic thread.
const TPS: u64 = 200;

/// Spawns the main logic thread that handles game state updates.
///
/// This thread runs a fixed-timestep game loop that:
/// 1. Processes input actions from the input thread
/// 2. Handles system events (resize, quit, etc.)
/// 3. Updates game state at a fixed rate
/// 4. Sends render snapshots to the render thread
pub fn start_thread(bus: SystemBus, db_manager: DbManager) {
    // Start the dedicated audio thread
    audio_thread::start_audio_thread(bus.clone());

    thread::Builder::new()
        .name("Logic Thread".to_string())
        .spawn(move || {
            log::info!("LOGIC: Thread started");

            // Initialize and load database
            db_manager.init();
            db_manager.load();

            let input_cmd_tx = bus.input_cmd_tx.clone();
            let mut state = GlobalState::new(db_manager, input_cmd_tx, bus.clone());

            let mut accumulator = Duration::new(0, 0);
            let mut last_time = Instant::now();
            let target_dt = Duration::from_secs_f64(1.0 / TPS as f64);

            loop {
                // 1. Process input actions
                while let Ok(action) = bus.action_rx.try_recv() {
                    state.handle_action(action);
                }

                // 2. Handle system events
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

                // 3. Fixed-timestep update loop
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

                // 4. Send render snapshot only if we updated
                // This avoids sending duplicate snapshots with the same audio time
                if updated {
                    let snapshot = state.create_snapshot();
                    let _ = bus.render_tx.try_send(snapshot);
                }
                state.frame_end();

                // Adaptive sleep: less sleep when there's heavy workload
                if loops == 0 {
                    thread::sleep(Duration::from_millis(1));
                }
            }
        })
        .expect("Failed to spawn Logic thread");
}
