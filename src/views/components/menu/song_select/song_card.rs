use egui::{Color32, Label, Margin, RichText, Stroke};

use crate::database::models::{Beatmap, Beatmapset};

pub struct SongCard;

impl SongCard {
    pub fn render(
        ui: &mut egui::Ui,
        beatmapset: &Beatmapset,
        beatmaps: &[Beatmap],
        is_selected: bool,
    ) -> egui::Response {
        let card_margin = Margin {
            left: 5,
            right: 0,
            top: 8,
            bottom: 0,
        };
        let res = egui::Frame::default()
            .inner_margin(card_margin)
            .outer_margin(0.0)
            .fill(Color32::from_rgba_unmultiplied(0, 0, 0, 250))
            .stroke({
                if is_selected {
                    Stroke::new(1.0, Color32::RED)
                } else {
                    Stroke::new(1.0, Color32::BLACK)
                }
            })
            .show(ui, |ui| {
                ui.set_width(ui.available_rect_before_wrap().width());
                ui.set_height(64.0);
                ui.set_max_height(64.0);

                if let Some(title) = &beatmapset.title {
                    ui.add(Label::new(RichText::new(title).heading()).selectable(false));
                }
                
                let artist_creator = if let Some(artist) = &beatmapset.artist {
                    format!("{}", artist)
                } else {
                    String::new()
                };
                ui.add(Label::new(&artist_creator).selectable(false));

                if let Some(beatmap) = beatmaps.first() {
                    if let Some(diff_name) = &beatmap.difficulty_name {
                        ui.add(Label::new(diff_name).selectable(false));
                    }
                }
            });

        res.response
    }
}

