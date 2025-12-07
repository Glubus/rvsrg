//! Trait implementations for MenuState.

use super::MenuState;
use crate::state::traits::{Snapshot, Transition, Update, UpdateContext};

// MenuState implements Snapshot by cloning itself.
// It's already Arc-wrapped for cheap clones.
impl Snapshot for MenuState {
    type Output = MenuState;

    fn create_snapshot(&self) -> Self::Output {
        self.clone()
    }
}

// MenuState performs cache updates during update().
impl Update for MenuState {
    fn update(&mut self, _dt: f64, _ctx: &mut UpdateContext) -> Option<Transition> {
        // Ensure caches are up-to-date
        self.ensure_selected_rate_cache();
        self.ensure_chart_cache();
        None
    }
}
