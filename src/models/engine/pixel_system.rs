//! Utility to convert screen pixels into normalized coordinates.

/// Handles pixel-to-normalized coordinate conversion.
#[derive(Clone)]
pub struct PixelSystem {
    pub pixel_size: f32, // Derived from height (2.0 / height)
    pub window_width: u32,
    pub window_height: u32,
    pub aspect_ratio: f32,
}

impl PixelSystem {
    pub fn new(window_width: u32, window_height: u32) -> Self {
        let pixel_size = 2.0 / window_height as f32;
        let aspect_ratio = window_width as f32 / window_height as f32;
        Self {
            pixel_size,
            window_width,
            window_height,
            aspect_ratio,
        }
    }

    /// Converts pixel units into normalized Y size (height).
    pub fn y_pixels_to_normalized(&self, pixels: f32) -> f32 {
        pixels * self.pixel_size
    }

    /// Converts pixels into normalized X size, applying aspect-ratio correction.
    pub fn x_pixels_to_normalized(&self, pixels: f32) -> f32 {
        (pixels * self.pixel_size) / self.aspect_ratio
    }

    /// Legacy helper that defaults to the Y conversion.
    pub fn pixels_to_normalized(&self, pixels: f32) -> f32 {
        self.y_pixels_to_normalized(pixels)
    }

    pub fn update_size(&mut self, width: u32, height: u32, forced_ratio: Option<f32>) {
        self.window_width = width;
        self.window_height = height;
        self.pixel_size = 2.0 / height as f32;

        // Respect a forced aspect ratio if provided, else compute the actual value.
        self.aspect_ratio = forced_ratio.unwrap_or(width as f32 / height as f32);
    }
}
