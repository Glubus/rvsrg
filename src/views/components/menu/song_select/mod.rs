//! Song selection screen components.

#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

pub(super) mod beatmap_info;
pub(super) mod difficulty_card;
pub(super) mod leaderboard;
pub(super) mod leaderboard_card;
pub(super) mod search_panel;
pub(super) mod song_card;
pub(super) mod song_list;

// Re-export CalculatorOption for use in MenuState
pub use beatmap_info::CalculatorOption;

use egui::{Color32, Direction, Label, RichText, TextureId};
use egui_extras::{Size, StripBuilder};
use image::DynamicImage;
use md5::Digest;
use wgpu::TextureView;
use winit::dpi::PhysicalSize;

use crate::core::input::actions::UIAction;
use crate::models::menu::{GameResultData, MenuState};
use crate::models::search::MenuSearchFilters;
use crate::views::components::menu::song_select::beatmap_info::BeatmapInfo;
use crate::views::components::menu::song_select::leaderboard::{Leaderboard, ScoreCard};
use crate::views::components::menu::song_select::search_panel::{SearchPanel, SearchPanelEvent};
use crate::views::components::menu::song_select::song_list::SongList;

pub struct CurrentBackground {
    pub image: DynamicImage,
    pub image_hash: md5::Digest,
}

/// Textures for UI panel backgrounds
pub struct UIPanelTextures {
    pub beatmap_info_bg: Option<TextureId>,
    pub search_panel_bg: Option<TextureId>,
    pub search_bar: Option<TextureId>,
    pub leaderboard_bg: Option<TextureId>,
}

impl Default for UIPanelTextures {
    fn default() -> Self {
        Self {
            beatmap_info_bg: None,
            search_panel_bg: None,
            search_bar: None,
            leaderboard_bg: None,
        }
    }
}

pub struct SongSelectScreen {
    song_list: SongList,
    leaderboard: Leaderboard,
    beatmap_info: BeatmapInfo,
    search_panel: SearchPanel,
    current_background_image: Option<CurrentBackground>,
    current_beatmap_hash: Option<String>,
}

impl SongSelectScreen {
    pub fn new() -> Self {
        Self {
            song_list: SongList::new(),
            leaderboard: Leaderboard::new(),
            beatmap_info: BeatmapInfo::new(),
            search_panel: SearchPanel::new(),
            current_background_image: None,
            current_beatmap_hash: None,
        }
    }

    pub fn set_scroll_to(&mut self, to: usize) {
        self.song_list.set_scroll_to(to);
    }

    pub fn increment_beatmap(&mut self) {
        // self.song_list.increment();
    }

    pub fn decrement_beatmap(&mut self) {
        // self.song_list.decrement();
    }

    pub fn set_background(&mut self, image: DynamicImage, md5: Digest) {
        if let Some(current_background) = &self.current_background_image
            && current_background.image_hash == md5
        {
            return;
        }
        self.current_background_image = Some(CurrentBackground {
            image,
            image_hash: md5,
        });
    }

    pub fn update_leaderboard(
        &mut self,
        replays: Vec<crate::database::models::Replay>,
        note_count_map: std::collections::HashMap<String, i32>,
    ) {
        let scores: Vec<ScoreCard> = replays
            .iter()
            .filter_map(|r| {
                let total_notes =
                    note_count_map.get(&r.beatmap_hash).copied().unwrap_or(0) as usize;
                ScoreCard::from_replay(r, total_notes)
            })
            .collect();
        self.leaderboard.update_scores(scores);
    }

    pub fn set_current_beatmap_hash(&mut self, hash: Option<String>) {
        self.current_beatmap_hash = hash;
    }

    pub fn on_resize(&mut self, _new_size: &PhysicalSize<u32>) {}

    // Signature extended to optionally bubble up GameResultData.
    // Returns: (UIAction, GameResultData, SearchFilters, CalculatorChanged)
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        menu_state: &MenuState, // Immutable
        _view: &TextureView,
        _screen_width: f32,
        _screen_height: f32,
        hit_window: &crate::models::engine::hit_window::HitWindow,
        hit_window_mode: crate::models::settings::HitWindowMode,
        hit_window_value: f64,
        btn_tex: Option<TextureId>,
        btn_sel_tex: Option<TextureId>,
        diff_tex: Option<TextureId>,
        diff_sel_tex: Option<TextureId>,
        song_sel_color: Color32,
        diff_sel_color: Color32,
        panel_textures: &UIPanelTextures,
    ) -> (
        Option<UIAction>,
        Option<GameResultData>,
        Option<MenuSearchFilters>,
        Option<String>, // Calculator changed
    ) {
        self.song_list.set_current(menu_state.selected_index);

        let mut action_triggered = None;
        let mut result_data_triggered = None;
        let mut search_request = None;
        let mut calculator_changed = None;

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                StripBuilder::new(ui)
                    .size(Size::relative(0.25))
                    .size(Size::remainder())
                    .size(Size::relative(0.45))
                    .horizontal(|mut strip| {
                        strip.cell(|ui| {
                            let (beatmapset, beatmap, rate, diff_name) = {
                                if let Some((bs, beatmaps)) =
                                    menu_state.beatmapsets.get(menu_state.selected_index)
                                {
                                    let bm = beatmaps.get(menu_state.selected_difficulty_index);
                                    let diff_name =
                                        bm.and_then(|bm| bm.beatmap.difficulty_name.clone());
                                    (Some(bs.clone()), bm.cloned(), menu_state.rate, diff_name)
                                } else {
                                    (None, None, 1.0, None)
                                }
                            };

                            if let Some(bm) = beatmap.as_ref() {
                                self.refresh_leaderboard(
                                    menu_state,
                                    &bm.beatmap.hash,
                                    bm.beatmap.note_count as usize,
                                );
                            } else {
                                self.leaderboard.update_scores(Vec::new());
                            }

                            if let Some(bs) = &beatmapset {
                                let rate_specific_ratings = beatmap.as_ref().and_then(|bm| {
                                    menu_state.get_cached_ratings_for(&bm.beatmap.hash, rate)
                                });

                                // Get current difficulty from cache (for custom calculators)
                                let current_ssr = menu_state.get_current_difficulty();

                                if let Some(new_calc) = self.beatmap_info.render(
                                    ui,
                                    bs,
                                    beatmap.as_ref(),
                                    rate,
                                    hit_window_mode,
                                    hit_window_value,
                                    rate_specific_ratings,
                                    panel_textures.beatmap_info_bg,
                                    &menu_state.available_calculators,
                                    &menu_state.active_calculator,
                                    current_ssr,
                                ) {
                                    calculator_changed = Some(new_calc);
                                }
                                ui.add_space(10.0);
                            }

                            // Capture the leaderboard click result if any.
                            // Passer la chart cachée pour permettre le recalcul des replays.
                            let cached_chart =
                                menu_state.get_cached_chart().map(|c| c.chart.as_slice());

                            let clicked_result = self.leaderboard.render(
                                ui,
                                diff_name.as_deref(),
                                hit_window,
                                cached_chart,
                            );

                            if let Some(result_data) = clicked_result {
                                result_data_triggered = Some(result_data);
                            }
                        });

                        strip.empty();

                        strip.strip(|builder| {
                            builder
                                .size(Size::relative(0.9))
                                .size(Size::relative(0.1))
                                .vertical(|mut strip| {
                                    strip.cell(|ui| {
                                        ui.vertical(|ui| {
                                            match self.search_panel.render(
                                                ui,
                                                menu_state,
                                                panel_textures.search_panel_bg,
                                                panel_textures.search_bar,
                                            ) {
                                                SearchPanelEvent::Apply(filters) => {
                                                    search_request = Some(filters);
                                                }
                                                SearchPanelEvent::None => {}
                                            }

                                            ui.add_space(8.0);
                                            action_triggered = self.song_list.render(
                                                ui,
                                                menu_state,
                                                btn_tex,
                                                btn_sel_tex,
                                                diff_tex,
                                                diff_sel_tex,
                                                song_sel_color,
                                                diff_sel_color,
                                            );
                                        });
                                    });

                                    strip.cell(|ui| {
                                        egui::Frame::default()
                                            .corner_radius(5.0)
                                            .outer_margin(10.0)
                                            .inner_margin(5.0)
                                            .fill(Color32::from_rgba_unmultiplied(0, 0, 0, 255))
                                            .show(ui, |ui| {
                                                ui.set_width(
                                                    ui.available_rect_before_wrap().width(),
                                                );
                                                ui.set_height(
                                                    ui.available_rect_before_wrap().height(),
                                                );
                                                self.render_beatmap_footer(ui, menu_state);
                                            });
                                    })
                                });
                        });
                    })
            });

        (
            action_triggered,
            result_data_triggered,
            search_request,
            calculator_changed,
        )
    }

    fn render_beatmap_footer(&mut self, ui: &mut egui::Ui, menu_state: &MenuState) {
        ui.with_layout(
            egui::Layout::centered_and_justified(Direction::LeftToRight),
            |ui| {
                let beatmap_count = menu_state.beatmapsets.len();
                let text = format!("Beatmaps: {}", beatmap_count);
                ui.add(Label::new(RichText::new(text).heading()).selectable(false));
            },
        );
    }

    fn refresh_leaderboard(
        &mut self,
        menu_state: &MenuState,
        beatmap_hash: &str,
        total_notes: usize,
    ) {
        if menu_state.leaderboard_hash.as_deref() == Some(beatmap_hash) {
            let cards = menu_state
                .leaderboard_scores
                .iter()
                .filter_map(|replay| ScoreCard::from_replay(replay, total_notes))
                .collect();
            self.leaderboard.update_scores(cards);
        } else {
            self.leaderboard.update_scores(Vec::new());
        }
    }
}
