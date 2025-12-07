//! Difficulty button configuration.

use crate::models::skin::common::{Color, Vec2Conf};
use serde::{Deserialize, Serialize};

fn default_size() -> Vec2Conf {
    Vec2Conf { x: 300.0, y: 60.0 }
}

fn default_bg_color() -> Color {
    [0.12, 0.12, 0.15, 0.9]
}
fn default_selected_bg_color() -> Color {
    [1.0, 1.0, 0.0, 0.3]
} // Yellow tint
fn default_text_color() -> Color {
    [1.0, 1.0, 1.0, 1.0]
}
fn default_selected_text_color() -> Color {
    [1.0, 1.0, 0.0, 1.0]
} // Yellow

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyButtonConfig {
    #[serde(default = "default_size")]
    pub size: Vec2Conf,

    #[serde(default = "default_bg_color")]
    pub background_color: Color,

    #[serde(default = "default_text_color")]
    pub text_color: Color,

    #[serde(default)]
    pub image: Option<String>,

    #[serde(default = "default_selected_bg_color")]
    pub selected_background_color: Color,

    #[serde(default = "default_selected_text_color")]
    pub selected_text_color: Color,

    #[serde(default)]
    pub selected_image: Option<String>,
}

impl Default for DifficultyButtonConfig {
    fn default() -> Self {
        Self {
            size: default_size(),
            background_color: default_bg_color(),
            text_color: default_text_color(),
            image: None,
            selected_background_color: default_selected_bg_color(),
            selected_text_color: default_selected_text_color(),
            selected_image: None,
        }
    }
}

