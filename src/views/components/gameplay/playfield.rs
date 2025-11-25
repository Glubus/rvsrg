use crate::models::engine::{
    HIT_LINE_Y, InstanceRaw, NUM_COLUMNS, NoteData, PixelSystem, PlayfieldConfig, VISIBLE_DISTANCE,
};
pub struct PlayfieldDisplay {
    pub config: PlayfieldConfig,
}
impl PlayfieldDisplay {
    pub fn new(config: PlayfieldConfig) -> Self {
        Self { config }
    }
    pub fn get_total_width_pixels(&self) -> f32 {
        let cols = NUM_COLUMNS as f32;
        let spaces = (cols - 1.0).max(0.0);
        (cols * self.config.column_width_pixels) + (spaces * self.config.receptor_spacing_pixels)
    }
    pub fn get_bounds(&self, pixel_system: &PixelSystem) -> (f32, f32) {
        let total_width_px = self.get_total_width_pixels();
        let width_norm = pixel_system.x_pixels_to_normalized(total_width_px);
        let x = -width_norm / 2.0;
        (x, width_norm)
    }
    pub fn render_notes(
        &self,
        visible_notes: &[NoteData],
        song_time: f64,
        scroll_speed_ms: f64,
        pixel_system: &PixelSystem,
    ) -> Vec<(usize, InstanceRaw)> {
        let (playfield_left_x, _) = self.get_bounds(pixel_system);
        let column_width_norm =
            pixel_system.x_pixels_to_normalized(self.config.column_width_pixels);
        let spacing_norm = pixel_system.x_pixels_to_normalized(self.config.receptor_spacing_pixels);
        let note_width_norm = pixel_system.x_pixels_to_normalized(self.config.note_width_pixels);
        let note_height_norm = pixel_system.y_pixels_to_normalized(self.config.note_height_pixels);
        let x_offset_norm = pixel_system.x_pixels_to_normalized(self.config.x_offset_pixels);
        let y_offset_norm = pixel_system.y_pixels_to_normalized(self.config.y_offset_pixels);
        let mut instances = Vec::with_capacity(visible_notes.len());
        for note in visible_notes {
            if note.hit {
                continue;
            }
            let time_to_hit = note.timestamp_ms - song_time;
            let progress = time_to_hit / scroll_speed_ms;
            let y_pos = HIT_LINE_Y + y_offset_norm + (VISIBLE_DISTANCE * progress as f32);
            let col_offset = note.column as f32 * (column_width_norm + spacing_norm);
            let center_x =
                playfield_left_x + col_offset + (column_width_norm / 2.0) + x_offset_norm;
            instances.push((
                note.column,
                InstanceRaw {
                    offset: [center_x, y_pos],
                    scale: [note_width_norm, note_height_norm],
                },
            ));
        }
        instances
    }
    pub fn render_receptors(&self, pixel_system: &PixelSystem) -> Vec<InstanceRaw> {
        let (playfield_left_x, _) = self.get_bounds(pixel_system);
        let column_width_norm =
            pixel_system.x_pixels_to_normalized(self.config.column_width_pixels);
        let spacing_norm = pixel_system.x_pixels_to_normalized(self.config.receptor_spacing_pixels);
        let receptor_width_norm =
            pixel_system.x_pixels_to_normalized(self.config.receptor_width_pixels);
        let receptor_height_norm =
            pixel_system.y_pixels_to_normalized(self.config.receptor_height_pixels);
        let x_offset_norm = pixel_system.x_pixels_to_normalized(self.config.x_offset_pixels);
        let y_offset_norm = pixel_system.y_pixels_to_normalized(self.config.y_offset_pixels);
        let mut instances = Vec::with_capacity(NUM_COLUMNS);
        for col in 0..NUM_COLUMNS {
            let col_offset = col as f32 * (column_width_norm + spacing_norm);
            let center_x =
                playfield_left_x + col_offset + (column_width_norm / 2.0) + x_offset_norm;
            let center_y = HIT_LINE_Y + y_offset_norm;
            instances.push(InstanceRaw {
                offset: [center_x, center_y],
                scale: [receptor_width_norm, receptor_height_norm],
            });
        }
        instances
    }
}
