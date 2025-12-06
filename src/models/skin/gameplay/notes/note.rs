//! Per-column note configuration for a specific keymode.

use crate::models::skin::common::{Color, Vec2Conf};
use serde::{Deserialize, Serialize};

fn default_color() -> Color {
    [1.0, 1.0, 1.0, 1.0]
}
fn default_size() -> Vec2Conf {
    Vec2Conf { x: 90.0, y: 90.0 }
}

/// Configuration for a single note column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteColumnConfig {
    #[serde(default = "default_color")]
    pub color: Color,

    #[serde(default = "default_size")]
    pub size: Vec2Conf,

    /// Image for this column's notes
    #[serde(default)]
    pub image: Option<String>,
}

impl Default for NoteColumnConfig {
    fn default() -> Self {
        Self {
            color: default_color(),
            size: default_size(),
            image: None,
        }
    }
}

/// Default note config (fallback when no per-column config exists)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NoteDefaults {
    #[serde(default = "default_color")]
    pub color: Color,

    #[serde(default = "default_size")]
    pub size: Vec2Conf,

    /// Fallback image for notes
    #[serde(default)]
    pub image: Option<String>,
}
