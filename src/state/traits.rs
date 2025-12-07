//! Traits for state management.
//!
//! These traits define the common interface for all game states.

use crate::database::DbManager;
use crate::input::events::GameAction;
use crate::models::settings::SettingsState;
use crate::system::bus::SystemBus;

use super::GameResultData;

/// Context passed to action handlers with shared resources.
pub struct ActionContext<'a> {
    pub db_manager: &'a mut DbManager,
    pub settings: &'a mut SettingsState,
    pub bus: &'a SystemBus,
}

/// Context passed to update methods with shared resources.
pub struct UpdateContext<'a> {
    pub db_manager: &'a mut DbManager,
    pub settings: &'a SettingsState,
    pub bus: &'a SystemBus,
}

/// Transition result from handling an action or update.
#[derive(Debug, Clone)]
pub enum Transition {
    /// Stay in current state.
    None,
    /// Transition to menu state.
    ToMenu,
    /// Transition to gameplay.
    ToGame,
    /// Transition to editor.
    ToEditor,
    /// Transition to result screen with game results.
    ToResult(GameResultData),
    /// Exit the application.
    Exit,
}

/// Trait for creating render-ready snapshots.
///
/// Snapshots are immutable captures of state sent to the render thread.
/// They decouple game logic from rendering.
pub trait Snapshot {
    /// The snapshot type produced.
    type Output;

    /// Creates an immutable snapshot for rendering.
    fn create_snapshot(&self) -> Self::Output;
}

/// Trait for per-frame updates.
///
/// States that need frame-by-frame updates (like gameplay) implement this.
/// Returns an optional transition to another state.
pub trait Update {
    /// Updates the state for one frame.
    ///
    /// # Arguments
    /// * `dt` - Delta time in seconds since last update.
    /// * `ctx` - Context with shared resources (db, settings, bus).
    ///
    /// # Returns
    /// Optional transition to another state.
    fn update(&mut self, dt: f64, ctx: &mut UpdateContext) -> Option<Transition>;
}

/// Trait for handling game actions.
///
/// Each state can handle actions differently and return transitions
/// to other states.
pub trait HandleAction {
    /// Handles a game action and returns any state transition.
    fn handle_action(&mut self, action: &GameAction, ctx: &mut ActionContext) -> Transition;
}
