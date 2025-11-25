#[derive(Clone)]
pub struct PlayfieldConfig {
    pub column_width_pixels: f32,
    pub note_width_pixels: f32,
    pub note_height_pixels: f32,
}

impl PlayfieldConfig {
    pub fn new() -> Self {
        Self {
            column_width_pixels: 100.0,
            note_width_pixels: 90.0,
            note_height_pixels: 90.0, // CHANGÉ de 20.0 à 90.0 pour être carré par défaut
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