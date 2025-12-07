//! rVsrg - Rust Vertical Scrolling Rhythm Game
//!
//! A high-performance rhythm game engine supporting osu!mania-style beatmaps.
//!
//! # Architecture
//!
//! The application uses a multi-threaded architecture:
//! - **Main/Render thread**: Window events and GPU rendering (wgpu)
//! - **Logic thread**: Game state at 200 TPS with fixed timestep
//! - **Input thread**: Keyboard input processing and keybind mapping
//! - **Audio thread**: Dedicated audio playback with pitch shifting
//!
//! Communication between threads uses lock-free channels via [`SystemBus`].

mod input;
mod logic;
mod render;
mod system;

mod database;
mod difficulty;
mod models;
mod shaders;
mod shared;
mod views;

use crate::database::DbManager;
use crate::system::bus::SystemBus;
use std::path::PathBuf;

/// Application entry point.
///
/// Initializes logging, creates the inter-thread communication bus,
/// spawns worker threads, and runs the main render loop.
fn main() {
    // Initialize logging
    unsafe {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    log::info!("MAIN: Booting rVsrg 2.0...");

    // Create the central communication hub
    let bus = SystemBus::new();

    let input_bus = bus.clone();
    let logic_bus = bus.clone();
    let render_bus = bus.clone();

    // Initialize database manager
    let db_path = PathBuf::from("main.db");
    let songs_path = PathBuf::from("songs");
    let db_manager = DbManager::new(db_path, songs_path);

    // Initialize input manager
    let input_manager = input::manager::InputManager::new();

    // Spawn worker threads
    input::start_thread(input_bus, input_manager);
    logic::start_thread(logic_bus, db_manager);

    // Run the render loop (blocking)
    render::app::App::run(render_bus);
}
