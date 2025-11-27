//! Result screen layout mixing stats and performance graphs.

pub mod graphs;
pub mod stats;

use crate::models::engine::hit_window::HitWindow;
use crate::models::menu::GameResultData;
use egui::{Color32, Key, RichText};

pub struct ResultScreen;

impl ResultScreen {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &mut self,
        ctx: &egui::Context,
        data: &GameResultData,
        hit_window: &HitWindow,
    ) -> bool {
        let mut should_close = false;

        // UI-level fallback in case winit focus handling fails.
        if ctx.input(|i| i.key_pressed(Key::Escape) || i.key_pressed(Key::Enter)) {
            should_close = true;
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(Color32::from_black_alpha(200))
                    .inner_margin(40.0),
            )
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("RESULTS")
                            .size(32.0)
                            .strong()
                            .color(Color32::WHITE),
                    );
                    ui.add_space(30.0);
                });

                ui.horizontal(|ui| {
                    let full_width = ui.available_width();
                    let height = ui.available_height();

                    // Compute layout widths.
                    let graphs_width = full_width * 0.40; // Cap graph column at ~40%.
                    let stats_width = full_width * 0.35; // Keep enough room for stats.
                    let spacer = full_width - graphs_width - stats_width;

                    // Left panel (stats).
                    egui::Frame::default()
                        .fill(Color32::TRANSPARENT)
                        .show(ui, |ui| {
                            ui.set_width(stats_width);
                            ui.set_height(height);
                            stats::render_stats(ui, data);
                        });

                    // Spacer between columns.
                    ui.add_space(spacer * 0.5);

                    // Right panel (graphs) constrained to 40%.
                    egui::Frame::default()
                        .fill(Color32::TRANSPARENT)
                        .show(ui, |ui| {
                            ui.set_width(graphs_width);
                            ui.set_height(height);
                            graphs::render_graphs(ui, &data.replay_data, hit_window);
                        });
                });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    let btn = ui.add(
                        egui::Button::new(RichText::new("CONTINUE (Press Enter)").size(16.0))
                            .fill(Color32::from_white_alpha(20))
                            .stroke(egui::Stroke::NONE),
                    );

                    if btn.clicked() {
                        should_close = true;
                    }
                });
            });

        should_close
    }
}
