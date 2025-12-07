//! Displays judgement panels, combo text, and the center flash overlay.
use crate::models::skin::JudgementLabels;
use crate::models::stats::{HitStats, Judgement, JudgementColors};
use wgpu_text::glyph_brush::{Section, Text};

/// The Judgement Panel displays stats (Marvelous: 100, Perfect: 50, etc.)
/// Notes Remaining and Scroll Speed are now SEPARATE elements!
pub struct JudgementPanel {
    position: (f32, f32),
    text_size: f32,
    colors: JudgementColors,
    judgement_lines: [String; 7],
}

impl JudgementPanel {
    pub fn new(x: f32, y: f32, colors: JudgementColors) -> Self {
        Self {
            position: (x, y),
            text_size: 16.0,
            colors,
            judgement_lines: std::array::from_fn(|_| String::new()),
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = (x, y);
    }
    pub fn set_size(&mut self, size: f32) {
        self.text_size = size;
    }

    /// Render ONLY the judgement counts, NO notes/speed (those are separate now)
    pub fn render(
        &mut self,
        stats: &HitStats,
        screen_width: f32,
        screen_height: f32,
        labels: &JudgementLabels,
    ) -> Vec<Section<'_>> {
        let mut sections = Vec::new();
        let (x, mut y) = self.position;
        let scale_ratio = screen_height / 1080.0;
        let font_scale = self.text_size * scale_ratio;
        let spacing = font_scale * 1.2;

        sections.push(Section {
            screen_position: (x, y),
            bounds: (screen_width, screen_height),
            text: vec![
                Text::new("judgement:")
                    .with_scale(font_scale * 1.1)
                    .with_color([1.0, 1.0, 1.0, 1.0]),
            ],
            ..Default::default()
        });
        y += spacing * 1.5;

        let lines = [
            (&labels.marv, self.colors.marv, stats.marv),
            (&labels.perfect, self.colors.perfect, stats.perfect),
            (&labels.great, self.colors.great, stats.great),
            (&labels.good, self.colors.good, stats.good),
            (&labels.bad, self.colors.bad, stats.bad),
            (&labels.miss, self.colors.miss, stats.miss),
            (&labels.ghost_tap, self.colors.ghost_tap, stats.ghost_tap),
        ];

        for (entry, (label, color, count)) in self.judgement_lines.iter_mut().zip(lines.iter()) {
            entry.clear();
            entry.push_str(label);
            entry.push_str(": ");
            entry.push_str(&count.to_string());
            sections.push(Section {
                screen_position: (x, y),
                bounds: (screen_width, screen_height),
                text: vec![Text::new(entry).with_scale(font_scale).with_color(*color)],
                ..Default::default()
            });
            y += spacing;
        }

        // NOTE: Notes Remaining and Scroll Speed are now separate elements!
        // They are rendered by NotesRemainingDisplay and ScrollSpeedDisplay

        sections
    }
}

/// The Judgement Flash displays a centered text when hitting notes
pub struct JudgementFlash {
    position: (f32, f32),
    text_buffer: String,
    /// If true, show +/- timing indicator (early = "-", late = "+")
    pub show_timing: bool,
}

impl JudgementFlash {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: (x, y),
            text_buffer: String::new(),
            show_timing: false,
        }
    }
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = (x, y);
    }

    /// Render the flash with optional timing indicator
    /// timing_ms: negative = early, positive = late (in milliseconds from perfect hit)
    pub fn render(
        &mut self,
        last_judgement: Option<Judgement>,
        timing_ms: Option<f64>,
        screen_width: f32,
        screen_height: f32,
        colors: &JudgementColors,
        labels: &JudgementLabels,
    ) -> Vec<Section<'_>> {
        let Some(judgement) = last_judgement else {
            return Vec::new();
        };

        let (label, color) = match judgement {
            Judgement::Marv => (labels.marv.as_str(), colors.marv),
            Judgement::Perfect => (labels.perfect.as_str(), colors.perfect),
            Judgement::Great => (labels.great.as_str(), colors.great),
            Judgement::Good => (labels.good.as_str(), colors.good),
            Judgement::Bad => (labels.bad.as_str(), colors.bad),
            Judgement::Miss => (labels.miss.as_str(), colors.miss),
            Judgement::GhostTap => (labels.ghost_tap.as_str(), colors.ghost_tap),
        };

        let scale_ratio = screen_height / 1080.0;
        let font_scale = 48.0 * scale_ratio;
        self.text_buffer.clear();

        // Add timing indicator if enabled
        // Early (negative ms) -> "-" on LEFT
        // Late (positive ms) -> "+" on RIGHT
        let mut is_early = false;
        let mut is_late = false;

        if self.show_timing {
            if let Some(ms) = timing_ms {
                if ms < 5.0 {
                    is_late = true;
                } else if ms > -5.0 {
                    is_early = true;
                }
            }
        }

        if is_early {
            self.text_buffer.push_str("- ");
        }

        self.text_buffer.push_str(label);

        if is_late {
            self.text_buffer.push_str(" +");
        }

        let text_width = self.text_buffer.len() as f32 * 0.6 * font_scale;
        let cx = self.position.0 - (text_width / 2.0);

        vec![Section {
            screen_position: (cx, self.position.1),
            bounds: (screen_width, screen_height),
            text: vec![
                Text::new(&self.text_buffer)
                    .with_scale(font_scale)
                    .with_color(color),
            ],
            ..Default::default()
        }]
    }
}

