use crate::models::engine::hit_window::HitWindow;
use crate::models::engine::NoteData;
use crate::models::menu::GameResultData;
use crate::models::replay::{simulate_replay, ReplayData, ReplayResult};
use crate::models::stats::HitStats;
use crate::views::components::menu::song_select::leaderboard_card::LeaderboardCard;
use egui::{Color32, ScrollArea};

#[derive(Clone)]
pub struct ScoreCard {
    pub timestamp: i64,
    pub rate: f64,
    pub replay_data: ReplayData,
    pub total_notes: usize,
    pub score: i32,
    pub accuracy: f64,
    pub max_combo: i32,
    pub beatmap_hash: String,
    /// Résultat de simulation (recalculé avec la chart cachée).
    pub cached_result: Option<ReplayResult>,
}

impl ScoreCard {
    pub fn from_replay(
        replay: &crate::database::models::Replay,
        total_notes: usize,
    ) -> Option<Self> {
        let replay_data = serde_json::from_str::<ReplayData>(&replay.data)
            .unwrap_or_else(|_| ReplayData::empty());

        Some(ScoreCard {
            timestamp: replay.timestamp,
            rate: replay.rate,
            replay_data,
            total_notes,
            score: replay.score,
            accuracy: replay.accuracy,
            max_combo: replay.max_combo,
            beatmap_hash: replay.beatmap_hash.clone(),
            cached_result: None,
        })
    }

    /// Simule le replay avec la chart et le hit window donnés.
    /// Met à jour le cache de résultat.
    pub fn simulate_with_chart(&mut self, chart: &[NoteData], hit_window: &HitWindow) {
        let result = simulate_replay(&self.replay_data, chart, hit_window);
        self.cached_result = Some(result);
    }
}

pub struct Leaderboard {
    scores: Vec<ScoreCard>,
}

impl Leaderboard {
    pub fn new() -> Self {
        Self { scores: Vec::new() }
    }

    pub fn update_scores(&mut self, scores: Vec<ScoreCard>) {
        self.scores = scores;
    }

    /// Simule tous les replays avec la chart et le hit window donnés.
    pub fn simulate_all(&mut self, chart: &[NoteData], hit_window: &HitWindow) {
        for score in &mut self.scores {
            score.simulate_with_chart(chart, hit_window);
        }
    }

    pub fn render(
        &self,
        ui: &mut egui::Ui,
        _difficulty_name: Option<&str>,
        hit_window: &HitWindow,
        chart: Option<&[NoteData]>,
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
                                // Utiliser le résultat simulé si disponible, sinon recalculer à la volée
                                let (hit_stats, accuracy, replay_result) = if let Some(ref result) = card.cached_result {
                                    (result.hit_stats.clone(), result.accuracy, result.clone())
                                } else if let Some(chart) = chart {
                                    // Simuler à la volée si on a la chart
                                    let result = simulate_replay(&card.replay_data, chart, hit_window);
                                    (result.hit_stats.clone(), result.accuracy, result)
                                } else {
                                    // Fallback: utiliser les données stockées
                                    (HitStats::new(), card.accuracy, ReplayResult::new())
                                };

                                let response = LeaderboardCard::render(
                                    ui,
                                    i,
                                    accuracy,
                                    card.rate,
                                    card.timestamp,
                                    &hit_stats,
                                );

                                if response.clicked() {
                                    let judge_text = "Replay View".to_string();

                                    clicked_result = Some(GameResultData {
                                        hit_stats: hit_stats.clone(),
                                        replay_data: card.replay_data.clone(),
                                        replay_result,
                                        score: card.score as u32,
                                        accuracy,
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
