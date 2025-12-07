//! Hold note configuration.

use crate::models::skin::common::{Color, Vec2Conf};
use serde::{Deserialize, Serialize};

fn default_color() -> Color {
    [0.8, 0.8, 1.0, 1.0]
}
fn default_body_width() -> f32 {
    90.0
}
fn default_end_size() -> Vec2Conf {
    Vec2Conf { x: 90.0, y: 30.0 }
}

/// Configuration for hold notes (can be per-column in KeyModeConfig)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldConfig {
    #[serde(default = "default_color")]
    pub color: Color,

    #[serde(default = "default_body_width")]
    pub body_width: f32,

    #[serde(default = "default_end_size")]
    pub end_size: Vec2Conf,

    /// Image for hold body (stretched vertically)
    #[serde(default)]
    pub body_image: Option<String>,

    /// Image for hold end (tail)
    #[serde(default)]
    pub end_image: Option<String>,
}

impl Default for HoldConfig {
    fn default() -> Self {
        Self {
            color: default_color(),
            body_width: default_body_width(),
            end_size: default_end_size(),
            body_image: None,
            end_image: None,
        }
    }
}

