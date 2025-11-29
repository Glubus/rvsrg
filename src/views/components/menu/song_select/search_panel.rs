//! Filter sidebar used by the song select screen.

use egui::{Color32, ComboBox, Frame, Margin, RichText, Slider, TextEdit, Ui};

use crate::models::menu::MenuState;
use crate::models::search::{MenuSearchFilters, RatingMetric, RatingSource};

/// Message emitted by the search panel when the user applies filters.
pub enum SearchPanelEvent {
    None,
    Apply(MenuSearchFilters),
}

/// Stateful form mirroring `MenuSearchFilters`.
pub struct SearchPanel {
    form_filters: MenuSearchFilters,
}

impl SearchPanel {
    /// Returns a panel with default (inactive) filters.
    pub fn new() -> Self {
        Self {
            form_filters: MenuSearchFilters::default(),
        }
    }

    /// Draws the panel and returns an event when the user applies filters.
    pub fn render(&mut self, ui: &mut Ui, menu_state: &MenuState) -> SearchPanelEvent {
        let mut should_apply = false;

        Frame::default()
            .corner_radius(5.0)
            .outer_margin(Margin::symmetric(0, 4))
            .inner_margin(Margin::same(10))
            .fill(Color32::from_rgba_unmultiplied(20, 20, 20, 220))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Search");
                    if menu_state.search_filters.is_active() {
                        ui.label(
                            RichText::new("Active filters")
                                .size(14.0)
                                .color(Color32::from_rgba_unmultiplied(120, 200, 255, 255)),
                        );
                    }
                });

                ui.add_space(4.0);
                should_apply |= self.render_query(ui);
                ui.add_space(4.0);
                should_apply |= self.render_source_and_metric(ui);
                ui.add_space(4.0);
                should_apply |= self.render_rating_section(ui);
                ui.add_space(4.0);
                should_apply |= self.render_duration_section(ui);
            });

        // Si l'utilisateur a modifié quelque chose, on envoie l'événement
        // Sinon, on synchronise avec menu_state si nécessaire
        if should_apply {
            SearchPanelEvent::Apply(self.form_filters.clone())
        } else {
            // Synchroniser seulement si aucun changement utilisateur n'a été détecté
            // Cela évite d'écraser les modifications en cours
            if self.form_filters != menu_state.search_filters {
                self.form_filters = menu_state.search_filters.clone();
            }
            SearchPanelEvent::None
        }
    }

    fn render_query(&mut self, ui: &mut Ui) -> bool {
        ui.add(
            TextEdit::singleline(&mut self.form_filters.query)
                .hint_text("Artist, title, or difficulty"),
        )
        .changed()
    }

    fn render_source_and_metric(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.horizontal_wrapped(|ui| {
            ComboBox::from_label("Source")
                .selected_text(match self.form_filters.rating_source {
                    RatingSource::Etterna => "Etterna",
                    RatingSource::Osu => "osu!",
                })
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

            ComboBox::from_label("Metric")
                .selected_text(self.form_filters.rating_metric.display_name())
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
        changed
    }

    fn render_rating_section(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                changed |= Self::toggle_slider(
                    ui,
                    "Min rating",
                    &mut self.form_filters.min_rating,
                    15.0,
                    0.0..=50.0,
                    " MSD",
                );
                changed |= Self::toggle_slider(
                    ui,
                    "Max rating",
                    &mut self.form_filters.max_rating,
                    30.0,
                    0.0..=50.0,
                    " MSD",
                );
            });
        });
        changed
    }

    fn render_duration_section(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                changed |= Self::toggle_slider(
                    ui,
                    "Min duration",
                    &mut self.form_filters.min_duration_seconds,
                    60.0,
                    0.0..=600.0,
                    "s",
                );
                changed |= Self::toggle_slider(
                    ui,
                    "Max duration",
                    &mut self.form_filters.max_duration_seconds,
                    240.0,
                    0.0..=600.0,
                    "s",
                );
            });
        });
        changed
    }

    fn toggle_slider(
        ui: &mut Ui,
        label: &str,
        target: &mut Option<f64>,
        default_value: f64,
        range: std::ops::RangeInclusive<f32>,
        suffix: &str,
    ) -> bool {
        let mut changed = false;
        let mut active = target.is_some();

        if ui.checkbox(&mut active, label).changed() {
            if active {
                *target = Some(default_value);
            } else {
                *target = None;
            }
            changed = true;
        }

        if active {
            let mut value = target.unwrap_or(default_value) as f32;
            if ui
                .add(Slider::new(&mut value, range).suffix(suffix))
                .changed()
            {
                *target = Some(value as f64);
                changed = true;
            }
        }

        changed
    }
}
