use crate::database::models::BeatmapWithRatings;
use crate::input::events::GameAction;
use crate::models::skin::menus::song_select::RatingColorsConfig;
use crate::state::MenuState;
use crate::views::components::menu::song_select::difficulty_card::DifficultyCard;
use crate::views::components::menu::song_select::difficulty_utils::{
    get_beatmap_rating, get_difficulty_color,
};
use crate::views::components::menu::song_select::song_card::SongCard;
use egui::{Align, Color32, ScrollArea, TextureId, scroll_area::ScrollBarVisibility};

// Hauteur Carte (80) + Marge (8)
const ROW_HEIGHT: f32 = 88.0;
// Hauteur Diff (35) + Marge interne (4)
const DIFFICULTY_HEIGHT: f32 = 39.0;

pub struct SongList {
    current: usize,
    previous_selection: usize,
    need_scroll_center: bool,
    /// Animation progress for selection transition (0.0 to 1.0)
    selection_anim: f32,
    min: usize,
    max: usize,
}

impl SongList {
    pub fn new() -> Self {
        Self {
            current: 0,
            previous_selection: 0,
            need_scroll_center: true, // Center on first render
            selection_anim: 1.0,
            min: 0,
            max: 0,
        }
    }

    pub fn set_scroll_to(&mut self, _to: usize) {
        self.need_scroll_center = true;
    }

    pub fn set_current(&mut self, current: usize) {
        if self.current != current {
            self.previous_selection = self.current;
            self.current = current;
            self.need_scroll_center = true;
            self.selection_anim = 0.0; // Start animation
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        menu_state: &MenuState,
        btn_tex: Option<TextureId>,
        btn_sel_tex: Option<TextureId>,
        diff_tex: Option<TextureId>,
        diff_sel_tex: Option<TextureId>,
        song_sel_color: Color32,
        diff_sel_color: Color32,
        rating_colors: Option<&RatingColorsConfig>,
    ) -> Option<GameAction> {
        let beatmapsets = &menu_state.beatmapsets;
        let current_from_state = menu_state.selected_index;
        let selected_difficulty_index = menu_state.selected_difficulty_index;
        let active_calculator = &menu_state.active_calculator;
        let mut action_triggered = None;

        // Detect selection change from state
        if self.current != current_from_state {
            self.previous_selection = self.current;
            self.current = current_from_state;
            self.need_scroll_center = true;
            self.selection_anim = 0.0;
        }

        // Animate selection transition
        if self.selection_anim < 1.0 {
            self.selection_anim = (self.selection_anim + 0.15).min(1.0);
            ui.ctx().request_repaint(); // Keep animating
        }

        let mut total_height = 0.0;
        for (i, (_, beatmaps)) in beatmapsets.iter().enumerate() {
            total_height += ROW_HEIGHT;
            if i == current_from_state && beatmaps.len() > 1 {
                total_height += DIFFICULTY_HEIGHT * (beatmaps.len() - 1) as f32;
            }
        }

        ScrollArea::vertical()
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .animated(true) // Enable smooth scrolling
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

                        // Apply selection animation opacity
                        let anim_alpha = if is_selected {
                            self.selection_anim
                        } else if id == self.previous_selection && self.selection_anim < 1.0 {
                            1.0 - self.selection_anim
                        } else {
                            0.0
                        };

                        // Blend selection color with animation
                        let animated_sel_color = if anim_alpha > 0.0 {
                            Color32::from_rgba_unmultiplied(
                                song_sel_color.r(),
                                song_sel_color.g(),
                                song_sel_color.b(),
                                (song_sel_color.a() as f32 * anim_alpha) as u8,
                            )
                        } else {
                            song_sel_color
                        };

                        let response = SongCard::render(
                            ui,
                            beatmapset,
                            beatmaps,
                            is_selected,
                            btn_tex,
                            btn_sel_tex,
                            animated_sel_color,
                            rating_colors,
                            active_calculator,
                        );

                        // Auto-center selected item when selection changes
                        if is_selected && self.need_scroll_center {
                            response.scroll_to_me(Some(Align::Center));
                            self.need_scroll_center = false;
                        }

                        let sense = response.interact(egui::Sense::click());

                        if sense.clicked() || sense.double_clicked() {
                            action_triggered = Some(GameAction::SetSelection(id));
                            response.scroll_to_me(Some(Align::Center));
                            ui.ctx().memory_mut(|m| m.surrender_focus(response.id));
                        }

                        if is_selected && beatmaps.len() > 1 {
                            for (diff_idx, beatmap) in beatmaps.iter().enumerate() {
                                let is_diff_selected = diff_idx == selected_difficulty_index;

                                // Get the difficulty rating and color for this beatmap
                                let rating = get_beatmap_rating(beatmap, active_calculator);
                                let diff_color =
                                    Self::get_diff_color_from_rating(rating, rating_colors);

                                let diff_response = DifficultyCard::render(
                                    ui,
                                    beatmap,
                                    is_diff_selected,
                                    diff_tex,
                                    diff_sel_tex,
                                    diff_sel_color,
                                    diff_color,
                                    rating,
                                );

                                let diff_sense = diff_response.interact(egui::Sense::click());

                                if diff_sense.clicked() {
                                    action_triggered = Some(GameAction::SetDifficulty(diff_idx));
                                    ui.ctx().memory_mut(|m| m.surrender_focus(diff_response.id));
                                }
                            }
                        }
                    }
                }

                self.min = min_row;
                self.max = max_row;
            });

        action_triggered
    }

    fn get_diff_color_from_rating(
        rating: Option<f64>,
        rating_colors: Option<&RatingColorsConfig>,
    ) -> Color32 {
        let Some(colors) = rating_colors else {
            return Color32::GRAY;
        };

        let r = rating.unwrap_or(0.0);
        get_difficulty_color(r, colors)
    }
}
