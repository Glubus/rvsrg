//! Filter sidebar with customizable skin colors and background images.

use egui::{
    Color32, ComboBox, CornerRadius, Frame, Margin, Pos2, Rect, RichText, Slider, Stroke,
    StrokeKind, TextEdit, TextureId, Ui, Vec2,
};

use crate::models::menu::MenuState;
use crate::models::search::{MenuSearchFilters, RatingMetric, RatingSource};

/// Message emitted by the search panel when the user applies filters.
pub enum SearchPanelEvent {
    None,
    Apply(MenuSearchFilters),
}

/// UI color configuration for the search panel.
#[derive(Clone)]
pub struct SearchPanelColors {
    pub panel_bg: Color32,
    pub panel_secondary: Color32,
    pub panel_border: Color32,
    pub accent: Color32,
    pub accent_dim: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub search_active: Color32,
}

impl Default for SearchPanelColors {
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
            search_active: Color32::from_rgba_unmultiplied(77, 191, 242, 255),
        }
    }
}

/// Stateful form mirroring `MenuSearchFilters`.
pub struct SearchPanel {
    form_filters: MenuSearchFilters,
    colors: SearchPanelColors,
    /// Whether the source/metric section is expanded
    source_metric_expanded: bool,
    /// Whether the filters section is expanded
    filters_expanded: bool,
}

impl SearchPanel {
    /// Returns a panel with default (inactive) filters.
    pub fn new() -> Self {
        Self {
            form_filters: MenuSearchFilters::default(),
            colors: SearchPanelColors::default(),
            source_metric_expanded: false,
            filters_expanded: false,
        }
    }

    /// Update colors from skin configuration.
    pub fn set_colors(&mut self, colors: SearchPanelColors) {
        self.colors = colors;
    }

    /// Draws the panel and returns an event when the user applies filters.
    pub fn render(
        &mut self,
        ui: &mut Ui,
        menu_state: &MenuState,
        background_texture: Option<TextureId>,
        search_bar_texture: Option<TextureId>,
    ) -> SearchPanelEvent {
        let mut should_apply = false;
        let colors = self.colors.clone();
        let rounding = CornerRadius::same(12);

        let available_rect = ui.available_rect_before_wrap();
        let margin = Margin::symmetric(0, 4);
        let panel_height = 200.0;
        let panel_rect = Rect::from_min_size(
            available_rect.min + Vec2::new(margin.left as f32, margin.top as f32),
            Vec2::new(
                available_rect.width() - margin.left as f32 - margin.right as f32,
                panel_height,
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
                Color32::from_rgba_unmultiplied(0, 0, 0, 140),
            );
        }

        let has_bg = background_texture.is_some();

        Frame::default()
            .corner_radius(rounding)
            .outer_margin(margin)
            .inner_margin(Margin::symmetric(12, 10))
            .fill(if has_bg {
                Color32::TRANSPARENT
            } else {
                colors.panel_bg
            })
            .stroke(Stroke::new(1.0, colors.panel_border))
            .show(ui, |ui| {
                // Header with title and active indicator
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🔍").size(16.0).color(colors.accent));
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Search")
                            .size(14.0)
                            .strong()
                            .color(colors.text_primary),
                    );

                    if menu_state.search_filters.is_active() {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            Frame::default()
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::symmetric(6, 2))
                                .fill(colors.search_active)
                                .show(ui, |ui| {
                                    ui.label(RichText::new("●").size(8.0).color(colors.panel_bg));
                                });
                        });
                    }
                });

                ui.add_space(8.0);

                // Search bar
                should_apply |= self.render_search_bar(ui, &colors, search_bar_texture, has_bg);

                ui.add_space(8.0);

                // Collapsible: Source & Metric
                should_apply |= self.render_collapsible_source_metric(ui, &colors, has_bg);

                ui.add_space(6.0);

                // Collapsible: Filters (Rating + Duration)
                should_apply |= self.render_collapsible_filters(ui, &colors, has_bg);
            });

        if should_apply {
            SearchPanelEvent::Apply(self.form_filters.clone())
        } else {
            if self.form_filters != menu_state.search_filters {
                self.form_filters = menu_state.search_filters.clone();
            }
            SearchPanelEvent::None
        }
    }

    fn render_search_bar(
        &mut self,
        ui: &mut Ui,
        colors: &SearchPanelColors,
        search_bar_texture: Option<TextureId>,
        has_bg: bool,
    ) -> bool {
        let available_width = ui.available_width();
        let bar_height = 32.0;

        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(available_width, bar_height), egui::Sense::hover());

        if let Some(bar_tex) = search_bar_texture {
            ui.painter().image(
                bar_tex,
                rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            let bar_bg = if has_bg {
                Color32::from_rgba_unmultiplied(0, 0, 0, 160)
            } else {
                colors.panel_secondary
            };
            ui.painter()
                .rect_filled(rect, CornerRadius::same(8), bar_bg);
            ui.painter().rect_stroke(
                rect,
                CornerRadius::same(8),
                Stroke::new(1.0, colors.panel_border),
                StrokeKind::Inside,
            );
        }

        let inner_rect = rect.shrink2(Vec2::new(10.0, 5.0));
        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(inner_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );

        child_ui.label(RichText::new("⌕").size(12.0).color(colors.text_muted));
        child_ui.add_space(4.0);

        let text_edit = TextEdit::singleline(&mut self.form_filters.query)
            .hint_text(
                RichText::new("Artist, title...")
                    .color(colors.text_muted)
                    .size(11.0),
            )
            .text_color(colors.text_primary)
            .frame(false)
            .desired_width(available_width - 40.0);

        child_ui.add(text_edit).changed()
    }

    fn render_collapsible_source_metric(
        &mut self,
        ui: &mut Ui,
        colors: &SearchPanelColors,
        has_bg: bool,
    ) -> bool {
        let mut changed = false;

        let header_bg = if has_bg {
            Color32::from_rgba_unmultiplied(0, 0, 0, 80)
        } else {
            colors.panel_secondary
        };

        // Collapsible header
        let header_response = Frame::default()
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::symmetric(8, 5))
            .fill(header_bg)
            .stroke(Stroke::new(1.0, colors.panel_border))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let arrow = if self.source_metric_expanded {
                        "▼"
                    } else {
                        "▶"
                    };
                    ui.label(RichText::new(arrow).size(9.0).color(colors.accent));
                    ui.add_space(3.0);
                    ui.label(
                        RichText::new("Source & Metric")
                            .size(11.0)
                            .color(colors.text_secondary),
                    );

                    // Show current values in collapsed state
                    if !self.source_metric_expanded {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!(
                                    "{} · {}",
                                    match self.form_filters.rating_source {
                                        RatingSource::Etterna => "Ett",
                                        RatingSource::Osu => "osu",
                                    },
                                    self.form_filters.rating_metric.display_name()
                                ))
                                .size(10.0)
                                .color(colors.text_muted),
                            );
                        });
                    }
                });
            })
            .response;

        if header_response.interact(egui::Sense::click()).clicked() {
            self.source_metric_expanded = !self.source_metric_expanded;
        }

        // Content (only if expanded)
        if self.source_metric_expanded {
            ui.add_space(4.0);
            changed |= self.render_source_and_metric(ui, colors, has_bg);
        }

        changed
    }

    fn render_source_and_metric(
        &mut self,
        ui: &mut Ui,
        colors: &SearchPanelColors,
        has_bg: bool,
    ) -> bool {
        let mut changed = false;
        let dropdown_bg = if has_bg {
            Color32::from_rgba_unmultiplied(0, 0, 0, 140)
        } else {
            colors.panel_secondary
        };

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(6.0, 4.0);

            Frame::default()
                .corner_radius(CornerRadius::same(5))
                .inner_margin(Margin::symmetric(6, 3))
                .fill(dropdown_bg)
                .stroke(Stroke::new(1.0, colors.panel_border))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Source:").size(10.0).color(colors.text_muted));

                        ComboBox::from_id_salt("source_combo")
                            .selected_text(
                                RichText::new(match self.form_filters.rating_source {
                                    RatingSource::Etterna => "Etterna",
                                    RatingSource::Osu => "osu!",
                                })
                                .size(10.0)
                                .color(colors.text_primary),
                            )
                            .show_ui(ui, |ui| {
                                changed |= ui
                                    .selectable_value(
                                        &mut self.form_filters.rating_source,
                                        RatingSource::Etterna,
                                        "Etterna",
                                    )
                                    .changed();
                                changed |= ui
                                    .selectable_value(
                                        &mut self.form_filters.rating_source,
                                        RatingSource::Osu,
                                        "osu!",
                                    )
                                    .changed();
                            });
                    });
                });

            Frame::default()
                .corner_radius(CornerRadius::same(5))
                .inner_margin(Margin::symmetric(6, 3))
                .fill(dropdown_bg)
                .stroke(Stroke::new(1.0, colors.panel_border))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Metric:").size(10.0).color(colors.text_muted));

                        ComboBox::from_id_salt("metric_combo")
                            .selected_text(
                                RichText::new(self.form_filters.rating_metric.display_name())
                                    .size(10.0)
                                    .color(colors.text_primary),
                            )
                            .show_ui(ui, |ui| {
                                for metric in [
                                    RatingMetric::Overall,
                                    RatingMetric::Stream,
                                    RatingMetric::Jumpstream,
                                    RatingMetric::Handstream,
                                    RatingMetric::Stamina,
                                    RatingMetric::Jackspeed,
                                    RatingMetric::Chordjack,
                                    RatingMetric::Technical,
                                ] {
                                    changed |= ui
                                        .selectable_value(
                                            &mut self.form_filters.rating_metric,
                                            metric.clone(),
                                            metric.display_name(),
                                        )
                                        .changed();
                                }
                            });
                    });
                });
        });

        changed
    }

    fn render_collapsible_filters(
        &mut self,
        ui: &mut Ui,
        colors: &SearchPanelColors,
        has_bg: bool,
    ) -> bool {
        let mut changed = false;

        let header_bg = if has_bg {
            Color32::from_rgba_unmultiplied(0, 0, 0, 80)
        } else {
            colors.panel_secondary
        };

        // Count active filters
        let active_count = [
            self.form_filters.min_rating.is_some(),
            self.form_filters.max_rating.is_some(),
            self.form_filters.min_duration_seconds.is_some(),
            self.form_filters.max_duration_seconds.is_some(),
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        // Collapsible header
        let header_response = Frame::default()
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::symmetric(8, 5))
            .fill(header_bg)
            .stroke(Stroke::new(1.0, colors.panel_border))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let arrow = if self.filters_expanded { "▼" } else { "▶" };
                    ui.label(RichText::new(arrow).size(9.0).color(colors.accent));
                    ui.add_space(3.0);
                    ui.label(
                        RichText::new("Filters")
                            .size(11.0)
                            .color(colors.text_secondary),
                    );

                    // Show active count badge
                    if active_count > 0 {
                        ui.add_space(4.0);
                        Frame::default()
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(Margin::symmetric(5, 1))
                            .fill(colors.accent)
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(format!("{}", active_count))
                                        .size(9.0)
                                        .strong()
                                        .color(colors.panel_bg),
                                );
                            });
                    }
                });
            })
            .response;

        if header_response.interact(egui::Sense::click()).clicked() {
            self.filters_expanded = !self.filters_expanded;
        }

        // Content (only if expanded)
        if self.filters_expanded {
            ui.add_space(4.0);

            // Rating section
            ui.label(RichText::new("Rating").size(10.0).color(colors.text_muted));
            ui.add_space(2.0);

            changed |= Self::toggle_slider_static(
                ui,
                "Min",
                &mut self.form_filters.min_rating,
                15.0,
                0.0..=50.0,
                "",
                colors,
                has_bg,
            );
            changed |= Self::toggle_slider_static(
                ui,
                "Max",
                &mut self.form_filters.max_rating,
                30.0,
                0.0..=50.0,
                "",
                colors,
                has_bg,
            );

            ui.add_space(6.0);

            // Duration section
            ui.label(
                RichText::new("Duration")
                    .size(10.0)
                    .color(colors.text_muted),
            );
            ui.add_space(2.0);

            changed |= Self::toggle_slider_static(
                ui,
                "Min",
                &mut self.form_filters.min_duration_seconds,
                60.0,
                0.0..=600.0,
                "s",
                colors,
                has_bg,
            );
            changed |= Self::toggle_slider_static(
                ui,
                "Max",
                &mut self.form_filters.max_duration_seconds,
                240.0,
                0.0..=600.0,
                "s",
                colors,
                has_bg,
            );
        }

        changed
    }

    fn toggle_slider_static(
        ui: &mut Ui,
        label: &str,
        target: &mut Option<f64>,
        default_value: f64,
        range: std::ops::RangeInclusive<f32>,
        suffix: &str,
        colors: &SearchPanelColors,
        has_bg: bool,
    ) -> bool {
        let mut changed = false;
        let mut active = target.is_some();

        let checkbox_bg = if has_bg {
            Color32::from_rgba_unmultiplied(0, 0, 0, 120)
        } else {
            colors.panel_secondary
        };

        ui.horizontal(|ui| {
            // Custom styled checkbox
            let checkbox_response = Frame::default()
                .corner_radius(CornerRadius::same(3))
                .inner_margin(Margin::same(1))
                .fill(if active { colors.accent } else { checkbox_bg })
                .stroke(Stroke::new(
                    1.0,
                    if active {
                        colors.accent
                    } else {
                        colors.panel_border
                    },
                ))
                .show(ui, |ui| {
                    ui.add_space(10.0);
                })
                .response;

            if checkbox_response.interact(egui::Sense::click()).clicked() {
                active = !active;
                if active {
                    *target = Some(default_value);
                } else {
                    *target = None;
                }
                changed = true;
            }

            ui.label(RichText::new(label).size(10.0).color(if active {
                colors.text_primary
            } else {
                colors.text_muted
            }));

            if active {
                let mut value = target.unwrap_or(default_value) as f32;
                let slider = Slider::new(&mut value, range)
                    .suffix(suffix)
                    .show_value(true);

                if ui.add(slider).changed() {
                    *target = Some(value as f64);
                    changed = true;
                }
            }
        });

        changed
    }
}

