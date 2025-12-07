//! Song button configuration for song select menu.

use crate::models::skin::common::{Color, Vec2Conf};
use serde::{Deserialize, Serialize};

fn default_position() -> Vec2Conf {
    Vec2Conf { x: 0.0, y: 0.0 }
}
fn default_size() -> Vec2Conf {
    Vec2Conf { x: 400.0, y: 80.0 }
}

fn default_bg_color() -> Color {
    [0.15, 0.15, 0.20, 0.9]
}
fn default_selected_bg_color() -> Color {
    [0.25, 0.35, 0.50, 0.95]
}
fn default_hover_bg_color() -> Color {
    [0.20, 0.25, 0.35, 0.9]
}

fn default_text_color() -> Color {
    [1.0, 1.0, 1.0, 1.0]
}
fn default_selected_text_color() -> Color {
    [1.0, 1.0, 1.0, 1.0]
}

fn default_border_color() -> Color {
    [0.3, 0.3, 0.4, 0.8]
}
fn default_selected_border_color() -> Color {
    [0.4, 0.7, 1.0, 1.0]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongButtonConfig {
    #[serde(default = "default_position")]
    pub position: Vec2Conf,

    #[serde(default = "default_size")]
    pub size: Vec2Conf,

    // Normal state
    #[serde(default = "default_bg_color")]
    pub background_color: Color,

    #[serde(default = "default_text_color")]
    pub text_color: Color,

    #[serde(default = "default_border_color")]
    pub border_color: Color,

    #[serde(default)]
    pub image: Option<String>,

    // Selected state
    #[serde(default = "default_selected_bg_color")]
    pub selected_background_color: Color,

    #[serde(default = "default_selected_text_color")]
    pub selected_text_color: Color,

    #[serde(default = "default_selected_border_color")]
    pub selected_border_color: Color,

    #[serde(default)]
    pub selected_image: Option<String>,

    // Hover state
    #[serde(default = "default_hover_bg_color")]
    pub hover_background_color: Color,
}

impl Default for SongButtonConfig {
    fn default() -> Self {
        Self {
            position: default_position(),
            size: default_size(),
            background_color: default_bg_color(),
            text_color: default_text_color(),
            border_color: default_border_color(),
            image: None,
            selected_background_color: default_selected_bg_color(),
            selected_text_color: default_selected_text_color(),
            selected_border_color: default_selected_border_color(),
            selected_image: None,
            hover_background_color: default_hover_bg_color(),
        }
    }
}
