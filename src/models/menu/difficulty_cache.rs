//! In-memory difficulty cache for on-demand calculation.
//!
//! This cache stores difficulty ratings calculated during the session.
//! Ratings are NOT persisted to the database - they are calculated on-the-fly
//! when a beatmap is selected.

use crate::difficulty::BeatmapSsr;
use ordered_float::OrderedFloat;
use std::collections::HashMap;

/// Key for the difficulty cache: (beatmap_hash, calculator_id, rate)
pub type DifficultyKey = (String, String, OrderedFloat<f64>);

/// In-memory cache for difficulty ratings.
#[derive(Debug, Clone, Default)]
pub struct DifficultyCache {
    /// Cache storage: (beatmap_hash, calculator_id, rate) -> SSR
    cache: HashMap<DifficultyKey, BeatmapSsr>,
    /// Maximum cache size (to prevent unbounded growth)
    max_size: usize,
}

impl DifficultyCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_size: 1000, // Cache up to 1000 entries
        }
    }

    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
        }
    }

    /// Gets a cached rating, if available.
    pub fn get(
        &self,
        beatmap_hash: &str,
        calculator_id: &str,
        rate: f64,
    ) -> Option<&BeatmapSsr> {
        let key = (
            beatmap_hash.to_string(),
            calculator_id.to_string(),
            OrderedFloat(rate),
        );
        self.cache.get(&key)
    }

    /// Stores a rating in the cache.
    pub fn insert(
        &mut self,
        beatmap_hash: &str,
        calculator_id: &str,
        rate: f64,
        ssr: BeatmapSsr,
    ) {
        // Simple eviction: clear half the cache when full
        if self.cache.len() >= self.max_size {
            let keys_to_remove: Vec<_> = self
                .cache
                .keys()
                .take(self.max_size / 2)
                .cloned()
                .collect();
            for key in keys_to_remove {
                self.cache.remove(&key);
            }
        }

        let key = (
            beatmap_hash.to_string(),
            calculator_id.to_string(),
            OrderedFloat(rate),
        );
        self.cache.insert(key, ssr);
    }

    /// Checks if a rating is cached.
    pub fn contains(
        &self,
        beatmap_hash: &str,
        calculator_id: &str,
        rate: f64,
    ) -> bool {
        let key = (
            beatmap_hash.to_string(),
            calculator_id.to_string(),
            OrderedFloat(rate),
        );
        self.cache.contains_key(&key)
    }

    /// Clears all cached ratings.
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Returns the number of cached entries.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Checks if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Gets all cached ratings for a specific beatmap at any rate.
    pub fn get_all_for_beatmap(&self, beatmap_hash: &str) -> Vec<(String, f64, &BeatmapSsr)> {
        self.cache
            .iter()
            .filter(|((hash, _, _), _)| hash == beatmap_hash)
            .map(|((_, calc, rate), ssr)| (calc.clone(), rate.0, ssr))
            .collect()
    }
}


