//! Search bar configuration.

use crate::models::skin::common::{Color, Vec2Conf};
use serde::{Deserialize, Serialize};

fn default_position() -> Vec2Conf {
    Vec2Conf { x: 0.0, y: 0.0 }
}
fn default_size() -> Vec2Conf {
    Vec2Conf { x: 300.0, y: 40.0 }
}

fn default_bg_color() -> Color {
    [0.1, 0.1, 0.12, 0.95]
}
fn default_active_bg_color() -> Color {
    [0.12, 0.15, 0.20, 0.98]
}
fn default_text_color() -> Color {
    [1.0, 1.0, 1.0, 1.0]
}
fn default_placeholder_color() -> Color {
    [0.5, 0.5, 0.55, 1.0]
}
fn default_border_color() -> Color {
    [0.25, 0.25, 0.30, 0.8]
}
fn default_active_border_color() -> Color {
    [0.3, 0.75, 0.95, 1.0]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchBarConfig {
    #[serde(default = "default_position")]
    pub position: Vec2Conf,

    #[serde(default = "default_size")]
    pub size: Vec2Conf,

    #[serde(default = "default_bg_color")]
    pub background_color: Color,

    #[serde(default = "default_active_bg_color")]
    pub active_background_color: Color,

    #[serde(default = "default_text_color")]
    pub text_color: Color,

    #[serde(default = "default_placeholder_color")]
    pub placeholder_color: Color,

    #[serde(default = "default_border_color")]
    pub border_color: Color,

    #[serde(default = "default_active_border_color")]
    pub active_border_color: Color,

    #[serde(default)]
    pub image: Option<String>,

    #[serde(default)]
    pub active_image: Option<String>,
}

impl Default for SearchBarConfig {
    fn default() -> Self {
        Self {
            position: default_position(),
            size: default_size(),
            background_color: default_bg_color(),
            active_background_color: default_active_bg_color(),
            text_color: default_text_color(),
            placeholder_color: default_placeholder_color(),
            border_color: default_border_color(),
            active_border_color: default_active_border_color(),
            image: None,
            active_image: None,
        }
    }
}

