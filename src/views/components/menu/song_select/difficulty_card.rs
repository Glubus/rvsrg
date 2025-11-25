use egui::{
    Color32, Label, Margin, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, TextureId, UiBuilder,
    Vec2,
};

use crate::database::models::Beatmap;

pub struct DifficultyCard;

impl DifficultyCard {
    pub fn render(
        ui: &mut egui::Ui,
        beatmap: &Beatmap,
        is_selected: bool,
        texture_normal: Option<TextureId>,
        texture_selected: Option<TextureId>,
        selected_color: Color32,
    ) -> egui::Response {
        let card_height = 35.0;
        let full_width = ui.available_width();

        // --- LOGIQUE D'ALIGNEMENT ---
        // SongCard a : left=10, right=10.
        // Ici on veut aligner à droite (right=10) mais indenter à gauche (left=60).
        let margin_right = 0.0;
        let margin_left = 40.0;

        let visual_width = (full_width - margin_left - margin_right).max(50.0);

        // Calcul de la position (X de départ + marge gauche)
        let start_pos = ui.cursor().min;
        let card_pos = Pos2::new(start_pos.x + margin_left, start_pos.y);
        let card_rect = Rect::from_min_size(card_pos, Vec2::new(visual_width, card_height));

        // Allocation sur la zone précise calculée
        let response = ui.allocate_rect(card_rect, Sense::click());

        // Rendu graphique
        if ui.is_rect_visible(card_rect) {
            let painter = ui.painter();

            let texture_id = if is_selected {
                texture_selected.or(texture_normal)
            } else {
                texture_normal
            };

            if let Some(tex_id) = texture_id {
                let base_tint = if is_selected {
                    Color32::WHITE
                } else {
                    Color32::from_gray(200)
                };
                painter.image(
                    tex_id,
                    card_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    base_tint,
                );

                if is_selected && texture_selected.is_none() && texture_normal.is_some() {
                    let overlay_color = Color32::from_rgba_unmultiplied(
                        selected_color.r(),
                        selected_color.g(),
                        selected_color.b(),
                        100,
                    );
                    painter.rect_filled(card_rect, 0.0, overlay_color);
                }
            } else {
                let fill_color = Color32::from_rgba_unmultiplied(30, 30, 30, 250);
                painter.rect_filled(card_rect, 0.0, fill_color);

                let stroke_color = if is_selected {
                    selected_color
                } else {
                    Color32::from_rgba_unmultiplied(60, 60, 60, 255)
                };
                painter.rect_stroke(
                    card_rect,
                    0.0,
                    Stroke::new(1.0, stroke_color),
                    StrokeKind::Inside,
                );
            }
        }

        // Contenu texte (centré dans la carte elle-même)
        let mut content_ui =
            ui.new_child(UiBuilder::new().max_rect(card_rect).layout(*ui.layout()));

        content_ui.vertical(|ui| {
            ui.centered_and_justified(|ui| {
                if let Some(diff_name) = &beatmap.difficulty_name {
                    ui.add(
                        Label::new(RichText::new(diff_name).size(16.0).color(Color32::WHITE))
                            .selectable(false),
                    );
                } else {
                    ui.add(
                        Label::new(RichText::new("Unknown").size(16.0).weak()).selectable(false),
                    );
                }
            });
        });

        // Espace vertical entre les difficultés
        ui.add_space(0.0);

        response
    }
}
