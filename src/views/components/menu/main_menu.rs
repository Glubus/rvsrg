//! Main menu screen component.
//!
//! Displays the title and main navigation buttons (Play, Quit).

use egui::{Color32, Label, RichText, Vec2};

/// Event returned by the main menu when user wants to navigate
#[derive(Debug, Clone, PartialEq)]
pub enum MainMenuAction {
    /// User clicked Play - go to song select
    Play,
    /// User clicked Quit - exit the game
    Quit,
    /// No action
    None,
}

pub struct MainMenuScreen;

impl MainMenuScreen {
    /// Renders the main menu screen.
    /// Returns the action to take based on user interaction.
    pub fn render(ctx: &egui::Context) -> MainMenuAction {
        let mut action = MainMenuAction::None;

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::from_rgb(15, 15, 20)))
            .show(ctx, |ui| {
                let available_size = ui.available_size();

                // Left panel with title and buttons
                let panel_width = 400.0_f32.min(available_size.x * 0.4);
                let panel_height = available_size.y;

                ui.allocate_ui_with_layout(
                    Vec2::new(panel_width, panel_height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.add_space(60.0);

                        // Title
                        ui.horizontal(|ui| {
                            ui.add_space(40.0);
                            ui.add(
                                Label::new(
                                    RichText::new("PLACEHOLDER IDK")
                                        .size(48.0)
                                        .strong()
                                        .color(Color32::WHITE),
                                )
                                .selectable(false),
                            );
                        });

                        ui.add_space(80.0);

                        // Menu buttons
                        ui.vertical(|ui| {
                            ui.add_space(20.0);

                            // Play button
                            ui.horizontal(|ui| {
                                ui.add_space(40.0);
                                if Self::render_menu_button(ui, "▶  Play", true) {
                                    action = MainMenuAction::Play;
                                }
                            });

                            ui.add_space(16.0);

                            // Quit button
                            ui.horizontal(|ui| {
                                ui.add_space(40.0);
                                if Self::render_menu_button(ui, "✕  Quit", false) {
                                    action = MainMenuAction::Quit;
                                }
                            });
                        });
                    },
                );
            });

        action
    }

    fn render_menu_button(ui: &mut egui::Ui, text: &str, is_primary: bool) -> bool {
        let button_width = 280.0;
        let button_height = 56.0;

        let bg_color = if is_primary {
            Color32::from_rgba_unmultiplied(100, 180, 255, 40)
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 15)
        };

        let hover_color = if is_primary {
            Color32::from_rgba_unmultiplied(100, 180, 255, 80)
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 30)
        };

        let text_color = if is_primary {
            Color32::from_rgb(100, 180, 255)
        } else {
            Color32::from_gray(200)
        };

        let response =
            ui.allocate_response(Vec2::new(button_width, button_height), egui::Sense::click());

        let rect = response.rect;
        let painter = ui.painter();

        // Background
        let fill = if response.hovered() {
            hover_color
        } else {
            bg_color
        };
        painter.rect_filled(rect, 8.0, fill);

        // Border
        let border_color = if response.hovered() {
            text_color
        } else {
            Color32::from_rgba_unmultiplied(text_color.r(), text_color.g(), text_color.b(), 60)
        };
        painter.rect_stroke(
            rect,
            8.0,
            egui::Stroke::new(1.5, border_color),
            egui::StrokeKind::Inside,
        );

        // Text
        let text_pos = rect.left_center() + Vec2::new(24.0, 0.0);
        painter.text(
            text_pos,
            egui::Align2::LEFT_CENTER,
            text,
            egui::FontId::proportional(22.0),
            text_color,
        );

        response.clicked()
    }
}
