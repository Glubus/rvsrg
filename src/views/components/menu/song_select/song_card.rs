//! Renders a single beatmapset card inside the song list.

use egui::{
    Color32, Label, Margin, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, TextureId, UiBuilder,
    Vec2,
};

use crate::database::models::{BeatmapWithRatings, Beatmapset};
use crate::models::skin::menus::song_select::RatingColorsConfig;
use crate::views::components::menu::song_select::difficulty_utils::{
    get_difficulty_color, get_difficulty_range,
};

pub struct SongCard;

impl SongCard {
    /// Renders a beatmapset row with difficulty range display.
    /// Returns the egui response for interaction.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        ui: &mut egui::Ui,
        beatmapset: &Beatmapset,
        beatmaps: &[BeatmapWithRatings],
        is_selected: bool,
        texture_normal: Option<TextureId>,
        texture_selected: Option<TextureId>,
        selected_color: Color32,
        rating_colors: Option<&RatingColorsConfig>,
        active_calculator: &str,
    ) -> egui::Response {
        let card_height = 80.0;
        let width = ui.available_width();
        let size = Vec2::new(width, card_height);

        let (rect, response) = ui.allocate_exact_size(size, Sense::click());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Determine which texture to use
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

                // Draw the base image
                painter.image(
                    tex_id,
                    rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    base_tint,
                );

                // Alpha-aware selection overlay:
                // We use the image tint directly instead of a solid overlay.
                // When selected and no separate selected texture, apply a colored tint
                // that respects the alpha channel of the original image.
                if is_selected && texture_selected.is_none() && texture_normal.is_some() {
                    // Draw the image again with a blended color tint
                    // This respects the alpha channel of the original image
                    let overlay_tint = Color32::from_rgba_unmultiplied(
                        (selected_color.r() as u16 * 180 / 255) as u8 + 75,
                        (selected_color.g() as u16 * 180 / 255) as u8 + 75,
                        (selected_color.b() as u16 * 180 / 255) as u8 + 75,
                        180,
                    );
                    painter.image(
                        tex_id,
                        rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        overlay_tint,
                    );
                }
            } else {
                // Fallback: solid color background when no texture
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
            bottom: 4,
        };

        let content_rect = Rect::from_min_max(
            rect.min + Vec2::new(card_margin.left as f32, card_margin.top as f32),
            rect.max - Vec2::new(card_margin.right as f32, card_margin.bottom as f32),
        );

        let mut content_ui =
            ui.new_child(UiBuilder::new().max_rect(content_rect).layout(*ui.layout()));

        content_ui.vertical(|ui| {
            // Title
            if let Some(title) = &beatmapset.title {
                ui.add(
                    Label::new(
                        RichText::new(title)
                            .size(22.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .selectable(false),
                );
            }

            // Artist
            let artist = beatmapset.artist.as_deref().unwrap_or("");
            ui.add(
                Label::new(RichText::new(artist).size(14.0).color(Color32::LIGHT_GRAY))
                    .selectable(false),
            );

            // Difficulty range display
            if let Some(colors) = rating_colors {
                if let Some((min_rating, max_rating)) =
                    get_difficulty_range(beatmaps, active_calculator)
                {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;

                        // Get colors for min and max
                        let min_color = get_difficulty_color(min_rating, colors);
                        let max_color = get_difficulty_color(max_rating, colors);

                        // Display range with colors
                        if (max_rating - min_rating).abs() < 0.5 {
                            // Single difficulty - show just one value
                            ui.add(
                                Label::new(
                                    RichText::new(format!("★ {:.1}", min_rating))
                                        .size(12.0)
                                        .strong()
                                        .color(min_color),
                                )
                                .selectable(false),
                            );
                        } else {
                            // Range - show min → max
                            ui.add(
                                Label::new(
                                    RichText::new(format!("★ {:.1}", min_rating))
                                        .size(12.0)
                                        .strong()
                                        .color(min_color),
                                )
                                .selectable(false),
                            );
                            ui.add(
                                Label::new(RichText::new(" → ").size(11.0).color(Color32::GRAY))
                                    .selectable(false),
                            );
                            ui.add(
                                Label::new(
                                    RichText::new(format!("{:.1}", max_rating))
                                        .size(12.0)
                                        .strong()
                                        .color(max_color),
                                )
                                .selectable(false),
                            );
                        }
                    });
                } else {
                    // No ratings found - show placeholder
                    ui.add(
                        Label::new(RichText::new("★ --").size(12.0).color(Color32::DARK_GRAY))
                            .selectable(false),
                    );
                }
            } else {
                // No rating colors config - shouldn't happen but fallback
                ui.add(
                    Label::new(RichText::new("★ ?").size(12.0).color(Color32::GRAY))
                        .selectable(false),
                );
            }
        });

        response
    }
}
