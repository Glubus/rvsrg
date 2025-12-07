//! Cache des ratings par rate.

use crate::database::BeatmapRating;
use crate::difficulty;
use std::collections::HashMap;

/// Cache des ratings calculés pour différents rates d'une beatmap.
#[derive(Clone, Debug)]
pub struct RateCacheEntry {
    available_rates: Vec<f64>,
    ratings_by_rate: HashMap<i32, Vec<BeatmapRating>>,
}

impl RateCacheEntry {
    pub fn from_analysis(beatmap_hash: &str, analysis: difficulty::RateDifficultyCache) -> Self {
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

    pub fn get_ratings(&self, rate: f64) -> Option<&Vec<BeatmapRating>> {
        let key = Self::normalize(rate);
        self.ratings_by_rate.get(&key)
    }

    pub fn contains_rate(&self, rate: f64) -> bool {
        self.get_ratings(rate).is_some()
    }

    pub fn closest_rate(&self, desired: f64) -> Option<f64> {
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

    pub fn next_rate(&self, current: f64) -> Option<f64> {
        let epsilon = 0.0001;
        self.available_rates
            .iter()
            .copied()
            .find(|&rate| rate > current + epsilon)
    }

    pub fn previous_rate(&self, current: f64) -> Option<f64> {
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
