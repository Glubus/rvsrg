use egui::{Color32, ScrollArea};

use crate::models::stats::HitStats;
use crate::views::components::menu::song_select::leaderboard_card::LeaderboardCard;

#[derive(Clone)]
pub struct ScoreCard {
    pub accuracy: f64,
    pub timestamp: i64,
    pub rate: f64,
    pub hit_stats: HitStats,
}

impl ScoreCard {
    pub fn from_replay(replay: &crate::database::models::Replay) -> Option<Self> {
        let hit_stats = if let Ok(replay_data) = serde_json::from_str::<crate::models::replay::ReplayData>(&replay.data) {
            replay_data.hit_stats.unwrap_or_else(HitStats::new)
        } else {
            HitStats::new()
        };
        
        Some(ScoreCard {
            accuracy: replay.accuracy,
            timestamp: replay.timestamp,
            rate: replay.rate,
            hit_stats,
        })
    }
}

pub struct Leaderboard {
    scores: Vec<ScoreCard>,
}

impl Leaderboard {
    pub fn new() -> Self {
        Self {
            scores: Vec::new(),
        }
    }

    pub fn update_scores(&mut self, scores: Vec<ScoreCard>) {
        self.scores = scores;
    }

    pub fn render(&self, ui: &mut egui::Ui) {
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
                                LeaderboardCard::render(
                                    ui,
                                    i,
                                    card.accuracy,
                                    card.rate,
                                    card.timestamp,
                                    &card.hit_stats,
                                );
                                
                                if i < self.scores.len().min(10).saturating_sub(1) {
                                    ui.add_space(5.0);
                                }
                            }
                        });
                }
            });
    }
}

