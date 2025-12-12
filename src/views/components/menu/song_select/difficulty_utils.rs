//! Shared difficulty utilities for song select UI components.

use crate::database::models::BeatmapWithRatings;
use crate::models::skin::menus::song_select::RatingColorsConfig;
use egui::Color32;

/// Converts a skin Color ([f32; 4]) to egui Color32.
pub fn color_to_egui(color: [f32; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (color[0] * 255.0) as u8,
        (color[1] * 255.0) as u8,
        (color[2] * 255.0) as u8,
        (color[3] * 255.0) as u8,
    )
}

/// Returns the appropriate color for a given difficulty rating.
/// Uses thresholds based on Etterna-style difficulty scaling.
pub fn get_difficulty_color(rating: f64, colors: &RatingColorsConfig) -> Color32 {
    match rating {
        r if r < 15.0 => color_to_egui(colors.stream), // Easy (green)
        r if r < 22.0 => color_to_egui(colors.jumpstream), // Normal (orange)
        r if r < 28.0 => color_to_egui(colors.handstream), // Hard (red-orange)
        r if r < 34.0 => color_to_egui(colors.stamina), // Expert (pink)
        _ => color_to_egui(colors.jackspeed),          // Master (purple)
    }
}

/// Computes the difficulty range (min, max) for a set of beatmaps.
/// Uses the first available rating's overall value.
/// Returns None if no beatmaps or no ratings available.
pub fn get_difficulty_range(
    beatmaps: &[BeatmapWithRatings],
    calculator: &str,
) -> Option<(f64, f64)> {
    let ratings: Vec<f64> = beatmaps
        .iter()
        .filter_map(|bm| {
            bm.ratings
                .iter()
                .find(|r| r.name.eq_ignore_ascii_case(calculator))
                .map(|r| r.overall)
        })
        .collect();

    if ratings.is_empty() {
        // Fallback: use any available rating
        let fallback: Vec<f64> = beatmaps
            .iter()
            .filter_map(|bm| bm.ratings.first().map(|r| r.overall))
            .collect();

        if fallback.is_empty() {
            return None;
        }

        let min = fallback.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = fallback.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        return Some((min, max));
    }

    let min = ratings.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = ratings.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    Some((min, max))
}

/// Get the overall rating for a specific beatmap and calculator.
pub fn get_beatmap_rating(beatmap: &BeatmapWithRatings, calculator: &str) -> Option<f64> {
    beatmap
        .ratings
        .iter()
        .find(|r| r.name.eq_ignore_ascii_case(calculator))
        .map(|r| r.overall)
        .or_else(|| beatmap.ratings.first().map(|r| r.overall))
}
