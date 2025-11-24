use crate::database::{Beatmap, Beatmapset, Database};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MenuState {
    pub beatmapsets: Vec<(Beatmapset, Vec<Beatmap>)>,
    pub start_index: usize,
    pub end_index: usize,
    pub selected_index: usize,
    pub selected_difficulty_index: usize,
    pub visible_count: usize,
    pub in_menu: bool,
    pub rate: f64,
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            beatmapsets: Vec::new(),
            start_index: 0,
            end_index: 0,
            selected_index: 0,
            selected_difficulty_index: 0,
            visible_count: 10,
            in_menu: true,
            rate: 1.0,
        }
    }

    pub fn increase_rate(&mut self) {
        self.rate = (self.rate + 0.1).min(2.0);
    }

    pub fn decrease_rate(&mut self) {
        self.rate = (self.rate - 0.1).max(0.5);
    }

    pub async fn load_from_db(
        menu_state: Arc<Mutex<Self>>,
        db: &Database,
    ) -> Result<(), sqlx::Error> {
        let beatmapsets = db.get_all_beatmapsets().await?;
        if let Ok(mut state) = menu_state.lock() {
            state.beatmapsets = beatmapsets.clone();
            state.selected_index = 0;
            state.selected_difficulty_index = 0;
            state.end_index = state.visible_count.min(state.beatmapsets.len());
            state.start_index = 0;
        }
        Ok(())
    }

    pub fn get_visible_items(&self) -> &[(Beatmapset, Vec<Beatmap>)] {
        if self.start_index >= self.beatmapsets.len() {
            return &[];
        }
        let end = self.end_index.min(self.beatmapsets.len());
        &self.beatmapsets[self.start_index..end]
    }

    pub fn get_relative_selected_index(&self) -> usize {
        if self.selected_index < self.start_index {
            0
        } else if self.selected_index >= self.end_index {
            self.end_index
                .saturating_sub(self.start_index)
                .saturating_sub(1)
        } else {
            self.selected_index - self.start_index
        }
    }

    pub fn move_up(&mut self) {
        if self.beatmapsets.is_empty() {
            return;
        }

        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.selected_difficulty_index = 0;

            if self.selected_index < self.start_index {
                self.start_index = self.selected_index;
                self.end_index =
                    (self.start_index + self.visible_count).min(self.beatmapsets.len());
            }
        }
    }

    pub fn move_down(&mut self) {
        if self.beatmapsets.is_empty() {
            return;
        }

        if self.selected_index < self.beatmapsets.len() - 1 {
            self.selected_index += 1;
            self.selected_difficulty_index = 0;

            if self.selected_index >= self.end_index {
                self.end_index = (self.selected_index + 1).min(self.beatmapsets.len());
                self.start_index = self.end_index.saturating_sub(self.visible_count);
            }
        }
    }

    pub fn get_selected_beatmapset(&self) -> Option<&(Beatmapset, Vec<Beatmap>)> {
        self.beatmapsets.get(self.selected_index)
    }

    pub fn get_selected_beatmap_path(&self) -> Option<PathBuf> {
        self.get_selected_beatmapset().and_then(|(_, beatmaps)| {
            beatmaps
                .get(
                    self.selected_difficulty_index
                        .min(beatmaps.len().saturating_sub(1)),
                )
                .map(|bm| PathBuf::from(&bm.path))
        })
    }

    pub fn next_difficulty(&mut self) {
        if let Some((_, beatmaps)) = self.get_selected_beatmapset() {
            if beatmaps.is_empty() {
                self.selected_difficulty_index = 0;
            } else {
                self.selected_difficulty_index =
                    (self.selected_difficulty_index + 1) % beatmaps.len();
            }
        }
    }

    pub fn previous_difficulty(&mut self) {
        if let Some((_, beatmaps)) = self.get_selected_beatmapset() {
            if beatmaps.is_empty() {
                self.selected_difficulty_index = 0;
            } else if self.selected_difficulty_index == 0 {
                self.selected_difficulty_index = beatmaps.len() - 1;
            } else {
                self.selected_difficulty_index -= 1;
            }
        }
    }

    pub fn get_selected_difficulty_name(&self) -> Option<String> {
        self.get_selected_beatmapset().and_then(|(_, beatmaps)| {
            beatmaps
                .get(
                    self.selected_difficulty_index
                        .min(beatmaps.len().saturating_sub(1)),
                )
                .and_then(|bm| bm.difficulty_name.clone())
        })
    }
}

