//! Hit bar (error bar) display configuration.

use crate::models::skin::common::{Color, Vec2Conf};
use serde::{Deserialize, Serialize};

fn default_position() -> Vec2Conf {
    Vec2Conf { x: 640.0, y: 600.0 }
}
fn default_size() -> Vec2Conf {
    Vec2Conf { x: 300.0, y: 20.0 }
}
fn default_bar_color() -> Color {
    [0.3, 0.3, 0.3, 0.8]
} // Dark gray
fn default_indicator_color() -> Color {
    [1.0, 1.0, 1.0, 1.0]
} // White
fn default_scale() -> f32 {
    20.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitBarConfig {
    #[serde(default = "default_position")]
    pub position: Vec2Conf,

    #[serde(default = "default_size")]
    pub size: Vec2Conf,

    #[serde(default = "default_bar_color")]
    pub bar_color: Color,

    #[serde(default = "default_indicator_color")]
    pub indicator_color: Color,

    #[serde(default = "default_scale")]
    pub scale: f32,

    /// Optional background image for hit bar
    #[serde(default)]
    pub background_image: Option<String>,

    /// Optional indicator image
    #[serde(default)]
    pub indicator_image: Option<String>,

    #[serde(default = "default_true")]
    pub visible: bool,
}

fn default_true() -> bool {
    true
}

impl Default for HitBarConfig {
    fn default() -> Self {
        Self {
            position: default_position(),
            size: default_size(),
            bar_color: default_bar_color(),
            indicator_color: default_indicator_color(),
            scale: default_scale(),
            background_image: None,
            indicator_image: None,
            visible: true,
        }
    }
}

