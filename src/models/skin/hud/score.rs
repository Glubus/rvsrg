//! Score display configuration.

use crate::models::skin::common::{Color, Vec2Conf};
use serde::{Deserialize, Serialize};

fn default_position() -> Vec2Conf {
    Vec2Conf { x: 960.0, y: 50.0 }
}
fn default_size() -> Vec2Conf {
    Vec2Conf { x: 200.0, y: 40.0 }
}
fn default_color() -> Color {
    [1.0, 1.0, 1.0, 1.0]
} // White
fn default_scale() -> f32 {
    24.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreConfig {
    #[serde(default = "default_position")]
    pub position: Vec2Conf,

    #[serde(default = "default_size")]
    pub size: Vec2Conf,

    #[serde(default = "default_color")]
    pub color: Color,

    #[serde(default = "default_scale")]
    pub scale: f32,

    /// Optional image for score digits (sprite sheet)
    #[serde(default)]
    pub image: Option<String>,

    /// Format string for score display (e.g., "{score:09}")
    #[serde(default = "default_format")]
    pub format: String,

    #[serde(default = "default_true")]
    pub visible: bool,
}

fn default_format() -> String {
    "{score}".into()
}
fn default_true() -> bool {
    true
}

impl Default for ScoreConfig {
    fn default() -> Self {
        Self {
            position: default_position(),
            size: default_size(),
            color: default_color(),
            scale: default_scale(),
            image: None,
            format: default_format(),
            visible: true,
        }
    }
}

