mod editor;
mod menu;
mod play;
mod result;

use crate::database::DbManager;
use crate::renderer::Renderer;
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
}

impl StateContext {
    pub fn new(renderer: Option<*mut Renderer>, db_manager: Option<*mut DbManager>) -> Self {
        Self {
            renderer,
            db_manager,
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
}

pub trait GameState {
    fn on_enter(&mut self, _ctx: &mut StateContext) {}
    fn on_exit(&mut self, _ctx: &mut StateContext) {}

    fn handle_input(&mut self, _event: &WindowEvent, _ctx: &mut StateContext) -> StateTransition {
        StateTransition::None
    }

    fn update(&mut self, _ctx: &mut StateContext) -> StateTransition {
        StateTransition::None
    }

    fn render(&mut self, _ctx: &mut StateContext) -> StateTransition {
        StateTransition::None
    }
}
