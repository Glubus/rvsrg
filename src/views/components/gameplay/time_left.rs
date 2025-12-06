//! Time Left display component
//! Shows song progress as bar, circle, or text

use wgpu_text::glyph_brush::{Section, Text};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeDisplayMode {
    Bar,
    Circle,
    Text,
}

pub struct TimeLeftDisplay {
    position: (f32, f32),
    size: (f32, f32),
    mode: TimeDisplayMode,
    progress_color: [f32; 4],
    background_color: [f32; 4],
    text_color: [f32; 4],
    text_scale: f32,
    format: String,
    text_buffer: String,
    pub visible: bool,
}

impl TimeLeftDisplay {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: (x, y),
            size: (200.0, 20.0),
            mode: TimeDisplayMode::Text,
            progress_color: [0.2, 0.8, 0.2, 1.0],
            background_color: [0.2, 0.2, 0.2, 0.8],
            text_color: [1.0, 1.0, 1.0, 1.0],
            text_scale: 16.0,
            format: "{elapsed} / {total}".to_string(),
            text_buffer: String::new(),
            visible: true,
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = (x, y);
    }

    pub fn set_size(&mut self, w: f32, h: f32) {
        self.size = (w, h);
    }

    pub fn set_mode(&mut self, mode: TimeDisplayMode) {
        self.mode = mode;
    }

    pub fn set_progress_color(&mut self, color: [f32; 4]) {
        self.progress_color = color;
    }

    pub fn set_background_color(&mut self, color: [f32; 4]) {
        self.background_color = color;
    }

    pub fn set_text_color(&mut self, color: [f32; 4]) {
        self.text_color = color;
    }

    pub fn set_text_scale(&mut self, scale: f32) {
        self.text_scale = scale;
    }

    pub fn set_format(&mut self, format: String) {
        self.format = format;
    }

    fn format_time(ms: f64) -> String {
        let total_seconds = (ms / 1000.0).max(0.0) as u32;
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{}:{:02}", minutes, seconds)
    }

    /// Render the time display
    /// elapsed_ms and total_ms are in milliseconds
    pub fn render(
        &mut self,
        elapsed_ms: f64,
        total_ms: f64,
        screen_width: f32,
        screen_height: f32,
    ) -> Vec<Section<'_>> {
        if !self.visible {
            return Vec::new();
        }

        let scale_ratio = screen_height / 1080.0;
        let font_scale = self.text_scale * scale_ratio;

        let remaining = (total_ms - elapsed_ms).max(0.0);
        let percent = if total_ms > 0.0 {
            (elapsed_ms / total_ms * 100.0).min(100.0)
        } else {
            0.0
        };

        // Format text based on format string
        self.text_buffer = self
            .format
            .replace("{elapsed}", &Self::format_time(elapsed_ms))
            .replace("{remaining}", &Self::format_time(remaining))
            .replace("{total}", &Self::format_time(total_ms))
            .replace("{percent}", &format!("{:.0}%", percent));

        match self.mode {
            TimeDisplayMode::Text => {
                vec![Section {
                    screen_position: self.position,
                    bounds: (screen_width, screen_height),
                    text: vec![
                        Text::new(&self.text_buffer)
                            .with_scale(font_scale)
                            .with_color(self.text_color),
                    ],
                    ..Default::default()
                }]
            }
            TimeDisplayMode::Bar | TimeDisplayMode::Circle => {
                // Bar and Circle modes need custom rendering via quads
                // For now, fallback to text display
                // TODO: Add quad-based rendering for bar/circle modes
                vec![Section {
                    screen_position: self.position,
                    bounds: (screen_width, screen_height),
                    text: vec![
                        Text::new(&self.text_buffer)
                            .with_scale(font_scale)
                            .with_color(self.text_color),
                    ],
                    ..Default::default()
                }]
            }
        }
    }

    /// Get progress percentage (0.0 to 1.0) for bar/circle rendering
    pub fn get_progress(&self, elapsed_ms: f64, total_ms: f64) -> f32 {
        if total_ms > 0.0 {
            (elapsed_ms / total_ms).clamp(0.0, 1.0) as f32
        } else {
            0.0
        }
    }
    /// Get progress instance for rendering
    pub fn get_progress_instance(
        &self,
        elapsed_ms: f64,
        total_ms: f64,
        screen_width: f32,
        screen_height: f32,
    ) -> Option<crate::views::components::common::primitives::ProgressInstance> {
        if !self.visible {
            return None;
        }

        match self.mode {
            TimeDisplayMode::Bar | TimeDisplayMode::Circle => {
                let progress = self.get_progress(elapsed_ms, total_ms);

                // Mode: 0 for Bar, 1 for Circle
                let mode_idx = if self.mode == TimeDisplayMode::Bar {
                    0
                } else {
                    1
                };

                Some(
                    crate::views::components::common::primitives::progress_from_rect(
                        self.position.0,
                        self.position.1,
                        self.size.0,
                        self.size.1,
                        self.progress_color,
                        self.background_color,
                        progress,
                        mode_idx,
                        screen_width,
                        screen_height,
                    ),
                )
            }
            _ => None,
        }
    }
}
