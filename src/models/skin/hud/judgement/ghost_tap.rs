//! Ghost tap judgement flash configuration.

use crate::models::skin::common::{Color, Vec2Conf};
use serde::{Deserialize, Serialize};

fn default_label() -> String {
    "Ghost Tap".into()
}
fn default_color() -> Color {
    [0.5, 0.5, 0.5, 1.0]
} // Gray
fn default_position() -> Vec2Conf {
    Vec2Conf { x: 640.0, y: 300.0 }
}
fn default_size() -> Vec2Conf {
    Vec2Conf { x: 200.0, y: 50.0 }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgementFlashGhostTap {
    #[serde(default = "default_label")]
    pub label: String,

    #[serde(default = "default_color")]
    pub color: Color,

    #[serde(default)]
    pub image: Option<String>,

    #[serde(default = "default_position")]
    pub position: Vec2Conf,

    #[serde(default = "default_size")]
    pub size: Vec2Conf,

    #[serde(default = "default_true")]
    pub visible: bool,
}

fn default_true() -> bool {
    true
}

impl Default for JudgementFlashGhostTap {
    fn default() -> Self {
        Self {
            label: default_label(),
            color: default_color(),
            image: None,
            position: default_position(),
            size: default_size(),
            visible: true,
        }
    }
}

