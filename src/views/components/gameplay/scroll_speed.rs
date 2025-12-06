//! Scroll Speed display component
//! Shows "Speed: 650" or similar text

use wgpu_text::glyph_brush::{Section, Text};

pub struct ScrollSpeedDisplay {
    position: (f32, f32),
    scale: f32,
    color: [f32; 4],
    format: String,
    text_buffer: String,
    pub visible: bool,
}

impl ScrollSpeedDisplay {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: (x, y),
            scale: 16.0,
            color: [1.0, 1.0, 1.0, 1.0],
            format: "Speed: {speed}".to_string(),
            text_buffer: String::new(),
            visible: true,
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = (x, y);
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }

    pub fn set_format(&mut self, format: String) {
        self.format = format;
    }

    pub fn render(
        &mut self,
        scroll_speed: f64,
        screen_width: f32,
        screen_height: f32,
    ) -> Vec<Section<'_>> {
        if !self.visible {
            return Vec::new();
        }

        let scale_ratio = screen_height / 1080.0;
        let font_scale = self.scale * scale_ratio;

        let speed_str = format!("{:.0}", scroll_speed);
        self.text_buffer = self
            .format
            .replace("{speed:.0}", &speed_str)
            .replace("{speed}", &speed_str);

        vec![Section {
            screen_position: self.position,
            bounds: (screen_width, screen_height),
            text: vec![
                Text::new(&self.text_buffer)
                    .with_scale(font_scale)
                    .with_color(self.color),
            ],
            ..Default::default()
        }]
    }
}
