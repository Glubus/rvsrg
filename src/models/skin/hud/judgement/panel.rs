//! Judgement Panel configuration - Separate from Judgement Flash
//! This is the stats display that shows counts (Marvelous: 100, Perfect: 50, etc.)

use crate::models::skin::common::{Color, Vec2Conf};
use serde::{Deserialize, Serialize};

fn default_position() -> Vec2Conf {
    Vec2Conf { x: 50.0, y: 300.0 }
}
fn default_size() -> Vec2Conf {
    Vec2Conf { x: 200.0, y: 200.0 }
}
fn default_text_scale() -> f32 {
    16.0
}

// Default colors for panel (can be different from flash)
fn default_marv_color() -> Color {
    [0.0, 1.0, 1.0, 1.0]
}
fn default_perfect_color() -> Color {
    [1.0, 1.0, 0.0, 1.0]
}
fn default_great_color() -> Color {
    [0.0, 1.0, 0.0, 1.0]
}
fn default_good_color() -> Color {
    [0.0, 0.0, 0.5, 1.0]
}
fn default_bad_color() -> Color {
    [1.0, 0.41, 0.71, 1.0]
}
fn default_miss_color() -> Color {
    [1.0, 0.0, 0.0, 1.0]
}
fn default_ghost_tap_color() -> Color {
    [0.5, 0.5, 0.5, 1.0]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgementPanelConfig {
    #[serde(default = "default_position")]
    pub position: Vec2Conf,

    #[serde(default = "default_size")]
    pub size: Vec2Conf,

    #[serde(default = "default_text_scale")]
    pub text_scale: f32,

    #[serde(default)]
    pub visible: bool,

    // Individual colors for the panel stats
    #[serde(default = "default_marv_color")]
    pub marv_color: Color,

    #[serde(default = "default_perfect_color")]
    pub perfect_color: Color,

    #[serde(default = "default_great_color")]
    pub great_color: Color,

    #[serde(default = "default_good_color")]
    pub good_color: Color,

    #[serde(default = "default_bad_color")]
    pub bad_color: Color,

    #[serde(default = "default_miss_color")]
    pub miss_color: Color,

    #[serde(default = "default_ghost_tap_color")]
    pub ghost_tap_color: Color,
}

impl Default for JudgementPanelConfig {
    fn default() -> Self {
        Self {
            position: default_position(),
            size: default_size(),
            text_scale: default_text_scale(),
            visible: true,
            marv_color: default_marv_color(),
            perfect_color: default_perfect_color(),
            great_color: default_great_color(),
            good_color: default_good_color(),
            bad_color: default_bad_color(),
            miss_color: default_miss_color(),
            ghost_tap_color: default_ghost_tap_color(),
        }
    }
}
