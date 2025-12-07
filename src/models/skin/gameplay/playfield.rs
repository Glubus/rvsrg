//! Playfield configuration.

use crate::models::skin::common::Vec2Conf;
use serde::{Deserialize, Serialize};

fn default_position() -> Vec2Conf {
    Vec2Conf { x: 640.0, y: 0.0 }
}
fn default_column_width() -> f32 {
    100.0
}
fn default_receptor_spacing() -> f32 {
    0.0
}
fn default_note_size() -> Vec2Conf {
    Vec2Conf { x: 90.0, y: 90.0 }
}
fn default_receptor_size() -> Vec2Conf {
    Vec2Conf { x: 90.0, y: 90.0 }
}
fn default_hit_position_y() -> f32 {
    0.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayfieldConfig {
    #[serde(default = "default_position")]
    pub position: Vec2Conf,

    #[serde(default = "default_column_width")]
    pub column_width: f32,

    #[serde(default = "default_receptor_spacing")]
    pub receptor_spacing: f32,

    #[serde(default = "default_note_size")]
    pub note_size: Vec2Conf,

    #[serde(default = "default_receptor_size")]
    pub receptor_size: Vec2Conf,

    #[serde(default = "default_hit_position_y")]
    pub hit_position_y: f32,

    /// Optional background image for the playfield lane
    #[serde(default)]
    pub lane_image: Option<String>,
}

impl Default for PlayfieldConfig {
    fn default() -> Self {
        Self {
            position: default_position(),
            column_width: default_column_width(),
            receptor_spacing: default_receptor_spacing(),
            note_size: default_note_size(),
            receptor_size: default_receptor_size(),
            hit_position_y: default_hit_position_y(),
            lane_image: None,
        }
    }
}

