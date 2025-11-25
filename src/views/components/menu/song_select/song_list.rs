use egui::{Align, Color32, ScrollArea, TextureId, scroll_area::ScrollBarVisibility};
use std::sync::{Arc, Mutex};

use crate::models::menu::MenuState;
use crate::views::components::menu::song_select::difficulty_card::DifficultyCard;
use crate::views::components::menu::song_select::song_card::SongCard;

// Hauteur Carte (80) + Marge (8)
const ROW_HEIGHT: f32 = 88.0;
// Hauteur Diff (30) + Marge interne (4) + Espace (4)
const DIFFICULTY_HEIGHT: f32 = 38.0;

pub struct SongList {
    menu_state: Arc<Mutex<MenuState>>,
    current: usize,
    need_scroll_to: Option<usize>,
    need_scroll_center: Option<usize>,
    min: usize,
    max: usize,
}

impl SongList {
    pub fn new(menu_state: Arc<Mutex<MenuState>>) -> Self {
        Self {
            menu_state,
            current: 0,
            need_scroll_to: None,
            need_scroll_center: None,
            min: 0,
            max: 0,
        }
    }

    pub fn set_scroll_to(&mut self, to: usize) {
        self.need_scroll_to = Some(to);
    }

    pub fn set_current(&mut self, current: usize) {
        self.current = current;
    }

    pub fn increment(&mut self) {
        let max_index = {
            if let Ok(state) = self.menu_state.lock() {
                state.beatmapsets.len().saturating_sub(1)
            } else {
                return;
            }
        };
        if self.current < max_index {
            self.set_scroll_to(self.current + 1);
        }
    }

    pub fn decrement(&mut self) {
        self.set_scroll_to(self.current.saturating_sub(1));
    }

    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        btn_tex: Option<TextureId>,
        btn_sel_tex: Option<TextureId>,
        diff_tex: Option<TextureId>,
        diff_sel_tex: Option<TextureId>,
        song_sel_color: Color32,
        diff_sel_color: Color32,
    ) {
        let (beatmapsets, current_from_state, selected_difficulty_index) = {
            let menu_state_guard = match self.menu_state.lock() {
                Ok(state) => state,
                Err(_) => return,
            };
            (
                menu_state_guard.beatmapsets.clone(),
                menu_state_guard.selected_index,
                menu_state_guard.selected_difficulty_index,
            )
        };

        self.current = current_from_state;

        let mut total_height = 0.0;
        for (i, (_, beatmaps)) in beatmapsets.iter().enumerate() {
            total_height += ROW_HEIGHT;
            if i == current_from_state && beatmaps.len() > 1 {
                total_height += DIFFICULTY_HEIGHT * (beatmaps.len() - 1) as f32;
            }
        }

        ScrollArea::vertical()
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .show_viewport(ui, |ui, rect| {
                ui.set_height(total_height);

                let mut cumulative_heights = Vec::new();
                let mut current_height = 0.0;
                for (i, (_, beatmaps)) in beatmapsets.iter().enumerate() {
                    cumulative_heights.push(current_height);
                    current_height += ROW_HEIGHT;
                    if i == current_from_state && beatmaps.len() > 1 {
                        current_height += DIFFICULTY_HEIGHT * (beatmaps.len() - 1) as f32;
                    }
                }

                if let Some(need_scroll_to) = self.need_scroll_to.take() {
                    if need_scroll_to < beatmapsets.len() {
                        let current_y =
                            cumulative_heights.get(self.current).copied().unwrap_or(0.0);
                        let target_y = cumulative_heights
                            .get(need_scroll_to)
                            .copied()
                            .unwrap_or(0.0);
                        let scroll_y = target_y - current_y;
                        self.current = need_scroll_to;

                        ui.scroll_with_delta(egui::Vec2::new(0.0, -1.0 * scroll_y));
                        self.need_scroll_center = Some(need_scroll_to);
                    }
                }

                let min_row = cumulative_heights
                    .iter()
                    .position(|&h| h >= rect.min.y)
                    .unwrap_or(0);
                let max_row = cumulative_heights
                    .iter()
                    .position(|&h| h > rect.max.y)
                    .unwrap_or(beatmapsets.len());

                let fill_top = cumulative_heights.get(min_row).copied().unwrap_or(0.0);
                egui::Frame::NONE.show(ui, |ui| {
                    ui.set_height(fill_top);
                });

                let start_idx = min_row.min(beatmapsets.len());
                let end_idx = max_row.min(beatmapsets.len());

                for i in start_idx..end_idx {
                    if let Some((beatmapset, beatmaps)) = beatmapsets.get(i) {
                        let id = i;
                        let is_selected = self.current == id;

                        let response = SongCard::render(
                            ui,
                            beatmapset,
                            beatmaps,
                            is_selected,
                            btn_tex,
                            btn_sel_tex,
                            song_sel_color,
                        );

                        let sense = response.interact(egui::Sense::click());

                        if let Some(need_scroll_center) = self.need_scroll_center {
                            if id == need_scroll_center {
                                response.scroll_to_me(Some(Align::Center));
                                let _ = self.need_scroll_center.take();
                            }
                        }

                        if sense.clicked() {
                            self.current = id;
                            {
                                if let Ok(mut state) = self.menu_state.lock() {
                                    state.selected_index = id;
                                    state.selected_difficulty_index = 0;
                                }
                            }
                            response.scroll_to_me(Some(Align::Center));
                        }

                        if sense.double_clicked() {
                            self.current = id;
                            {
                                if let Ok(mut state) = self.menu_state.lock() {
                                    state.selected_index = id;
                                    state.selected_difficulty_index = 0;
                                }
                            }
                            response.scroll_to_me(Some(Align::Center));
                        }

                        if is_selected && beatmaps.len() > 1 {
                            for (diff_idx, beatmap) in beatmaps.iter().enumerate() {
                                let is_diff_selected = diff_idx == selected_difficulty_index;
                                let diff_response = DifficultyCard::render(
                                    ui,
                                    beatmap,
                                    is_diff_selected,
                                    diff_tex,
                                    diff_sel_tex,
                                    diff_sel_color,
                                );

                                let diff_sense = diff_response.interact(egui::Sense::click());

                                if diff_sense.clicked() {
                                    if let Ok(mut state) = self.menu_state.lock() {
                                        state.selected_difficulty_index = diff_idx;
                                    }
                                }
                            }
                        }
                    }
                }

                self.min = min_row;
                self.max = max_row;
            });
    }
}
