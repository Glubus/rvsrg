use crate::database::models::Replay;
use crate::database::{BeatmapRating, BeatmapWithRatings, Beatmapset, Database};
use crate::difficulty;
use crate::models::replay::ReplayData;
use crate::models::search::MenuSearchFilters;
use crate::models::stats::HitStats;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct GameResultData {
    pub hit_stats: HitStats,
    pub replay_data: ReplayData,
    pub score: u32,
    pub accuracy: f64,
    pub max_combo: u32,
    pub beatmap_hash: Option<String>,
    pub rate: f64,
    pub judge_text: String,
}

#[derive(Clone, Debug)]
pub struct MenuState {
    pub beatmapsets: Vec<(Beatmapset, Vec<BeatmapWithRatings>)>,
    pub start_index: usize,
    pub end_index: usize,
    pub selected_index: usize,
    pub selected_difficulty_index: usize,
    pub visible_count: usize,
    pub in_menu: bool,
    pub in_editor: bool,
    pub show_result: bool,
    // NOUVEAU : Source de vérité pour l'affichage des settings
    pub show_settings: bool,
    pub rate: f64,
    pub last_result: Option<GameResultData>,
    pub should_close_result: bool,
    pub rate_cache: HashMap<String, RateCacheEntry>,
    pub search_filters: MenuSearchFilters,
    pub leaderboard_scores: Vec<Replay>,
    pub leaderboard_hash: Option<String>,
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            beatmapsets: Vec::new(),
            start_index: 0,
            end_index: 0,
            selected_index: 0,
            selected_difficulty_index: 0,
            visible_count: 10,
            in_menu: true,
            in_editor: false,
            show_result: false,
            show_settings: false, // Par défaut fermé
            rate: 1.0,
            last_result: None,
            should_close_result: false,
            rate_cache: HashMap::new(),
            search_filters: MenuSearchFilters::default(),
            leaderboard_scores: Vec::new(),
            leaderboard_hash: None,
        }
    }

    pub fn increase_rate(&mut self) {
        let next_rate = {
            let current = self.rate;
            self.ensure_selected_rate_entry()
                .and_then(|entry| entry.next_rate(current))
        };
        if let Some(rate) = next_rate {
            self.rate = rate;
            return;
        }
        self.rate = (self.rate + 0.1).min(2.0);
    }

    pub fn decrease_rate(&mut self) {
        let previous_rate = {
            let current = self.rate;
            self.ensure_selected_rate_entry()
                .and_then(|entry| entry.previous_rate(current))
        };
        if let Some(rate) = previous_rate {
            self.rate = rate;
            return;
        }
        self.rate = (self.rate - 0.1).max(0.5);
    }

    pub fn ensure_selected_rate_cache(&mut self) {
        let _ = self.ensure_selected_rate_entry();
    }

    pub fn get_cached_ratings_for(
        &self,
        beatmap_hash: &str,
        rate: f64,
    ) -> Option<&[BeatmapRating]> {
        self.rate_cache
            .get(beatmap_hash)
            .and_then(|entry| entry.get_ratings(rate))
            .map(|list| list.as_slice())
    }

    fn ensure_selected_rate_entry(&mut self) -> Option<&RateCacheEntry> {
        let selected = self.get_selected_beatmap()?;
        let beatmap_hash = selected.beatmap.hash.clone();
        let beatmap_path = selected.beatmap.path.clone();

        if !self.rate_cache.contains_key(&beatmap_hash) {
            let map = match rosu_map::Beatmap::from_path(&beatmap_path) {
                Ok(map) => map,
                Err(err) => {
                    log::error!(
                        "MENU: Impossible de charger la beatmap {} pour calculer les rates: {}",
                        beatmap_hash,
                        err
                    );
                    return None;
                }
            };

            match difficulty::analyze_all_rates(&map) {
                Ok(rate_data) => {
                    let entry = RateCacheEntry::from_analysis(&beatmap_hash, rate_data);
                    let adjusted_rate = entry.closest_rate(self.rate);
                    if let Some(rate) = adjusted_rate {
                        self.rate = rate;
                    }
                    self.rate_cache.insert(beatmap_hash.clone(), entry);
                }
                Err(err) => {
                    log::error!(
                        "MENU: Impossible de calculer les rates pour {}: {}",
                        beatmap_hash,
                        err
                    );
                    return None;
                }
            }
        } else if let Some(entry) = self.rate_cache.get(&beatmap_hash) {
            if !entry.contains_rate(self.rate) {
                if let Some(rate) = entry.closest_rate(self.rate) {
                    self.rate = rate;
                }
            }
        }

        self.rate_cache.get(&beatmap_hash)
    }

    pub async fn load_from_db(
        menu_state: Arc<Mutex<Self>>,
        db: &Database,
    ) -> Result<(), sqlx::Error> {
        let beatmapsets = db.get_all_beatmapsets().await?;
        if let Ok(mut state) = menu_state.lock() {
            state.beatmapsets = beatmapsets.clone();
            state.selected_index = 0;
            state.selected_difficulty_index = 0;
            state.end_index = state.visible_count.min(state.beatmapsets.len());
            state.start_index = 0;
            state.rate_cache.clear();
            state.rate = 1.0;
            state.search_filters = MenuSearchFilters::default();
            state.leaderboard_scores.clear();
            state.leaderboard_hash = None;
        }
        Ok(())
    }

    pub fn get_visible_items(&self) -> &[(Beatmapset, Vec<BeatmapWithRatings>)] {
        if self.start_index >= self.beatmapsets.len() {
            return &[];
        }
        let end = self.end_index.min(self.beatmapsets.len());
        &self.beatmapsets[self.start_index..end]
    }

    pub fn get_relative_selected_index(&self) -> usize {
        if self.selected_index < self.start_index {
            0
        } else if self.selected_index >= self.end_index {
            self.end_index
                .saturating_sub(self.start_index)
                .saturating_sub(1)
        } else {
            self.selected_index - self.start_index
        }
    }

    pub fn move_up(&mut self) {
        if self.beatmapsets.is_empty() {
            return;
        }

        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.selected_difficulty_index = 0;

            if self.selected_index < self.start_index {
                self.start_index = self.selected_index;
                self.end_index =
                    (self.start_index + self.visible_count).min(self.beatmapsets.len());
            }
        }
    }

    pub fn move_down(&mut self) {
        if self.beatmapsets.is_empty() {
            return;
        }

        if self.selected_index < self.beatmapsets.len() - 1 {
            self.selected_index += 1;
            self.selected_difficulty_index = 0;

            if self.selected_index >= self.end_index {
                self.end_index = (self.selected_index + 1).min(self.beatmapsets.len());
                self.start_index = self.end_index.saturating_sub(self.visible_count);
            }
        }
    }

    pub fn get_selected_beatmapset(&self) -> Option<&(Beatmapset, Vec<BeatmapWithRatings>)> {
        self.beatmapsets.get(self.selected_index)
    }

    fn get_selected_beatmap(&self) -> Option<&BeatmapWithRatings> {
        self.get_selected_beatmapset().and_then(|(_, beatmaps)| {
            let idx = self
                .selected_difficulty_index
                .min(beatmaps.len().saturating_sub(1));
            beatmaps.get(idx)
        })
    }

    pub fn get_selected_beatmap_path(&self) -> Option<PathBuf> {
        self.get_selected_beatmap()
            .map(|bm| PathBuf::from(&bm.beatmap.path))
    }

    pub fn next_difficulty(&mut self) {
        if let Some((_, beatmaps)) = self.get_selected_beatmapset() {
            if beatmaps.is_empty() {
                self.selected_difficulty_index = 0;
            } else {
                self.selected_difficulty_index =
                    (self.selected_difficulty_index + 1) % beatmaps.len();
            }
        }
    }

    pub fn previous_difficulty(&mut self) {
        if let Some((_, beatmaps)) = self.get_selected_beatmapset() {
            if beatmaps.is_empty() {
                self.selected_difficulty_index = 0;
            } else if self.selected_difficulty_index == 0 {
                self.selected_difficulty_index = beatmaps.len() - 1;
            } else {
                self.selected_difficulty_index -= 1;
            }
        }
    }

    pub fn get_selected_difficulty_name(&self) -> Option<String> {
        self.get_selected_beatmap()
            .and_then(|bm| bm.beatmap.difficulty_name.clone())
    }

    pub fn get_selected_beatmap_hash(&self) -> Option<String> {
        self.get_selected_beatmap()
            .map(|bm| bm.beatmap.hash.clone())
    }

    pub fn set_leaderboard(&mut self, hash: Option<String>, scores: Vec<Replay>) {
        self.leaderboard_hash = hash;
        self.leaderboard_scores = scores;
    }
}

#[derive(Clone, Debug)]
pub struct RateCacheEntry {
    available_rates: Vec<f64>,
    ratings_by_rate: HashMap<i32, Vec<BeatmapRating>>,
}

impl RateCacheEntry {
    fn from_analysis(beatmap_hash: &str, analysis: difficulty::RateDifficultyCache) -> Self {
        let mut ratings_by_rate = HashMap::new();

        for (rate, values) in analysis.ratings_by_rate.into_iter() {
            let key = Self::normalize(rate);
            let converted = values
                .into_iter()
                .enumerate()
                .map(|(idx, value)| BeatmapRating {
                    id: -((idx as i64) + 1),
                    beatmap_hash: beatmap_hash.to_string(),
                    name: value.name,
                    overall: value.ssr.overall,
                    stream: value.ssr.stream,
                    jumpstream: value.ssr.jumpstream,
                    handstream: value.ssr.handstream,
                    stamina: value.ssr.stamina,
                    jackspeed: value.ssr.jackspeed,
                    chordjack: value.ssr.chordjack,
                    technical: value.ssr.technical,
                })
                .collect::<Vec<_>>();
            ratings_by_rate.insert(key, converted);
        }

        Self {
            available_rates: analysis.available_rates,
            ratings_by_rate,
        }
    }

    fn get_ratings(&self, rate: f64) -> Option<&Vec<BeatmapRating>> {
        let key = Self::normalize(rate);
        self.ratings_by_rate.get(&key)
    }

    fn contains_rate(&self, rate: f64) -> bool {
        self.get_ratings(rate).is_some()
    }

    fn closest_rate(&self, desired: f64) -> Option<f64> {
        if self.available_rates.is_empty() {
            return None;
        }
        let mut best_rate = self.available_rates[0];
        let mut best_diff = (best_rate - desired).abs();
        for &candidate in &self.available_rates {
            let diff = (candidate - desired).abs();
            if diff < best_diff {
                best_rate = candidate;
                best_diff = diff;
            }
        }
        Some(best_rate)
    }

    fn next_rate(&self, current: f64) -> Option<f64> {
        let epsilon = 0.0001;
        self.available_rates
            .iter()
            .copied()
            .find(|&rate| rate > current + epsilon)
    }

    fn previous_rate(&self, current: f64) -> Option<f64> {
        let epsilon = 0.0001;
        self.available_rates
            .iter()
            .rev()
            .copied()
            .find(|&rate| rate < current - epsilon)
    }

    fn normalize(rate: f64) -> i32 {
        (rate * 100.0).round() as i32
    }
}
