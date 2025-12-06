//! Beatmap info panel configuration.

use crate::models::skin::common::{Color, Vec2Conf};
use serde::{Deserialize, Serialize};

fn default_position() -> Vec2Conf {
    Vec2Conf { x: 0.0, y: 0.0 }
}
fn default_size() -> Vec2Conf {
    Vec2Conf { x: 400.0, y: 300.0 }
}
fn default_bg_color() -> Color {
    [0.08, 0.08, 0.10, 0.95]
}
fn default_text_color() -> Color {
    [1.0, 1.0, 1.0, 1.0]
}
fn default_secondary_text_color() -> Color {
    [0.75, 0.75, 0.80, 1.0]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatmapInfoConfig {
    #[serde(default = "default_position")]
    pub position: Vec2Conf,

    #[serde(default = "default_size")]
    pub size: Vec2Conf,

    #[serde(default = "default_bg_color")]
    pub background_color: Color,

    #[serde(default = "default_text_color")]
    pub text_color: Color,

    #[serde(default = "default_secondary_text_color")]
    pub secondary_text_color: Color,

    #[serde(default)]
    pub background_image: Option<String>,
}

impl Default for BeatmapInfoConfig {
    fn default() -> Self {
        Self {
            position: default_position(),
            size: default_size(),
            background_color: default_bg_color(),
            text_color: default_text_color(),
            secondary_text_color: default_secondary_text_color(),
            background_image: None,
        }
    }
}
