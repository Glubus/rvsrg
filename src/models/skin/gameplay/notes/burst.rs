//! Burst note configuration.

use crate::models::skin::common::{Color, Vec2Conf};
use serde::{Deserialize, Serialize};

fn default_color() -> Color {
    [1.0, 0.8, 0.2, 1.0]
} // Orange
fn default_body_size() -> Vec2Conf {
    Vec2Conf { x: 90.0, y: 1.0 }
}
fn default_end_size() -> Vec2Conf {
    Vec2Conf { x: 90.0, y: 30.0 }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurstConfig {
    #[serde(default = "default_color")]
    pub color: Color,

    #[serde(default = "default_body_size")]
    pub body_size: Vec2Conf,

    #[serde(default = "default_end_size")]
    pub end_size: Vec2Conf,

    /// Image for burst body (stretched)
    #[serde(default)]
    pub body_image: Option<String>,

    /// Image for burst end (tail)
    #[serde(default)]
    pub end_image: Option<String>,
}

impl Default for BurstConfig {
    fn default() -> Self {
        Self {
            color: default_color(),
            body_size: default_body_size(),
            end_size: default_end_size(),
            body_image: None,
            end_image: None,
        }
    }
}

