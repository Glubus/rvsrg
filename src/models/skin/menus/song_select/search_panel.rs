//! Search panel configuration.

use crate::models::skin::common::{Color, Vec2Conf};
use serde::{Deserialize, Serialize};

fn default_position() -> Vec2Conf {
    Vec2Conf { x: 0.0, y: 0.0 }
}
fn default_size() -> Vec2Conf {
    Vec2Conf { x: 350.0, y: 600.0 }
}
fn default_bg_color() -> Color {
    [0.08, 0.08, 0.10, 0.95]
}
fn default_border_color() -> Color {
    [0.25, 0.25, 0.30, 0.8]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPanelConfig {
    #[serde(default = "default_position")]
    pub position: Vec2Conf,

    #[serde(default = "default_size")]
    pub size: Vec2Conf,

    #[serde(default = "default_bg_color")]
    pub background_color: Color,

    #[serde(default = "default_border_color")]
    pub border_color: Color,

    #[serde(default)]
    pub background_image: Option<String>,
}

impl Default for SearchPanelConfig {
    fn default() -> Self {
        Self {
            position: default_position(),
            size: default_size(),
            background_color: default_bg_color(),
            border_color: default_border_color(),
            background_image: None,
        }
    }
}

