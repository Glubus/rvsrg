//! Leaderboard panel configuration.

use crate::models::skin::common::{Color, Vec2Conf};
use serde::{Deserialize, Serialize};

fn default_position() -> Vec2Conf {
    Vec2Conf { x: 0.0, y: 0.0 }
}
fn default_size() -> Vec2Conf {
    Vec2Conf { x: 350.0, y: 400.0 }
}
fn default_bg_color() -> Color {
    [0.08, 0.08, 0.10, 0.95]
}
fn default_text_color() -> Color {
    [1.0, 1.0, 1.0, 1.0]
}
fn default_entry_bg_color() -> Color {
    [0.12, 0.12, 0.15, 0.9]
}
fn default_entry_selected_color() -> Color {
    [0.2, 0.3, 0.45, 0.95]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardConfig {
    #[serde(default = "default_position")]
    pub position: Vec2Conf,

    #[serde(default = "default_size")]
    pub size: Vec2Conf,

    #[serde(default = "default_bg_color")]
    pub background_color: Color,

    #[serde(default = "default_text_color")]
    pub text_color: Color,

    #[serde(default = "default_entry_bg_color")]
    pub entry_background_color: Color,

    #[serde(default = "default_entry_selected_color")]
    pub entry_selected_color: Color,

    #[serde(default)]
    pub background_image: Option<String>,
}

impl Default for LeaderboardConfig {
    fn default() -> Self {
        Self {
            position: default_position(),
            size: default_size(),
            background_color: default_bg_color(),
            text_color: default_text_color(),
            entry_background_color: default_entry_bg_color(),
            entry_selected_color: default_entry_selected_color(),
            background_image: None,
        }
    }
}
