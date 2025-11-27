//! Input thread bootstrapping and high-level event routing.

pub mod events;
pub mod manager;

use crate::input::events::InputCommand;
use crate::input::manager::InputManager;
use crate::system::bus::SystemBus;
use crossbeam_channel::select;
use std::thread;

pub fn start_thread(bus: SystemBus, mut manager: InputManager) {
    thread::Builder::new()
        .name("Input Thread".to_string())
        .spawn(move || {
            log::info!("INPUT: Thread started");

            // Blocking loop: wait for an event, handle it, repeat.
            // Keeps CPU usage at zero when idle.
            loop {
                select! {
                    recv(bus.raw_input_rx) -> raw => {
                        match raw {
                            Ok(raw_event) => {
                                if let Some(action) = manager.process(raw_event) {
                                    if let Err(e) = bus.action_tx.send(action) {
                                        log::error!("INPUT: Failed to send action (Logic thread died?): {}", e);
                                        break;
                                    }
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    recv(bus.input_cmd_rx) -> cmd => {
                        match cmd {
                            Ok(InputCommand::ReloadKeybinds(map)) => manager.reload_keybinds(&map),
                            Err(_) => break,
                        }
                    }
                }
            }

            log::info!("INPUT: Thread stopped");
        })
        .expect("Failed to spawn Input thread");
}
