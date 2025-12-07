//! Time Left / Progress display configuration
//! Supports multiple display modes: Bar, Circle (watch-like), or Text (minutes:seconds)

use crate::models::skin::common::{Color, Vec2Conf};
use serde::{Deserialize, Serialize};

/// Display mode for time remaining
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum TimeDisplayMode {
    /// Horizontal progress bar
    #[default]
    Bar,
    /// Circular progress (like a watch)
    Circle,
    /// Text display (e.g., "2:34")
    Text,
}

fn default_position() -> Vec2Conf {
    Vec2Conf { x: 960.0, y: 50.0 }
}
fn default_size() -> Vec2Conf {
    Vec2Conf { x: 400.0, y: 20.0 }
}
fn default_bar_color() -> Color {
    [0.2, 0.8, 0.3, 1.0] // Green
}
fn default_progress_color() -> Color {
    [0.3, 0.9, 0.4, 1.0] // Bright green
}
fn default_background_color() -> Color {
    [0.1, 0.1, 0.1, 0.8] // Dark background
}
fn default_text_color() -> Color {
    [1.0, 1.0, 1.0, 1.0] // White
}
fn default_scale() -> f32 {
    24.0
}
fn default_border_width() -> f32 {
    2.0
}
fn default_circle_radius() -> f32 {
    30.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeLeftConfig {
    #[serde(default = "default_position")]
    pub position: Vec2Conf,

    #[serde(default = "default_size")]
    pub size: Vec2Conf,

    #[serde(default)]
    pub mode: TimeDisplayMode,

    /// Color of the filled/progress portion
    #[serde(default = "default_progress_color")]
    pub progress_color: Color,

    /// Color of the empty/background portion
    #[serde(default = "default_background_color")]
    pub background_color: Color,

    /// Border color (for bar and circle)
    #[serde(default = "default_bar_color")]
    pub border_color: Color,

    /// Text color (for text mode)
    #[serde(default = "default_text_color")]
    pub text_color: Color,

    /// Border width
    #[serde(default = "default_border_width")]
    pub border_width: f32,

    /// Circle radius (for circle mode)
    #[serde(default = "default_circle_radius")]
    pub circle_radius: f32,

    /// Text scale (for text mode)
    #[serde(default = "default_scale")]
    pub text_scale: f32,

    /// Format string for text mode
    /// Use {elapsed}, {remaining}, {total}, {percent}
    #[serde(default = "default_format")]
    pub format: String,

    /// Optional background image (bar mode)
    #[serde(default)]
    pub background_image: Option<String>,

    /// Optional progress/fill image (bar mode)
    #[serde(default)]
    pub progress_image: Option<String>,

    /// Optional circle image (circle mode)
    #[serde(default)]
    pub circle_image: Option<String>,

    #[serde(default = "default_visible")]
    pub visible: bool,
}

fn default_format() -> String {
    "{remaining}".to_string()
}

fn default_visible() -> bool {
    true
}

impl Default for TimeLeftConfig {
    fn default() -> Self {
        Self {
            position: default_position(),
            size: default_size(),
            mode: TimeDisplayMode::default(),
            progress_color: default_progress_color(),
            background_color: default_background_color(),
            border_color: default_bar_color(),
            text_color: default_text_color(),
            border_width: default_border_width(),
            circle_radius: default_circle_radius(),
            text_scale: default_scale(),
            format: default_format(),
            background_image: None,
            progress_image: None,
            circle_image: None,
            visible: true,
        }
    }
}
