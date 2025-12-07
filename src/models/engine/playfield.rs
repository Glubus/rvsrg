//! Playfield configuration and layout.



/// Configuration for the playfield layout.
#[derive(Clone)]
pub struct PlayfieldConfig {
    pub column_width_pixels: f32,
    pub note_width_pixels: f32,
    pub note_height_pixels: f32,
    pub receptor_width_pixels: f32,
    pub receptor_height_pixels: f32,
    pub receptor_spacing_pixels: f32,
    pub x_offset_pixels: f32,
    pub y_offset_pixels: f32,
}

impl PlayfieldConfig {
    pub fn new() -> Self {
        Self {
            column_width_pixels: 100.0,
            note_width_pixels: 90.0,
            note_height_pixels: 90.0,
            receptor_width_pixels: 90.0,
            receptor_height_pixels: 90.0,
            receptor_spacing_pixels: 0.0,
            x_offset_pixels: 0.0,
            y_offset_pixels: 0.0,
        }
    }
    pub fn decrease_note_size(&mut self) {
        self.note_width_pixels = (self.note_width_pixels - 5.0).max(10.0);
        self.note_height_pixels = self.note_width_pixels;
        self.column_width_pixels = self.note_width_pixels;
    }
    pub fn increase_note_size(&mut self) {
        self.note_width_pixels = (self.note_width_pixels + 5.0).min(200.0);
        self.note_height_pixels = self.note_width_pixels;
        self.column_width_pixels = self.note_width_pixels;
    }
}

