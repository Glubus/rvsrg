mod app;
mod core;    // NOUVEAU
mod database;
mod logic;   // NOUVEAU
mod models;
mod renderer;
mod shaders;
mod shared;  // NOUVEAU
mod states;
mod views;

use app::App;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    env_logger::init();

    // Créer un runtime tokio global pour les opérations async (Database, etc.)
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _enter = rt.enter(); // Entrer dans le contexte du runtime

    let event_loop = EventLoop::new().unwrap();
    // Pour un jeu de rythme, Poll est essentiel pour une latence minimale
    event_loop.set_control_flow(ControlFlow::Poll); 

    let mut app = App::new();

    // Winit 0.30 lance l'application
    let _ = event_loop.run_app(&mut app);
}