//! Beatmap information panel with customizable skin colors and background.

use egui::{
    Color32, ComboBox, CornerRadius, Frame, Margin, Pos2, Rect, RichText, Stroke, TextureId, Ui,
    Vec2,
};

use crate::database::models::{BeatmapRating, BeatmapWithRatings, Beatmapset};
use crate::difficulty::BeatmapSsr;
use crate::models::settings::HitWindowMode;

/// UI color configuration for the beatmap info panel.
#[derive(Clone)]
pub struct BeatmapInfoColors {
    pub panel_bg: Color32,
    pub panel_secondary: Color32,
    pub panel_border: Color32,
    pub accent: Color32,
    pub accent_dim: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub rating_stream: Color32,
    pub rating_jumpstream: Color32,
    pub rating_handstream: Color32,
    pub rating_stamina: Color32,
    pub rating_jackspeed: Color32,
    pub rating_chordjack: Color32,
    pub rating_technical: Color32,
}

impl Default for BeatmapInfoColors {
    fn default() -> Self {
        Self {
            panel_bg: Color32::from_rgba_unmultiplied(20, 20, 26, 242),
            panel_secondary: Color32::from_rgba_unmultiplied(31, 31, 38, 230),
            panel_border: Color32::from_rgba_unmultiplied(64, 64, 77, 204),
            accent: Color32::from_rgba_unmultiplied(102, 179, 255, 255),
            accent_dim: Color32::from_rgba_unmultiplied(64, 115, 179, 255),
            text_primary: Color32::WHITE,
            text_secondary: Color32::from_rgba_unmultiplied(191, 191, 204, 255),
            text_muted: Color32::from_rgba_unmultiplied(128, 128, 140, 255),
            rating_stream: Color32::from_rgba_unmultiplied(77, 217, 128, 255),
            rating_jumpstream: Color32::from_rgba_unmultiplied(242, 191, 51, 255),
            rating_handstream: Color32::from_rgba_unmultiplied(230, 115, 77, 255),
            rating_stamina: Color32::from_rgba_unmultiplied(217, 77, 140, 255),
            rating_jackspeed: Color32::from_rgba_unmultiplied(153, 102, 230, 255),
            rating_chordjack: Color32::from_rgba_unmultiplied(102, 153, 242, 255),
            rating_technical: Color32::from_rgba_unmultiplied(51, 204, 217, 255),
        }
    }
}

/// Calculator info for the dropdown.
#[derive(Clone, Debug)]
pub struct CalculatorOption {
    pub id: String,
    pub display_name: String,
}

impl CalculatorOption {
    pub fn new(id: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
        }
    }
}

pub struct BeatmapInfo {
    colors: BeatmapInfoColors,
    /// Whether the pattern breakdown section is expanded
    pattern_breakdown_expanded: bool,
}

impl BeatmapInfo {
    pub fn new() -> Self {
        Self {
            colors: BeatmapInfoColors::default(),
            pattern_breakdown_expanded: false,
        }
    }

    /// Update colors from skin configuration.
    pub fn set_colors(&mut self, colors: BeatmapInfoColors) {
        self.colors = colors;
    }

    /// Renders the beatmap info panel.
    ///
    /// `active_calculator` - the currently selected calculator ID from MenuState
    /// `current_ssr` - the calculated SSR for the active calculator (from difficulty_cache)
    /// Returns the new calculator ID if the user changed it via dropdown
    pub fn render(
        &mut self,
        ui: &mut Ui,
        _beatmapset: &Beatmapset,
        beatmap: Option<&BeatmapWithRatings>,
        rate: f64,
        hit_window_mode: HitWindowMode,
        hit_window_value: f64,
        override_ratings: Option<&[BeatmapRating]>,
        background_texture: Option<TextureId>,
        available_calculators: &[CalculatorOption],
        active_calculator: &str,
        current_ssr: Option<&BeatmapSsr>,
    ) -> Option<String> {
        let colors = self.colors.clone();
        let rounding = CornerRadius::same(12);
        let margin = Margin::symmetric(8, 6);
        let mut calculator_changed: Option<String> = None;

        let available_rect = ui.available_rect_before_wrap();
        let panel_rect = Rect::from_min_size(
            available_rect.min + Vec2::new(margin.left as f32, margin.top as f32),
            Vec2::new(
                available_rect.width() - margin.left as f32 - margin.right as f32,
                available_rect.height().min(500.0),
            ),
        );

        if let Some(bg_tex) = background_texture {
            ui.painter().image(
                bg_tex,
                panel_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
            ui.painter().rect_filled(
                panel_rect,
                rounding,
                Color32::from_rgba_unmultiplied(0, 0, 0, 160),
            );
        }

        Frame::default()
            .corner_radius(rounding)
            .outer_margin(margin)
            .inner_margin(Margin::same(0))
            .fill(if background_texture.is_some() {
                Color32::TRANSPARENT
            } else {
                colors.panel_bg
            })
            .stroke(Stroke::new(1.0, colors.panel_border))
            .show(ui, |ui| {
                ui.set_width(ui.available_rect_before_wrap().width());

                // Header section with difficulty name
                if let Some(bm) = beatmap
                    && let Some(diff_name) = &bm.beatmap.difficulty_name
                {
                    Frame::default()
                        .corner_radius(CornerRadius {
                            nw: 12,
                            ne: 12,
                            sw: 0,
                            se: 0,
                        })
                        .inner_margin(Margin::symmetric(16, 10))
                        .fill(if background_texture.is_some() {
                            Color32::from_rgba_unmultiplied(0, 0, 0, 100)
                        } else {
                            colors.panel_secondary
                        })
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let bar_rect = ui.available_rect_before_wrap();
                                let bar_rect =
                                    Rect::from_min_size(bar_rect.min, Vec2::new(4.0, 24.0));
                                ui.painter().rect_filled(
                                    bar_rect,
                                    CornerRadius::same(2),
                                    colors.accent,
                                );
                                ui.add_space(12.0);

                                ui.label(
                                    RichText::new(diff_name)
                                        .size(20.0)
                                        .strong()
                                        .color(colors.text_primary),
                                );
                            });
                        });
                }

                // Content section
                Frame::default()
                    .inner_margin(Margin::symmetric(14, 10))
                    .show(ui, |ui| {
                        // Metadata row
                        self.render_metadata_row(
                            ui,
                            beatmap,
                            &colors,
                            background_texture.is_some(),
                            rate,
                        );

                        ui.add_space(10.0);

                        // Calculator dropdown + Rate display on same line
                        ui.horizontal(|ui| {
                            if let Some(new_calc) = self.render_calculator_dropdown(
                                ui,
                                &colors,
                                background_texture.is_some(),
                                available_calculators,
                                active_calculator,
                            ) {
                                calculator_changed = Some(new_calc);
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    self.render_rate_badge(
                                        ui,
                                        rate,
                                        &colors,
                                        background_texture.is_some(),
                                    );
                                    ui.add_space(6.0);
                                    self.render_hit_window_badge(
                                        ui,
                                        hit_window_mode,
                                        hit_window_value,
                                        &colors,
                                        background_texture.is_some(),
                                    );
                                },
                            );
                        });

                        // Rating details
                        let ratings_slice =
                            override_ratings.or_else(|| beatmap.map(|bm| bm.ratings.as_slice()));

                        let active_rating = find_rating(ratings_slice, active_calculator);

                        // Use current_ssr if we have it, otherwise fall back to active_rating
                        if let Some(ssr) = current_ssr {
                            ui.add_space(10.0);

                            // Overall rating display from SSR
                            self.render_overall_rating_from_ssr(ui, ssr, &colors);

                            ui.add_space(8.0);

                            // Collapsible SSR breakdown
                            self.render_collapsible_breakdown_from_ssr(
                                ui,
                                ssr,
                                &colors,
                                background_texture.is_some(),
                            );
                        } else if let Some(rating) = active_rating {
                            ui.add_space(10.0);

                            // Overall rating display
                            self.render_overall_rating(ui, rating, &colors);

                            ui.add_space(8.0);

                            // Collapsible SSR breakdown
                            self.render_collapsible_breakdown(
                                ui,
                                rating,
                                &colors,
                                background_texture.is_some(),
                            );
                        } else {
                            ui.add_space(12.0);
                            ui.centered_and_justified(|ui| {
                                ui.label(
                                    RichText::new("No rating data")
                                        .size(13.0)
                                        .italics()
                                        .color(colors.text_muted),
                                );
                            });
                        }
                    });
            });

        calculator_changed
    }

    fn render_metadata_row(
        &self,
        ui: &mut Ui,
        beatmap: Option<&BeatmapWithRatings>,
        colors: &BeatmapInfoColors,
        has_bg: bool,
        rate: f64,
    ) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(6.0, 4.0);

            let badge_bg = if has_bg {
                Color32::from_rgba_unmultiplied(0, 0, 0, 140)
            } else {
                colors.accent_dim
            };

            if let Some(bm) = beatmap {
                self.render_badge(
                    ui,
                    "♫",
                    &format!("{}", bm.beatmap.note_count),
                    badge_bg,
                    colors,
                );

                // Display BPM adjusted for current rate
                let effective_bpm = bm.beatmap.bpm * rate;
                self.render_badge(
                    ui,
                    "BPM",
                    &format!("{:.0}", effective_bpm),
                    badge_bg,
                    colors,
                );
            }
        });
    }

    fn render_badge(
        &self,
        ui: &mut Ui,
        icon: &str,
        value: &str,
        bg_color: Color32,
        colors: &BeatmapInfoColors,
    ) {
        Frame::default()
            .corner_radius(CornerRadius::same(5))
            .inner_margin(Margin::symmetric(8, 4))
            .fill(bg_color)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.label(RichText::new(icon).size(11.0).color(colors.text_secondary));
                    ui.label(
                        RichText::new(value)
                            .size(12.0)
                            .strong()
                            .color(colors.text_primary),
                    );
                });
            });
    }

    fn render_calculator_dropdown(
        &self,
        ui: &mut Ui,
        colors: &BeatmapInfoColors,
        has_bg: bool,
        available_calculators: &[CalculatorOption],
        active_calculator: &str,
    ) -> Option<String> {
        let mut changed: Option<String> = None;

        let bg = if has_bg {
            Color32::from_rgba_unmultiplied(0, 0, 0, 120)
        } else {
            colors.panel_secondary
        };

        // Find current display name
        let current_name = available_calculators
            .iter()
            .find(|c| c.id == active_calculator)
            .map(|c| c.display_name.clone())
            .unwrap_or_else(|| active_calculator.to_string());

        Frame::default()
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::symmetric(2, 0))
            .fill(bg)
            .stroke(Stroke::new(1.0, colors.panel_border))
            .show(ui, |ui| {
                ComboBox::from_id_salt("calculator_select")
                    .selected_text(
                        RichText::new(&current_name)
                            .size(11.0)
                            .color(colors.text_primary),
                    )
                    .width(120.0)
                    .show_ui(ui, |ui| {
                        for calc in available_calculators {
                            let is_selected = active_calculator == calc.id;
                            if ui
                                .selectable_label(is_selected, &calc.display_name)
                                .clicked()
                            {
                                if active_calculator != calc.id {
                                    changed = Some(calc.id.clone());
                                }
                            }
                        }
                    });
            });

        changed
    }

    fn render_hit_window_badge(
        &self,
        ui: &mut Ui,
        hit_window_mode: HitWindowMode,
        hit_window_value: f64,
        colors: &BeatmapInfoColors,
        has_bg: bool,
    ) {
        let hit_window_text = match hit_window_mode {
            HitWindowMode::OsuOD => format!("OD {:.1}", hit_window_value),
            HitWindowMode::EtternaJudge => format!("J{}", hit_window_value as u8),
        };

        let bg = if has_bg {
            Color32::from_rgba_unmultiplied(0, 0, 0, 120)
        } else {
            colors.panel_secondary
        };

        Frame::default()
            .corner_radius(CornerRadius::same(4))
            .inner_margin(Margin::symmetric(6, 3))
            .fill(bg)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(&hit_window_text)
                        .size(10.0)
                        .color(colors.text_muted),
                );
            });
    }

    fn render_rate_badge(&self, ui: &mut Ui, rate: f64, colors: &BeatmapInfoColors, has_bg: bool) {
        let is_modified = (rate - 1.0).abs() > 0.01;
        let bg = if is_modified {
            colors.accent
        } else if has_bg {
            Color32::from_rgba_unmultiplied(0, 0, 0, 120)
        } else {
            colors.panel_secondary
        };

        Frame::default()
            .corner_radius(CornerRadius::same(5))
            .inner_margin(Margin::symmetric(8, 4))
            .fill(bg)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!("{:.2}x", rate))
                        .size(13.0)
                        .strong()
                        .color(if is_modified {
                            colors.panel_bg
                        } else {
                            colors.text_primary
                        }),
                );
            });
    }

    fn render_overall_rating(
        &self,
        ui: &mut Ui,
        rating: &BeatmapRating,
        colors: &BeatmapInfoColors,
    ) {
        self.render_overall_value(ui, rating.overall, colors);
    }

    fn render_overall_rating_from_ssr(
        &self,
        ui: &mut Ui,
        ssr: &BeatmapSsr,
        colors: &BeatmapInfoColors,
    ) {
        self.render_overall_value(ui, ssr.overall, colors);
    }

    fn render_overall_value(&self, ui: &mut Ui, overall: f64, colors: &BeatmapInfoColors) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Overall")
                    .size(13.0)
                    .color(colors.text_secondary),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{:.2}", overall))
                        .size(26.0)
                        .strong()
                        .color(self.get_difficulty_color(overall, colors)),
                );
            });
        });
    }

    fn get_difficulty_color(&self, rating: f64, colors: &BeatmapInfoColors) -> Color32 {
        match rating {
            r if r < 15.0 => colors.rating_stream,
            r if r < 22.0 => colors.rating_jumpstream,
            r if r < 28.0 => colors.rating_handstream,
            r if r < 34.0 => colors.rating_stamina,
            _ => colors.rating_jackspeed,
        }
    }

    fn render_collapsible_breakdown(
        &mut self,
        ui: &mut Ui,
        rating: &BeatmapRating,
        colors: &BeatmapInfoColors,
        has_bg: bool,
    ) {
        let metrics = [
            ("Stream", rating.stream, colors.rating_stream),
            ("JS", rating.jumpstream, colors.rating_jumpstream),
            ("HS", rating.handstream, colors.rating_handstream),
            ("Stamina", rating.stamina, colors.rating_stamina),
            ("Jack", rating.jackspeed, colors.rating_jackspeed),
            ("CJ", rating.chordjack, colors.rating_chordjack),
            ("Tech", rating.technical, colors.rating_technical),
        ];
        self.render_collapsible_breakdown_impl(ui, &metrics, colors, has_bg);
    }

    fn render_collapsible_breakdown_from_ssr(
        &mut self,
        ui: &mut Ui,
        ssr: &BeatmapSsr,
        colors: &BeatmapInfoColors,
        has_bg: bool,
    ) {
        let metrics = [
            ("Stream", ssr.stream, colors.rating_stream),
            ("JS", ssr.jumpstream, colors.rating_jumpstream),
            ("HS", ssr.handstream, colors.rating_handstream),
            ("Stamina", ssr.stamina, colors.rating_stamina),
            ("Jack", ssr.jackspeed, colors.rating_jackspeed),
            ("CJ", ssr.chordjack, colors.rating_chordjack),
            ("Tech", ssr.technical, colors.rating_technical),
        ];
        self.render_collapsible_breakdown_impl(ui, &metrics, colors, has_bg);
    }

    fn render_collapsible_breakdown_impl(
        &mut self,
        ui: &mut Ui,
        metrics: &[(&str, f64, Color32); 7],
        colors: &BeatmapInfoColors,
        has_bg: bool,
    ) {
        let header_bg = if has_bg {
            Color32::from_rgba_unmultiplied(0, 0, 0, 80)
        } else {
            colors.panel_secondary
        };

        // Collapsible header
        let header_response = Frame::default()
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::symmetric(10, 6))
            .fill(header_bg)
            .stroke(Stroke::new(1.0, colors.panel_border))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Arrow indicator
                    let arrow = if self.pattern_breakdown_expanded {
                        "▼"
                    } else {
                        "▶"
                    };
                    ui.label(RichText::new(arrow).size(10.0).color(colors.accent));
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Pattern Breakdown")
                            .size(11.0)
                            .color(colors.text_secondary),
                    );
                });
            })
            .response;

        if header_response.interact(egui::Sense::click()).clicked() {
            self.pattern_breakdown_expanded = !self.pattern_breakdown_expanded;
        }

        // Content (only if expanded)
        if self.pattern_breakdown_expanded {
            ui.add_space(6.0);
            self.render_ssr_breakdown_impl(ui, metrics, colors, has_bg);
        }
    }

    fn render_ssr_breakdown(
        &self,
        ui: &mut Ui,
        rating: &BeatmapRating,
        colors: &BeatmapInfoColors,
        has_bg: bool,
    ) {
        let metrics = [
            ("Stream", rating.stream, colors.rating_stream),
            ("JS", rating.jumpstream, colors.rating_jumpstream),
            ("HS", rating.handstream, colors.rating_handstream),
            ("Stamina", rating.stamina, colors.rating_stamina),
            ("Jack", rating.jackspeed, colors.rating_jackspeed),
            ("CJ", rating.chordjack, colors.rating_chordjack),
            ("Tech", rating.technical, colors.rating_technical),
        ];
        self.render_ssr_breakdown_impl(ui, &metrics, colors, has_bg);
    }

    fn render_ssr_breakdown_impl(
        &self,
        ui: &mut Ui,
        metrics: &[(&str, f64, Color32); 7],
        colors: &BeatmapInfoColors,
        has_bg: bool,
    ) {
        let max_val = metrics
            .iter()
            .map(|(_, v, _)| *v)
            .fold(0.0f64, |a, b| a.max(b))
            .max(1.0);

        for (name, value, color) in metrics {
            if *value > 0.01 {
                self.render_metric_bar(ui, name, *value, max_val, *color, colors, has_bg);
                ui.add_space(3.0);
            }
        }
    }

    fn render_metric_bar(
        &self,
        ui: &mut Ui,
        name: &str,
        value: f64,
        max_value: f64,
        bar_color: Color32,
        colors: &BeatmapInfoColors,
        has_bg: bool,
    ) {
        let available_width = ui.available_width();
        let bar_width = ((value / max_value) * (available_width - 70.0) as f64) as f32;

        let bar_bg = if has_bg {
            Color32::from_rgba_unmultiplied(0, 0, 0, 100)
        } else {
            colors.panel_secondary
        };

        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(40.0, 14.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.label(RichText::new(name).size(10.0).color(colors.text_secondary));
                },
            );

            let (rect, _) = ui.allocate_exact_size(
                Vec2::new(available_width - 70.0, 12.0),
                egui::Sense::hover(),
            );

            ui.painter()
                .rect_filled(rect, CornerRadius::same(3), bar_bg);

            let fill_rect =
                Rect::from_min_size(rect.min, Vec2::new(bar_width.max(4.0), rect.height()));
            ui.painter()
                .rect_filled(fill_rect, CornerRadius::same(3), bar_color);

            ui.label(
                RichText::new(format!("{:.1}", value))
                    .size(10.0)
                    .strong()
                    .color(colors.text_primary),
            );
        });
    }
}

fn find_rating<'a>(
    ratings: Option<&'a [BeatmapRating]>,
    target: &str,
) -> Option<&'a BeatmapRating> {
    ratings.and_then(|list| {
        list.iter()
            .find(|rating| rating.name.eq_ignore_ascii_case(target))
    })
}

/// Default calculators (builtin).
pub fn default_calculators() -> Vec<CalculatorOption> {
    vec![
        CalculatorOption::new("etterna", "Etterna"),
        CalculatorOption::new("osu", "osu!"),
    ]
}
