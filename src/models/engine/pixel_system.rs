#[derive(Clone)]
pub struct PixelSystem {
    pub pixel_size: f32,
    pub window_width: u32,
    pub window_height: u32,
}

impl PixelSystem {
    pub fn new(window_width: u32, window_height: u32) -> Self {
        let pixel_size = 2.0 / window_height as f32;
        Self {
            pixel_size,
            window_width,
            window_height,
        }
    }

    pub fn pixels_to_normalized(&self, pixels: f32) -> f32 {
        pixels * self.pixel_size
    }

    pub fn update_size(&mut self, width: u32, height: u32) {
        self.window_width = width;
        self.window_height = height;
        self.pixel_size = 2.0 / height as f32;
    }
}
