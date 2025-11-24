use crate::models::stats::{HitStats, Judgement, JudgementColors};
use wgpu_text::glyph_brush::{Section, Text};

pub struct JudgementPanel {
    position: (f32, f32),
    colors: JudgementColors,
    judgement_lines: [String; 7],
    remaining_text: String,
    scroll_speed_text: String,
}

impl JudgementPanel {
    pub fn new(x_pixels: f32, y_pixels: f32, colors: JudgementColors) -> Self {
        Self {
            position: (x_pixels, y_pixels),
            colors,
            judgement_lines: std::array::from_fn(|_| String::new()),
            remaining_text: String::new(),
            scroll_speed_text: String::new(),
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = (x, y);
    }

    pub fn render(
        &mut self,
        stats: &HitStats,
        remaining_notes: usize,
        scroll_speed_ms: f64,
        screen_width: f32,
        screen_height: f32,
    ) -> Vec<Section<'_>> {
        let mut sections = Vec::new();
        let (x, mut y) = self.position;
        let scale_ratio = screen_height / 1080.0;
        let spacing_small = 25.0 * scale_ratio;
        let spacing_large = 30.0 * scale_ratio;

        sections.push(Section {
            screen_position: (x, y),
            bounds: (screen_width, screen_height),
            text: vec![
                Text::new("judgement:")
                    .with_scale(18.0 * scale_ratio)
                    .with_color([1.0, 1.0, 1.0, 1.0]),
            ],
            ..Default::default()
        });

        y += spacing_large;

        let lines = [
            ("Marv", self.colors.marv, stats.marv),
            ("Perfect", self.colors.perfect, stats.perfect),
            ("Great", self.colors.great, stats.great),
            ("Good", self.colors.good, stats.good),
            ("Bad", self.colors.bad, stats.bad),
            ("Miss", self.colors.miss, stats.miss),
            ("Ghost Tap", self.colors.ghost_tap, stats.ghost_tap),
        ];

        for (entry, (label, color, count)) in self.judgement_lines.iter_mut().zip(lines.iter()) {
            entry.clear();
            entry.push_str(label);
            entry.push_str(": ");
            entry.push_str(&count.to_string());
            sections.push(Section {
                screen_position: (x, y),
                bounds: (screen_width, screen_height),
                text: vec![
                    Text::new(entry)
                        .with_scale(16.0 * scale_ratio)
                        .with_color(*color),
                ],
                ..Default::default()
            });
            y += spacing_small;
        }

        self.remaining_text = format!("Remaining notes: {}", remaining_notes);
        sections.push(Section {
            screen_position: (x, y),
            bounds: (screen_width, screen_height),
            text: vec![
                Text::new(&self.remaining_text)
                    .with_scale(16.0 * scale_ratio)
                    .with_color([1.0, 1.0, 1.0, 1.0]),
            ],
            ..Default::default()
        });
        y += spacing_small;

        self.scroll_speed_text = format!("Speed: {:.1} ms", scroll_speed_ms);
        sections.push(Section {
            screen_position: (x, y),
            bounds: (screen_width, screen_height),
            text: vec![
                Text::new(&self.scroll_speed_text)
                    .with_scale(16.0 * scale_ratio)
                    .with_color([1.0, 1.0, 1.0, 1.0]),
            ],
            ..Default::default()
        });

        sections
    }
}

pub struct JudgementFlash {
    position: (f32, f32),
    text_buffer: String,
}

impl JudgementFlash {
    pub fn new(x_pixels: f32, y_pixels: f32) -> Self {
        Self {
            position: (x_pixels, y_pixels),
            text_buffer: String::new(),
        }
    }

    pub fn set_position(&mut self, x_pixels: f32, y_pixels: f32) {
        self.position = (x_pixels, y_pixels);
    }

    pub fn render(
        &mut self,
        last_judgement: Option<Judgement>,
        screen_width: f32,
        screen_height: f32,
    ) -> Vec<Section<'_>> {
        let Some(judgement) = last_judgement else {
            return Vec::new();
        };

        let (label, color) = match judgement {
            Judgement::Marv => ("Marvelous", [0.0, 1.0, 1.0, 1.0]),
            Judgement::Perfect => ("Perfect", [1.0, 1.0, 0.0, 1.0]),
            Judgement::Great => ("Great", [0.0, 1.0, 0.0, 1.0]),
            Judgement::Good => ("Good", [0.0, 0.0, 0.5, 1.0]),
            Judgement::Bad => ("Bad", [1.0, 0.41, 0.71, 1.0]),
            Judgement::Miss => ("Miss", [1.0, 0.0, 0.0, 1.0]),
            Judgement::GhostTap => ("Ghost Tap", [0.5, 0.5, 0.5, 1.0]),
        };

        let scale_ratio = screen_height / 1080.0;
        let font_scale = 36.0 * scale_ratio;

        self.text_buffer.clear();
        self.text_buffer.push_str(label);

        let text_width_estimate = self.text_buffer.len() as f32 * 0.6 * font_scale;
        let centered_x = self.position.0 - (text_width_estimate / 2.0);

        vec![Section {
            screen_position: (centered_x, self.position.1),
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
