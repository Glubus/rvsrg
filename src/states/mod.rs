mod editor;
mod menu;
mod play;
mod result;

use crate::core::input::actions::KeyAction;
use crate::database::DbManager;
use crate::renderer::Renderer;
use crate::shared::messages::MainToLogic;
use std::sync::mpsc::Sender; // NOUVEAU

pub use editor::EditorStateController;
pub use menu::MenuStateController;
pub use play::PlayStateController;
pub use result::ResultStateController;
use winit::event::WindowEvent;

pub enum StateTransition {
    None,
    Push(Box<dyn GameState>),
    Pop,
    Replace(Box<dyn GameState>),
    Exit,
}

pub struct StateContext {
    renderer: Option<*mut Renderer>,
    db_manager: Option<*mut DbManager>,
    // NOUVEAU : Canal pour parler au cerveau (Logic)
    pub logic_tx: Option<Sender<MainToLogic>>, 
}

impl StateContext {
    // Mise à jour du constructeur
    pub fn new(
        renderer: Option<*mut Renderer>, 
        db_manager: Option<*mut DbManager>,
        logic_tx: Option<Sender<MainToLogic>>
    ) -> Self {
        Self {
            renderer,
            db_manager,
            logic_tx,
        }
    }

    pub fn with_renderer<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut Renderer) -> R,
    {
        if let Some(ptr) = self.renderer {
            unsafe { ptr.as_mut().map(|renderer| f(renderer)) }
        } else {
            None
        }
    }

    pub fn with_db_manager<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut DbManager) -> R,
    {
        if let Some(ptr) = self.db_manager {
            unsafe { ptr.as_mut().map(|db| f(db)) }
        } else {
            None
        }
    }
    
    // NOUVEAU : Helper pour envoyer un message
    pub fn send_to_logic(&self, msg: MainToLogic) {
        if let Some(tx) = &self.logic_tx {
            let _ = tx.send(msg);
        }
    }
}

pub trait GameState {
    fn on_enter(&mut self, _ctx: &mut StateContext) {}
    fn on_exit(&mut self, _ctx: &mut StateContext) {}

    fn handle_input(
        &mut self,
        _event: &WindowEvent,
        _action: Option<KeyAction>,
        _ctx: &mut StateContext,
    ) -> StateTransition {
        StateTransition::None
    }

    fn update(&mut self, _ctx: &mut StateContext) -> StateTransition {
        StateTransition::None
    }

    fn render(&mut self, _ctx: &mut StateContext) -> StateTransition {
        StateTransition::None
    }
}