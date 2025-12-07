//! Main menu state management.
//!
//! This module handles the song select menu state including beatmap selection,
//! rate changes, difficulty caching, and leaderboard display.
//!
//! ## Architecture
//!
//! - Beatmaps are loaded via pagination (50 items at a time)
//! - Difficulty ratings are calculated ON-DEMAND when a map is selected
//! - Ratings are cached in memory (not DB) for the session



use super::{ChartCache, DifficultyCache, RateCacheEntry};
use crate::database::models::Replay;
use crate::database::{BeatmapRating, BeatmapWithRatings, Beatmapset, Database};
use crate::difficulty::{self, BeatmapSsr};
use crate::models::search::MenuSearchFilters;
use crate::views::components::menu::song_select::CalculatorOption;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::GameResultData;

/// Constants for pagination
pub const PAGE_SIZE: usize = 50;
pub const PRELOAD_MARGIN: usize = 10;

/// Main state for the song selection menu.
#[derive(Clone, Debug)]
pub struct MenuState {
    // Current beatmapsets (loaded via pagination or full load for backwards compat)
    pub beatmapsets: Vec<(Beatmapset, Vec<BeatmapWithRatings>)>,

    // Pagination state
    pub total_count: usize,
    pub current_offset: usize,

    // Selection state
    pub start_index: usize,
    pub end_index: usize,
    pub selected_index: usize,
    pub selected_difficulty_index: usize,
    pub visible_count: usize,

    // UI state
    pub in_menu: bool,
    pub in_editor: bool,
    pub show_result: bool,
    pub show_settings: bool,

    // Playback rate
    pub rate: f64,

    // Result screen
    pub last_result: Option<GameResultData>,
    pub should_close_result: bool,

    // Rate cache (available rates per beatmap)
    pub rate_cache: HashMap<String, RateCacheEntry>,

    // On-demand difficulty cache (in RAM only!)
    pub difficulty_cache: DifficultyCache,

    // Active difficulty calculator
    pub active_calculator: String,

    // Available calculators (builtin + custom)
    pub available_calculators: Vec<CalculatorOption>,

    // Search/filter
    pub search_filters: MenuSearchFilters,

    // Leaderboard
    pub leaderboard_scores: Vec<Replay>,
    pub leaderboard_hash: Option<String>,

    // Chart cache for gameplay
    pub chart_cache: Option<ChartCache>,
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            beatmapsets: Vec::new(),
            total_count: 0,
            current_offset: 0,
            start_index: 0,
            end_index: 0,
            selected_index: 0,
            selected_difficulty_index: 0,
            visible_count: 10,
            in_menu: true,
            in_editor: false,
            show_result: false,
            show_settings: false,
            rate: 1.0,
            last_result: None,
            should_close_result: false,
            rate_cache: HashMap::new(),
            difficulty_cache: DifficultyCache::new(),
            active_calculator: "etterna".to_string(),
            available_calculators: vec![
                CalculatorOption::new("etterna", "Etterna"),
                CalculatorOption::new("osu", "osu!"),
            ],
            search_filters: MenuSearchFilters::default(),
            leaderboard_scores: Vec::new(),
            leaderboard_hash: None,
            chart_cache: None,
        }
    }

    /// Loads the currently selected beatmap's chart into cache.
    ///
    /// Returns `true` if a new chart was loaded, `false` if already cached.
    pub fn ensure_chart_cache(&mut self) -> bool {
        let selected = match self.get_selected_beatmap() {
            Some(bm) => bm,
            None => return false,
        };

        let beatmap_hash = selected.beatmap.hash.clone();
        let beatmap_path = PathBuf::from(&selected.beatmap.path);

        if let Some(ref cache) = self.chart_cache
            && cache.beatmap_hash == beatmap_hash
        {
            return false;
        }

        match crate::models::engine::load_map_safe(&beatmap_path) {
            Some((audio_path, chart)) => {
                log::info!(
                    "MENU: Chart cached for {} ({} notes)",
                    beatmap_hash,
                    chart.len()
                );
                self.chart_cache = Some(ChartCache {
                    beatmap_hash,
                    chart,
                    audio_path,
                    map_path: beatmap_path,
                });
                true
            }
            None => {
                log::error!("MENU: Failed to load chart for caching");
                self.chart_cache = None;
                false
            }
        }
    }

    pub fn get_cached_chart(&self) -> Option<&ChartCache> {
        self.chart_cache.as_ref()
    }

    pub fn get_cached_chart_note_count(&self) -> usize {
        self.chart_cache
            .as_ref()
            .map(|c| c.chart.len())
            .unwrap_or(0)
    }

    /// Calculates difficulty for the currently selected beatmap on-demand.
    /// Results are cached in memory (not DB).
    pub fn ensure_difficulty_calculated(&mut self) -> Option<BeatmapSsr> {
        let selected = self.get_selected_beatmap()?;
        let beatmap_hash = selected.beatmap.hash.clone();
        let beatmap_path = selected.beatmap.path.clone();
        let calculator = self.active_calculator.clone();
        let rate = self.rate;

        // Check cache first
        if let Some(cached) = self.difficulty_cache.get(&beatmap_hash, &calculator, rate) {
            return Some(cached.clone());
        }

        // Load and calculate
        let map = match rosu_map::Beatmap::from_path(&beatmap_path) {
            Ok(map) => map,
            Err(err) => {
                log::error!("MENU: Failed to load beatmap for difficulty calc: {}", err);
                return None;
            }
        };

        match difficulty::calculate_on_demand(&map, &calculator, rate) {
            Ok(ssr) => {
                self.difficulty_cache
                    .insert(&beatmap_hash, &calculator, rate, ssr.clone());
                Some(ssr)
            }
            Err(err) => {
                log::error!("MENU: Failed to calculate difficulty: {}", err);
                None
            }
        }
    }

    /// Gets the cached difficulty for the selected beatmap at the current rate.
    pub fn get_current_difficulty(&self) -> Option<&BeatmapSsr> {
        let selected = self.get_selected_beatmap()?;
        self.difficulty_cache
            .get(&selected.beatmap.hash, &self.active_calculator, self.rate)
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
                        "MENU: Failed to load beatmap {} to compute rates: {}",
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
                        "MENU: Unable to compute rates for {}: {}",
                        beatmap_hash,
                        err
                    );
                    return None;
                }
            }
        } else if let Some(entry) = self.rate_cache.get(&beatmap_hash)
            && !entry.contains_rate(self.rate)
            && let Some(rate) = entry.closest_rate(self.rate)
        {
            self.rate = rate;
        }

        self.rate_cache.get(&beatmap_hash)
    }

    pub async fn load_from_db(
        menu_state: Arc<Mutex<Self>>,
        db: &Database,
    ) -> Result<(), sqlx::Error> {
        let beatmapsets = db.get_all_beatmapsets().await?;
        let total_count = beatmapsets.len();

        if let Ok(mut state) = menu_state.lock() {
            state.beatmapsets = beatmapsets;
            state.total_count = total_count;
            state.current_offset = 0;
            state.selected_index = 0;
            state.selected_difficulty_index = 0;
            state.end_index = state.visible_count.min(state.beatmapsets.len());
            state.start_index = 0;
            state.rate_cache.clear();
            state.difficulty_cache.clear();
            state.rate = 1.0;
            state.search_filters = MenuSearchFilters::default();
            state.leaderboard_scores.clear();
            state.leaderboard_hash = None;
            state.chart_cache = None;
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

    /// Sets the active difficulty calculator.
    pub fn set_calculator(&mut self, calculator_id: &str) {
        if self.active_calculator != calculator_id {
            self.active_calculator = calculator_id.to_string();
            // Clear difficulty cache when changing calculator
            // (rate cache is independent of calculator)
        }
    }

    /// Gets the available calculators.
    pub fn available_calculators(&self) -> Vec<(&'static str, &'static str)> {
        vec![("etterna", "Etterna (MinaCalc)"), ("osu", "osu! (rosu-pp)")]
    }
}

