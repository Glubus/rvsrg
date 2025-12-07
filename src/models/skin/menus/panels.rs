//! Generic panel styling configuration.

use crate::models::skin::common::Color;
use serde::{Deserialize, Serialize};

fn default_bg() -> Color {
    [0.08, 0.08, 0.10, 0.95]
}
fn default_secondary() -> Color {
    [0.12, 0.12, 0.15, 0.90]
}
fn default_border() -> Color {
    [0.25, 0.25, 0.30, 0.80]
}
fn default_accent() -> Color {
    [0.40, 0.70, 1.0, 1.0]
}
fn default_accent_dim() -> Color {
    [0.25, 0.45, 0.70, 1.0]
}
fn default_text_primary() -> Color {
    [1.0, 1.0, 1.0, 1.0]
}
fn default_text_secondary() -> Color {
    [0.75, 0.75, 0.80, 1.0]
}
fn default_text_muted() -> Color {
    [0.50, 0.50, 0.55, 1.0]
}

/// Default panel color theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelStyleConfig {
    #[serde(default = "default_bg")]
    pub background: Color,

    #[serde(default = "default_secondary")]
    pub secondary: Color,

    #[serde(default = "default_border")]
    pub border: Color,

    #[serde(default = "default_accent")]
    pub accent: Color,

    #[serde(default = "default_accent_dim")]
    pub accent_dim: Color,

    #[serde(default = "default_text_primary")]
    pub text_primary: Color,

    #[serde(default = "default_text_secondary")]
    pub text_secondary: Color,

    #[serde(default = "default_text_muted")]
    pub text_muted: Color,
}

impl Default for PanelStyleConfig {
    fn default() -> Self {
        Self {
            background: default_bg(),
            secondary: default_secondary(),
            border: default_border(),
            accent: default_accent(),
            accent_dim: default_accent_dim(),
            text_primary: default_text_primary(),
            text_secondary: default_text_secondary(),
            text_muted: default_text_muted(),
        }
    }
}

