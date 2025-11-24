use wgpu_text::glyph_brush::{Section, Text};

pub struct ScoreDisplay {
    position: (f32, f32),
    current_score: u32,
    score_text: String,
}

impl ScoreDisplay {
    pub fn new(x_pixels: f32, y_pixels: f32) -> Self {
        Self {
            position: (x_pixels, y_pixels),
            current_score: 0,
            score_text: String::new(),
        }
    }

    pub fn set_position(&mut self, x_pixels: f32, y_pixels: f32) {
        self.position = (x_pixels, y_pixels);
    }

    pub fn set_score(&mut self, value: u32) {
        self.current_score = value;
    }

    pub fn render(&mut self, screen_width: f32, screen_height: f32) -> Vec<Section<'_>> {
        let scale_ratio = screen_height / 1080.0;
        let spacing = 25.0 * scale_ratio;
        self.score_text.clear();
        self.score_text.push_str(&self.current_score.to_string());

        vec![
            Section {
                screen_position: self.position,
                bounds: (screen_width, screen_height),
                text: vec![
                    Text::new("Score")
                        .with_scale(20.0 * scale_ratio)
                        .with_color([1.0, 1.0, 1.0, 1.0]),
                ],
                ..Default::default()
            },
            Section {
                screen_position: (self.position.0, self.position.1 + spacing),
                bounds: (screen_width, screen_height),
                text: vec![
                    Text::new(&self.score_text)
                        .with_scale(24.0 * scale_ratio)
                        .with_color([1.0, 1.0, 1.0, 1.0]),
                ],
                ..Default::default()
            },
        ]
    }
}
