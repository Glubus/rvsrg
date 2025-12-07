//! NPS (Notes Per Second) display configuration.

use crate::models::skin::common::{Color, Vec2Conf};
use serde::{Deserialize, Serialize};

fn default_position() -> Vec2Conf {
    Vec2Conf { x: 1150.0, y: 50.0 }
}
fn default_size() -> Vec2Conf {
    Vec2Conf { x: 100.0, y: 30.0 }
}
fn default_color() -> Color {
    [1.0, 1.0, 1.0, 1.0]
} // White
fn default_scale() -> f32 {
    20.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpsConfig {
    #[serde(default = "default_position")]
    pub position: Vec2Conf,

    #[serde(default = "default_size")]
    pub size: Vec2Conf,

    #[serde(default = "default_color")]
    pub color: Color,

    #[serde(default = "default_scale")]
    pub scale: f32,

    /// Optional image for NPS display
    #[serde(default)]
    pub image: Option<String>,

    /// Format string for NPS display (e.g., "{nps:.1} NPS")
    #[serde(default = "default_format")]
    pub format: String,

    #[serde(default = "default_true")]
    pub visible: bool,
}

fn default_format() -> String {
    "{nps:.1} NPS".into()
}
fn default_true() -> bool {
    true
}

impl Default for NpsConfig {
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
