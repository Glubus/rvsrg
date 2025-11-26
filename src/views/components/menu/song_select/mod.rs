pub(super) mod beatmap_info;
pub(super) mod difficulty_card;
pub(super) mod leaderboard;
pub(super) mod leaderboard_card;
pub(super) mod song_card;
pub(super) mod song_list;

use egui::{Color32, Direction, Label, RichText, TextureId};
use egui_extras::{Size, StripBuilder};
use image::DynamicImage;
use md5::Digest;
use wgpu::TextureView;
use winit::dpi::PhysicalSize;

use crate::models::menu::MenuState;
use crate::views::components::menu::song_select::beatmap_info::BeatmapInfo;
use crate::views::components::menu::song_select::leaderboard::{Leaderboard, ScoreCard};
use crate::views::components::menu::song_select::song_list::SongList;
use crate::core::input::actions::UIAction;

pub struct CurrentBackground {
    pub image: DynamicImage,
    pub image_hash: md5::Digest,
}

pub struct SongSelectScreen {
    song_list: SongList,
    leaderboard: Leaderboard,
    beatmap_info: BeatmapInfo,
    current_background_image: Option<CurrentBackground>,
    current_beatmap_hash: Option<String>,
}

impl SongSelectScreen {
    pub fn new() -> Self {
        Self {
            song_list: SongList::new(),
            leaderboard: Leaderboard::new(),
            beatmap_info: BeatmapInfo::new(),
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
        if let Some(current_background) = &self.current_background_image {
            if current_background.image_hash == md5 {
                return;
            }
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
    ) -> Option<UIAction> {
        self.song_list.set_current(menu_state.selected_index);
        
        let mut action_triggered = None;

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
                                    let diff_name = bm.and_then(|bm| bm.difficulty_name.clone());
                                    (Some(bs.clone()), bm.cloned(), menu_state.rate, diff_name)
                                } else {
                                    (None, None, 1.0, None)
                                }
                            };

                            if let Some(bs) = &beatmapset {
                                self.beatmap_info.render(
                                    ui,
                                    bs,
                                    beatmap.as_ref(),
                                    rate,
                                    hit_window_mode,
                                    hit_window_value,
                                );
                                ui.add_space(10.0);
                            }

                            let clicked_result =
                                self.leaderboard
                                    .render(ui, diff_name.as_deref(), hit_window);

                            if let Some(_result_data) = clicked_result {
                                // TODO: Action pour voir le replay
                            }
                        });

                        strip.empty();

                        strip.strip(|builder| {
                            builder
                                .size(Size::relative(0.9))
                                .size(Size::relative(0.1))
                                .vertical(|mut strip| {
                                    strip.cell(|ui| {
                                        // On propage l'action
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
            
        action_triggered
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
}