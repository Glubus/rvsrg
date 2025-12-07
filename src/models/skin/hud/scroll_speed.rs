//! Scroll Speed display configuration

use crate::models::skin::common::{Color, Vec2Conf};
use serde::{Deserialize, Serialize};

fn default_position() -> Vec2Conf {
    Vec2Conf { x: 50.0, y: 530.0 }
}
fn default_size() -> Vec2Conf {
    Vec2Conf { x: 150.0, y: 30.0 }
}
fn default_color() -> Color {
    [1.0, 1.0, 1.0, 1.0]
}
fn default_scale() -> f32 {
    16.0
}
fn default_format() -> String {
    "Speed: {speed:.0}".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollSpeedConfig {
    #[serde(default = "default_position")]
    pub position: Vec2Conf,

    #[serde(default = "default_size")]
    pub size: Vec2Conf,

    #[serde(default = "default_color")]
    pub color: Color,

    #[serde(default = "default_scale")]
    pub scale: f32,

    #[serde(default = "default_format")]
    pub format: String,

    #[serde(default)]
    pub image: Option<String>,

    #[serde(default = "default_visible")]
    pub visible: bool,
}

fn default_visible() -> bool {
    true
}

impl Default for ScrollSpeedConfig {
    fn default() -> Self {
        Self {
            position: default_position(),
            size: default_size(),
            color: default_color(),
            scale: default_scale(),
            format: default_format(),
            image: None,
            visible: true,
        }
    }
}
