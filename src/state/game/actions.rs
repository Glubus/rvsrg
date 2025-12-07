//! Trait implementations for GameEngine.

use std::time::{SystemTime, UNIX_EPOCH};

use super::GameEngine;
use crate::database::SaveReplayCommand;
use crate::models::replay::simulate_replay;
use crate::models::settings::HitWindowMode;
use crate::shared::snapshot::GameplaySnapshot;
use crate::state::GameResultData;
use crate::state::traits::{Snapshot, Transition, Update, UpdateContext};

// GameEngine implements Snapshot by creating a GameplaySnapshot.
impl Snapshot for GameEngine {
    type Output = GameplaySnapshot;

    fn create_snapshot(&self) -> Self::Output {
        self.get_snapshot()
    }
}

// GameEngine needs per-frame updates for gameplay timing.
// When the game is finished, it builds the result and returns a transition.
impl Update for GameEngine {
    fn update(&mut self, dt: f64, ctx: &mut UpdateContext) -> Option<Transition> {
        // Run the core gameplay update
        GameEngine::update(self, dt);

        // Check if game is finished
        if !self.is_finished() {
            return None;
        }

        // Game finished - build results and save replay
        let chart = self.get_chart();
        let replay_result = simulate_replay(&self.replay_data, &chart, &self.hit_window);
        let accuracy = replay_result.accuracy;

        // Save replay to database
        if let Some(payload) = build_replay_payload(self, accuracy) {
            ctx.db_manager.save_replay(payload);
        }

        // Format judge text from settings
        let judge_text =
            format_hit_window_text(ctx.settings.hit_window_mode, ctx.settings.hit_window_value);

        // Build result data
        let result = GameResultData {
            hit_stats: replay_result.hit_stats.clone(),
            replay_data: self.replay_data.clone(),
            replay_result,
            score: self.score,
            accuracy,
            max_combo: self.max_combo,
            beatmap_hash: self.beatmap_hash.clone(),
            rate: self.rate,
            judge_text,
            show_settings: false,
        };

        Some(Transition::ToResult(result))
    }
}

/// Formats the hit window mode and value as a display string.
fn format_hit_window_text(mode: HitWindowMode, value: f64) -> String {
    match mode {
        HitWindowMode::OsuOD => format!("OD {:.1}", value),
        HitWindowMode::EtternaJudge => format!("Judge {:.0}", value),
    }
}

/// Converts gameplay stats into a DB command for replay persistence.
fn build_replay_payload(engine: &GameEngine, accuracy: f64) -> Option<SaveReplayCommand> {
    let hash = match engine.beatmap_hash.clone() {
        Some(h) => h,
        None => {
            log::error!("REPLAY: Cannot save - beatmap_hash is None!");
            return None;
        }
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    Some(SaveReplayCommand {
        beatmap_hash: hash,
        timestamp,
        score: engine.score.min(i32::MAX as u32) as i32,
        accuracy,
        max_combo: engine.max_combo.min(i32::MAX as u32) as i32,
        rate: engine.rate,
        data: engine.replay_data.clone(),
    })
}
