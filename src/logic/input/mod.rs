pub mod editor;
pub mod game;
pub mod menu;

use std::sync::mpsc::Sender;
use crate::core::input::actions::KeyAction;
use crate::database::DbManager;
use crate::models::engine::GameEngine;
use crate::models::menu::MenuState;
use crate::shared::messages::LogicToMain;

// Le contexte contient tout ce dont un handler a besoin pour travailler
pub struct InputContext<'a> {
    pub menu_state: &'a mut MenuState,
    pub game_engine: &'a mut Option<GameEngine>,
    pub db_manager: &'a DbManager,
    pub tx: &'a Sender<LogicToMain>,
}

pub trait InputHandler {
    // Retourne true si l'input a été consommé/traité
    fn handle(&mut self, action: KeyAction, ctx: &mut InputContext) -> bool;
}