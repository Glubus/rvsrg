//! Mine note configuration.

use crate::models::skin::common::{Color, Vec2Conf};
use serde::{Deserialize, Serialize};

fn default_color() -> Color {
    [1.0, 0.0, 0.0, 1.0]
} // Red
fn default_size() -> Vec2Conf {
    Vec2Conf { x: 90.0, y: 90.0 }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MineConfig {
    #[serde(default = "default_color")]
    pub color: Color,

    #[serde(default = "default_size")]
    pub size: Vec2Conf,

    /// Image for mine note
    #[serde(default)]
    pub image: Option<String>,
}

impl Default for MineConfig {
    fn default() -> Self {
        Self {
            color: default_color(),
            size: default_size(),
            image: None,
        }
    }
}

