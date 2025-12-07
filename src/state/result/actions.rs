//! Trait implementations for GameResultData.

use super::GameResultData;
use crate::state::traits::{Snapshot, Transition, Update, UpdateContext};

// GameResultData implements Snapshot by cloning itself.
impl Snapshot for GameResultData {
    type Output = GameResultData;

    fn create_snapshot(&self) -> Self::Output {
        self.clone()
    }
}

// Result screen doesn't need per-frame updates.
impl Update for GameResultData {
    fn update(&mut self, _dt: f64, _ctx: &mut UpdateContext) -> Option<Transition> {
        // Result screen is static - no updates needed.
        None
    }
}
