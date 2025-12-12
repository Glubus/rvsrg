//! Difficulty card widget used inside the song select layout.

use egui::{
    Color32, CornerRadius, Label, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, TextureId,
    UiBuilder, Vec2,
};

use crate::database::models::BeatmapWithRatings;

pub struct DifficultyCard;

impl DifficultyCard {
    /// Renders a difficulty card with a colored indicator bar on the left and rating display.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        ui: &mut egui::Ui,
        beatmap: &BeatmapWithRatings,
        is_selected: bool,
        texture_normal: Option<TextureId>,
        texture_selected: Option<TextureId>,
        selected_color: Color32,
        difficulty_color: Color32,
        difficulty_rating: Option<f64>,
    ) -> egui::Response {
        let card_height = 35.0;
        let full_width = ui.available_width();

        // Align to the right while keeping a left indent to match the song card layout.
        let margin_right = 0.0;
        let margin_left = 40.0;

        let visual_width = (full_width - margin_left - margin_right).max(50.0);

        // Compute the starting position with the left margin.
        let start_pos = ui.cursor().min;
        let card_pos = Pos2::new(start_pos.x + margin_left, start_pos.y);
        let card_rect = Rect::from_min_size(card_pos, Vec2::new(visual_width, card_height));

        // Allocate space using the computed bounds.
        let response = ui.allocate_rect(card_rect, Sense::click());

        // Render graphics
        if ui.is_rect_visible(card_rect) {
            let painter = ui.painter();

            // Difficulty color bar on the left
            let bar_width = 4.0;
            let bar_rect = Rect::from_min_size(card_rect.min, Vec2::new(bar_width, card_height));
            painter.rect_filled(
                bar_rect,
                CornerRadius {
                    nw: 4,
                    sw: 4,
                    ne: 0,
                    se: 0,
                },
                difficulty_color,
            );

            // Main card area (after the bar)
            let main_card_rect = Rect::from_min_max(
                Pos2::new(card_rect.min.x + bar_width, card_rect.min.y),
                card_rect.max,
            );

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
                    main_card_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    base_tint,
                );

                // Alpha-aware selection overlay using tint
                if is_selected && texture_selected.is_none() && texture_normal.is_some() {
                    let overlay_tint = Color32::from_rgba_unmultiplied(
                        (selected_color.r() as u16 * 180 / 255) as u8 + 75,
                        (selected_color.g() as u16 * 180 / 255) as u8 + 75,
                        (selected_color.b() as u16 * 180 / 255) as u8 + 75,
                        160,
                    );
                    painter.image(
                        tex_id,
                        main_card_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        overlay_tint,
                    );
                }
            } else {
                // Fallback: solid color background
                let fill_color = Color32::from_rgba_unmultiplied(30, 30, 30, 250);
                painter.rect_filled(main_card_rect, 0.0, fill_color);

                let stroke_color = if is_selected {
                    selected_color
                } else {
                    Color32::from_rgba_unmultiplied(60, 60, 60, 255)
                };
                painter.rect_stroke(
                    main_card_rect,
                    0.0,
                    Stroke::new(1.0, stroke_color),
                    StrokeKind::Inside,
                );
            }
        }

        // Text content inside the card.
        let text_rect = Rect::from_min_max(
            Pos2::new(card_rect.min.x + 12.0, card_rect.min.y),
            Pos2::new(card_rect.max.x - 8.0, card_rect.max.y),
        );
        let mut content_ui =
            ui.new_child(UiBuilder::new().max_rect(text_rect).layout(*ui.layout()));

        content_ui.horizontal(|ui| {
            ui.centered_and_justified(|ui| {
                ui.horizontal(|ui| {
                    // Rating first on the left with color (after the bar)
                    if let Some(rating) = difficulty_rating {
                        ui.add(
                            Label::new(
                                RichText::new(format!("{:.1}", rating))
                                    .size(13.0)
                                    .strong()
                                    .color(difficulty_color),
                            )
                            .selectable(false),
                        );
                        ui.add_space(8.0);
                    }

                    // Then difficulty name
                    if let Some(diff_name) = &beatmap.beatmap.difficulty_name {
                        ui.add(
                            Label::new(RichText::new(diff_name).size(14.0).color(Color32::WHITE))
                                .selectable(false),
                        );
                    } else {
                        ui.add(
                            Label::new(RichText::new("Unknown").size(14.0).weak())
                                .selectable(false),
                        );
                    }
                });
            });
        });

        // Add spacing between rows of difficulties.
        ui.add_space(0.0);

        response
    }
}
