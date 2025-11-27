//! Application entry point and thread bootstrapper.

mod input;
mod logic;
mod render;
mod system;

mod core;
mod database;
mod difficulty;
mod models;
mod shaders;
mod shared;
mod states;
mod views;


use crate::database::DbManager;
use crate::system::bus::SystemBus;
use std::path::PathBuf;

fn main() {
    unsafe {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    log::info!("MAIN: Booting rVsrg 2.0...");

    let bus = SystemBus::new();

    let input_bus = bus.clone();
    let logic_bus = bus.clone();
    let render_bus = bus.clone();

    let db_path = PathBuf::from("main.db");
    let songs_path = PathBuf::from("songs");
    let db_manager = DbManager::new(db_path, songs_path);

    let input_manager = input::manager::InputManager::new();

    input::start_thread(input_bus, input_manager);
    logic::start_thread(logic_bus, db_manager);

    render::app::App::run(render_bus);
}
