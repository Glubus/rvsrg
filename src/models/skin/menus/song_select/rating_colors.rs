//! Rating colors configuration for difficulty ratings.

use crate::models::skin::common::Color;
use serde::{Deserialize, Serialize};

fn default_stream() -> Color {
    [0.30, 0.85, 0.50, 1.0]
} // Green
fn default_jumpstream() -> Color {
    [0.95, 0.75, 0.20, 1.0]
} // Orange
fn default_handstream() -> Color {
    [0.90, 0.45, 0.30, 1.0]
} // Red-orange
fn default_stamina() -> Color {
    [0.85, 0.30, 0.55, 1.0]
} // Pink
fn default_jackspeed() -> Color {
    [0.60, 0.40, 0.90, 1.0]
} // Purple
fn default_chordjack() -> Color {
    [0.40, 0.60, 0.95, 1.0]
} // Blue
fn default_technical() -> Color {
    [0.20, 0.80, 0.85, 1.0]
} // Cyan

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingColorsConfig {
    #[serde(default = "default_stream")]
    pub stream: Color,

    #[serde(default = "default_jumpstream")]
    pub jumpstream: Color,

    #[serde(default = "default_handstream")]
    pub handstream: Color,

    #[serde(default = "default_stamina")]
    pub stamina: Color,

    #[serde(default = "default_jackspeed")]
    pub jackspeed: Color,

    #[serde(default = "default_chordjack")]
    pub chordjack: Color,

    #[serde(default = "default_technical")]
    pub technical: Color,
}

impl Default for RatingColorsConfig {
    fn default() -> Self {
        Self {
            stream: default_stream(),
            jumpstream: default_jumpstream(),
            handstream: default_handstream(),
            stamina: default_stamina(),
            jackspeed: default_jackspeed(),
            chordjack: default_chordjack(),
            technical: default_technical(),
        }
    }
}
