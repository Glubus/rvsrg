//! Renders a single beatmapset card inside the song list.

use egui::{
    Color32, Label, Margin, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, TextureId, UiBuilder,
    Vec2,
};

use crate::database::models::{BeatmapWithRatings, Beatmapset};

pub struct SongCard;

impl SongCard {
    /// Renders a beatmapset row; returns the egui response for interaction.
    pub fn render(
        ui: &mut egui::Ui,
        beatmapset: &Beatmapset,
        _beatmaps: &[BeatmapWithRatings],
        is_selected: bool,
        texture_normal: Option<TextureId>,
        texture_selected: Option<TextureId>,
        selected_color: Color32,
    ) -> egui::Response {
        // Slightly shorter card to reduce visual bulk.
        let card_height = 80.0;
        let width = ui.available_width();
        let size = Vec2::new(width, card_height);

        let (rect, response) = ui.allocate_exact_size(size, Sense::click());

        if ui.is_rect_visible(rect) {
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
                    rect,
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
                    painter.rect_filled(rect, 0.0, overlay_color);
                }
            } else {
                let fill_color = if is_selected {
                    Color32::from_rgba_unmultiplied(
                        selected_color.r(),
                        selected_color.g(),
                        selected_color.b(),
                        50,
                    )
                } else {
                    Color32::from_rgba_unmultiplied(0, 0, 0, 250)
                };
                painter.rect_filled(rect, 0.0, fill_color);

                let stroke_color = if is_selected {
                    selected_color
                } else {
                    Color32::BLACK
                };
                painter.rect_stroke(
                    rect,
                    0.0,
                    Stroke::new(1.0, stroke_color),
                    StrokeKind::Inside,
                );
            }
        }

        // Narrower margins so the card spans the full row.
        let card_margin = Margin {
            left: 10,
            right: 10,
            top: 8,
            bottom: 0,
        };

        let content_rect = Rect::from_min_max(
            rect.min + Vec2::new(card_margin.left as f32, card_margin.top as f32),
            rect.max - Vec2::new(card_margin.right as f32, card_margin.bottom as f32),
        );

        let mut content_ui =
            ui.new_child(UiBuilder::new().max_rect(content_rect).layout(*ui.layout()));

        content_ui.vertical(|ui| {
            if let Some(title) = &beatmapset.title {
                ui.add(
                    Label::new(
                        RichText::new(title)
                            .size(24.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .selectable(false),
                );
            }

            let artist_creator = if let Some(artist) = &beatmapset.artist {
                format!("{}", artist)
            } else {
                String::new()
            };
            ui.add(
                Label::new(
                    RichText::new(&artist_creator)
                        .size(16.0)
                        .color(Color32::LIGHT_GRAY),
                )
                .selectable(false),
            );
        });

        response
    }
}
