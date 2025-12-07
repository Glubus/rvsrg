use wgpu_text::glyph_brush::{Section, Text};

pub struct ComboDisplay {
    position: (f32, f32),
    text_size: f32, // Nouveau
    text_buffer: String,
}

impl ComboDisplay {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: (x, y),
            text_size: 48.0,
            text_buffer: String::new(),
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = (x, y);
    }
    pub fn set_size(&mut self, size: f32) {
        self.text_size = size;
    }

    pub fn render(
        &mut self,
        combo: u32,
        screen_width: f32,
        screen_height: f32,
    ) -> Vec<Section<'_>> {
        let scale_ratio = screen_height / 1080.0;
        self.text_buffer = combo.to_string();

        // Utilise text_size du skin
        let font_scale = self.text_size * scale_ratio;
        let text_width_estimate = self.text_buffer.len() as f32 * 0.6 * font_scale;
        let centered_x = self.position.0 - (text_width_estimate / 2.0);

        vec![Section {
            screen_position: (centered_x, self.position.1),
            bounds: (screen_width, screen_height),
            text: vec![
                Text::new(&self.text_buffer)
                    .with_scale(font_scale)
                    .with_color([1.0, 1.0, 1.0, 1.0]),
            ],
            ..Default::default()
        }]
    }
}

