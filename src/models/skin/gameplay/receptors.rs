//! Receptor configuration.

use crate::models::skin::common::{Color, Vec2Conf};
use serde::{Deserialize, Serialize};

fn default_color() -> Color {
    [1.0, 1.0, 1.0, 1.0]
}
fn default_pressed_color() -> Color {
    [0.8, 0.8, 1.0, 1.0]
}
fn default_size() -> Vec2Conf {
    Vec2Conf { x: 90.0, y: 90.0 }
}

/// Per-column receptor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceptorColumnConfig {
    #[serde(default = "default_color")]
    pub color: Color,

    #[serde(default = "default_pressed_color")]
    pub pressed_color: Color,

    #[serde(default = "default_size")]
    pub size: Vec2Conf,

    /// Image for this column's receptor
    #[serde(default)]
    pub image: Option<String>,

    /// Image when pressed
    #[serde(default)]
    pub pressed_image: Option<String>,
}

impl Default for ReceptorColumnConfig {
    fn default() -> Self {
        Self {
            color: default_color(),
            pressed_color: default_pressed_color(),
            size: default_size(),
            image: None,
            pressed_image: None,
        }
    }
}

/// Default receptor config (fallback)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReceptorDefaults {
    #[serde(default = "default_color")]
    pub color: Color,

    #[serde(default = "default_pressed_color")]
    pub pressed_color: Color,

    #[serde(default = "default_size")]
    pub size: Vec2Conf,

    /// Fallback receptor image
    #[serde(default)]
    pub image: Option<String>,

    /// Fallback pressed image
    #[serde(default)]
    pub pressed_image: Option<String>,
}

