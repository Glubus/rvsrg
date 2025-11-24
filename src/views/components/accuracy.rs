use wgpu_text::glyph_brush::{Section, Text};

pub struct AccuracyDisplay {
    position: (f32, f32),
    text_buffer: String,
}

impl AccuracyDisplay {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: (x, y),
            text_buffer: String::new(),
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = (x, y);
    }

    pub fn render(
        &mut self,
        accuracy: f64,
        screen_width: f32,
        screen_height: f32,
    ) -> Vec<Section<'_>> {
        let scale_ratio = screen_height / 1080.0;
        self.text_buffer = format!("accuracy: {:.2}%", accuracy);

        vec![Section {
            screen_position: self.position,
            bounds: (screen_width, screen_height),
            text: vec![
                Text::new(&self.text_buffer)
                    .with_scale(20.0 * scale_ratio)
                    .with_color([1.0, 1.0, 1.0, 1.0]),
            ],
            ..Default::default()
        }]
    }
}
