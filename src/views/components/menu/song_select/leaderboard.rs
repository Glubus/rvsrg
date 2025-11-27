//! Leaderboard list plus helper `ScoreCard` conversions.

use crate::models::engine::hit_window::HitWindow;
use crate::models::menu::GameResultData;
use crate::models::replay::ReplayData;
use crate::views::components::menu::song_select::leaderboard_card::LeaderboardCard;
use egui::{Color32, ScrollArea};

#[derive(Clone)]
/// Lightweight view model built from DB replay rows.
pub struct ScoreCard {
    pub timestamp: i64,
    pub rate: f64,
    pub replay_data: ReplayData,
    pub total_notes: usize,
    pub score: i32,
    pub accuracy: f64,
    pub max_combo: i32,
    pub beatmap_hash: String,
}

impl ScoreCard {
    /// Builds a score card from a replay row, falling back to an empty replay if parsing fails.
    pub fn from_replay(
        replay: &crate::database::models::Replay,
        total_notes: usize,
    ) -> Option<Self> {
        let replay_data = if let Ok(data) = serde_json::from_str::<ReplayData>(&replay.data) {
            data
        } else {
            ReplayData::new()
        };

        Some(ScoreCard {
            timestamp: replay.timestamp,
            rate: replay.rate,
            replay_data,
            total_notes,
            score: replay.score,
            accuracy: replay.accuracy,
            max_combo: replay.max_combo,
            beatmap_hash: replay.beatmap_hash.clone(),
        })
    }
}

/// Leaderboard UI state that caches up to 10 score cards.
pub struct Leaderboard {
    scores: Vec<ScoreCard>,
}

impl Leaderboard {
    /// Returns an empty leaderboard.
    pub fn new() -> Self {
        Self { scores: Vec::new() }
    }

    /// Replaces the leaderboard entries.
    pub fn update_scores(&mut self, scores: Vec<ScoreCard>) {
        self.scores = scores;
    }

    /// Renders the leaderboard and returns a replay to preview if the user clicked a row.
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        _difficulty_name: Option<&str>,
        hit_window: &HitWindow,
    ) -> Option<GameResultData> {
        let mut clicked_result = None;

        egui::Frame::default()
            .corner_radius(5.0)
            .outer_margin(10.0)
            .inner_margin(5.0)
            .fill(Color32::from_rgba_unmultiplied(38, 38, 38, 230))
            .show(ui, |ui| {
                ui.set_width(ui.available_rect_before_wrap().width());
                ui.set_height(ui.available_rect_before_wrap().height());

                ui.heading("Top Scores");
                ui.separator();

                if self.scores.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label("No Score Set");
                    });
                } else {
                    ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            for (i, card) in self.scores.iter().take(10).enumerate() {
                                let (hit_stats, accuracy) =
                                    crate::models::replay::recalculate_accuracy_with_hit_window(
                                        &card.replay_data,
                                        card.total_notes,
                                        hit_window,
                                    );

                                let response = LeaderboardCard::render(
                                    ui,
                                    i,
                                    accuracy,
                                    card.rate,
                                    card.timestamp,
                                    &hit_stats,
                                );

                                if response.clicked() {
                                    // Derive a textual description for the current hit window.
                                    // We do not have HitWindowMode here, so reuse a generic label.
                                    let judge_text = "Replay View".to_string();

                                    clicked_result = Some(GameResultData {
                                        hit_stats,
                                        replay_data: card.replay_data.clone(),
                                        score: card.score as u32,
                                        accuracy: accuracy,
                                        max_combo: card.max_combo as u32,
                                        beatmap_hash: Some(card.beatmap_hash.clone()),
                                        rate: card.rate,
                                        judge_text,
                                    });
                                }

                                if i < self.scores.len().min(10).saturating_sub(1) {
                                    ui.add_space(5.0);
                                }
                            }
                        });
                }
            });

        clicked_result
    }
}
