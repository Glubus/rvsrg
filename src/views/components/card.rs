use crate::database::models::{Beatmap, Beatmapset};

/// Una card visuelle pour afficher une map
pub struct CardDisplay {
    pub beatmapset: Beatmapset,
    pub beatmaps: Vec<Beatmap>,
    pub current_difficulty_index: usize,
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub is_selected: bool,
}

impl CardDisplay {
    pub fn new(
        beatmapset: Beatmapset,
        beatmaps: Vec<Beatmap>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        is_selected: bool,
        current_difficulty_index: usize,
    ) -> Self {
        Self {
            beatmapset,
            beatmaps,
            current_difficulty_index,
            width,
            height,
            x,
            y,
            is_selected,
        }
    }

    pub fn title_text(&self) -> String {
        self.beatmapset
            .title
            .as_deref()
            .unwrap_or("Unknown Title")
            .to_string()
    }

    pub fn artist_difficulty_text(&self) -> String {
        let artist = self
            .beatmapset
            .artist
            .as_deref()
            .unwrap_or("Unknown Artist");
        let difficulty = self
            .beatmaps
            .get(
                self.current_difficulty_index
                    .min(self.beatmaps.len().saturating_sub(1)),
            )
            .and_then(|bm| bm.difficulty_name.as_deref())
            .unwrap_or("Unknown");
        if self.beatmaps.len() > 1 {
            format!(
                "{} | {} ({}/{})",
                artist,
                difficulty,
                self.current_difficulty_index + 1,
                self.beatmaps.len()
            )
        } else {
            format!("{} | {}", artist, difficulty)
        }
    }

    pub fn text_color(&self) -> [f32; 4] {
        if self.is_selected {
            [1.0, 1.0, 0.0, 1.0]
        } else {
            [0.9, 0.9, 0.9, 1.0]
        }
    }

    pub fn background_color(&self) -> [f32; 4] {
        if self.is_selected {
            [0.0, 0.0, 0.0, 0.9]
        } else {
            [0.0, 0.0, 0.0, 0.8]
        }
    }
}
