use egui::{Color32, Label, Margin, RichText, Stroke};

use crate::database::models::Beatmap;

pub struct DifficultyCard;

impl DifficultyCard {
    pub fn render(
        ui: &mut egui::Ui,
        beatmap: &Beatmap,
        is_selected: bool,
    ) -> egui::Response {
        let card_margin = Margin {
            left: 20,
            right: 0,
            top: 4,
            bottom: 4,
        };
        let res = egui::Frame::default()
            .inner_margin(card_margin)
            .outer_margin(0.0)
            .fill(Color32::from_rgba_unmultiplied(20, 20, 20, 250))
            .stroke({
                if is_selected {
                    Stroke::new(1.0, Color32::YELLOW)
                } else {
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(40, 40, 40, 255))
                }
            })
            .show(ui, |ui| {
                ui.set_width(ui.available_rect_before_wrap().width());
                ui.set_height(40.0);
                ui.set_max_height(40.0);

                if let Some(diff_name) = &beatmap.difficulty_name {
                    ui.add(Label::new(RichText::new(diff_name).small()).selectable(false));
                } else {
                    ui.add(Label::new(RichText::new("Unknown").small().weak()).selectable(false));
                }
            });

        res.response
    }
}

