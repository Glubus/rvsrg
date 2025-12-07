use super::layout::SkinEditorState;
use crate::models::skin::{Skin, Vec2Conf};
use egui::{
    Align2, Color32, FontId, Id, PointerButton, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2,
};

pub struct GamePreviewViewport;

impl GamePreviewViewport {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut Ui, state: &mut SkinEditorState, skin: &mut Skin) {
        let available_size = ui.available_size();

        let target_aspect = state.target_aspect_ratio();
        let mut width = available_size.x;
        let mut height = width / target_aspect;

        if height > available_size.y {
            height = available_size.y;
            width = height * target_aspect;
        }

        let viewport_size = Vec2::new(width, height);
        let (response, painter) = ui.allocate_painter(available_size, Sense::click_and_drag());
        let viewport_rect = Rect::from_center_size(response.rect.center(), viewport_size);

        if let Some(tex_id) = state.game_texture_id {
            painter.image(
                tex_id,
                viewport_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            painter.rect_filled(viewport_rect, 0.0, Color32::from_rgb(10, 10, 15));
            painter.text(
                viewport_rect.center(),
                Align2::CENTER_CENTER,
                "No Preview",
                FontId::proportional(20.0),
                Color32::from_gray(100),
            );
        }
        painter.rect_stroke(
            viewport_rect,
            0.0,
            Stroke::new(2.0, Color32::from_gray(60)),
            StrokeKind::Inside,
        );

        let scale_x = viewport_rect.width() / state.preview_width as f32;
        let scale_y = viewport_rect.height() / state.preview_height as f32;

        // All selectable elements
        let element_ids = [
            "Notes - Default",
            "Receptors - Default",
            "Score Display",
            "Combo Counter",
            "Accuracy",
            "NPS Display",
            "📊 Hit Bar",
            "📋 Judgement Panel",
            "📝 Notes Remaining",
            "⚡ Scroll Speed",
            "⏱️ Time Left",
            "Flash - All",
            "Flash - Marvelous",
            "Flash - Perfect",
            "Flash - Great",
            "Flash - Good",
            "Flash - Bad",
            "Flash - Miss",
        ];

        // Click-to-Select
        if response.clicked_by(PointerButton::Primary) {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                let mut found = None;
                for id in element_ids.iter() {
                    let rect =
                        self.calculate_element_rect(id, skin, viewport_rect, scale_x, scale_y);
                    if rect.contains(mouse_pos) {
                        found = Some(id.to_string());
                        break;
                    }
                }
                if let Some(id) = found {
                    state.selected_element_id = Some(id);
                }
            }
        }

        // Gizmo for selected element
        if let Some(selected_id) = &state.selected_element_id {
            let gizmo_rect =
                self.calculate_element_rect(selected_id, skin, viewport_rect, scale_x, scale_y);

            if gizmo_rect != Rect::NOTHING {
                painter.rect_stroke(
                    gizmo_rect,
                    0.0,
                    Stroke::new(2.0, Color32::YELLOW),
                    StrokeKind::Inside,
                );
                painter.rect_filled(
                    gizmo_rect,
                    0.0,
                    Color32::from_rgba_unmultiplied(255, 255, 0, 30),
                );

                // Label
                let display_name =
                    selected_id.trim_start_matches(|c: char| !c.is_alphabetic() && c != '-');
                painter.text(
                    gizmo_rect.min - Vec2::new(0.0, 5.0),
                    Align2::LEFT_BOTTOM,
                    display_name,
                    FontId::monospace(11.0),
                    Color32::YELLOW,
                );

                // Drag handling
                let gizmo_id = Id::new("gizmo").with(selected_id);
                let gizmo_response = ui.interact(gizmo_rect, gizmo_id, Sense::drag());

                if gizmo_response.dragged() {
                    let delta = gizmo_response.drag_delta();
                    self.apply_movement(selected_id, skin, delta.x / scale_x, delta.y / scale_y);
                }
            }
        }
    }

    fn calculate_element_rect(&self, id: &str, skin: &Skin, vp: Rect, sx: f32, sy: f32) -> Rect {
        let gameplay = &skin.gameplay;
        let hud = &skin.hud;

        let to_screen = |pos: Vec2Conf, size: Vec2| -> Rect {
            let x = vp.min.x + (pos.x * sx);
            let y = vp.min.y + (pos.y * sy);
            Rect::from_min_size(Pos2::new(x, y), Vec2::new(size.x * sx, size.y * sy))
        };

        let to_screen_centered = |pos: Vec2Conf, size: Vec2| -> Rect {
            let screen_w = size.x * sx;
            let screen_h = size.y * sy;
            let screen_x = vp.min.x + (pos.x * sx) - (screen_w / 2.0);
            let screen_y = vp.min.y + (pos.y * sy) - (screen_h / 2.0);
            Rect::from_min_size(Pos2::new(screen_x, screen_y), Vec2::new(screen_w, screen_h))
        };

        match id {
            "Notes - Default" | "Receptors - Default" => {
                let col_w = gameplay.playfield.column_width;
                let spacing = gameplay.playfield.receptor_spacing;
                let total_w = (4.0 * col_w) + (3.0 * spacing);
                let h = 600.0;

                let center_x = gameplay.playfield.position.x;
                let top_y = gameplay.playfield.position.y;

                let screen_w = total_w * sx;
                let screen_h = h * sy;
                let screen_x = vp.min.x + (center_x * sx) - (screen_w / 2.0);
                let screen_y = vp.min.y + (top_y * sy);

                Rect::from_min_size(Pos2::new(screen_x, screen_y), Vec2::new(screen_w, screen_h))
            }

            "📊 Hit Bar" => {
                let col_w = gameplay.playfield.column_width;
                let total_w = (4.0 * col_w) + (3.0 * gameplay.playfield.receptor_spacing);
                let w = total_w * 0.8;
                let h = hud.hit_bar.scale;

                let center_x = hud.hit_bar.position.x;
                let y = hud.hit_bar.position.y;

                let screen_w = w * sx;
                let screen_h = h * sy;
                let screen_x = vp.min.x + (center_x * sx) - (screen_w / 2.0);
                let screen_y = vp.min.y + (y * sy);

                Rect::from_min_size(Pos2::new(screen_x, screen_y), Vec2::new(screen_w, screen_h))
            }

            // Flash - All uses marv position as reference
            "Flash - All" => to_screen_centered(
                hud.judgement.marv.position,
                Vec2::new(hud.judgement.marv.size.x, hud.judgement.marv.size.y),
            ),

            // Each judgement flash level has its own position
            "Flash - Marvelous" => to_screen_centered(
                hud.judgement.marv.position,
                Vec2::new(hud.judgement.marv.size.x, hud.judgement.marv.size.y),
            ),
            "Flash - Perfect" => to_screen_centered(
                hud.judgement.perfect.position,
                Vec2::new(hud.judgement.perfect.size.x, hud.judgement.perfect.size.y),
            ),
            "Flash - Great" => to_screen_centered(
                hud.judgement.great.position,
                Vec2::new(hud.judgement.great.size.x, hud.judgement.great.size.y),
            ),
            "Flash - Good" => to_screen_centered(
                hud.judgement.good.position,
                Vec2::new(hud.judgement.good.size.x, hud.judgement.good.size.y),
            ),
            "Flash - Bad" => to_screen_centered(
                hud.judgement.bad.position,
                Vec2::new(hud.judgement.bad.size.x, hud.judgement.bad.size.y),
            ),
            "Flash - Miss" => to_screen_centered(
                hud.judgement.miss.position,
                Vec2::new(hud.judgement.miss.size.x, hud.judgement.miss.size.y),
            ),
            "Flash - Ghost Tap" => to_screen_centered(
                hud.judgement.ghost_tap.position,
                Vec2::new(
                    hud.judgement.ghost_tap.size.x,
                    hud.judgement.ghost_tap.size.y,
                ),
            ),

            "Score Display" => to_screen(
                hud.score.position,
                Vec2::new(hud.score.size.x, hud.score.scale * 1.5),
            ),
            "Combo Counter" => to_screen(
                hud.combo.position,
                Vec2::new(hud.combo.size.x, hud.combo.scale * 1.5),
            ),
            "Accuracy" => to_screen(
                hud.accuracy.position,
                Vec2::new(hud.accuracy.size.x, hud.accuracy.scale * 1.5),
            ),
            "NPS Display" => to_screen(
                hud.nps.position,
                Vec2::new(hud.nps.size.x, hud.nps.scale * 1.5),
            ),

            // Judgement Panel - SEPARATE from Flash, uses its own position!
            "📋 Judgement Panel" => to_screen(
                hud.judgement_panel.position,
                Vec2::new(hud.judgement_panel.size.x, hud.judgement_panel.size.y),
            ),

            // NEW: Notes Remaining display
            "📝 Notes Remaining" => to_screen(
                hud.notes_remaining.position,
                Vec2::new(hud.notes_remaining.size.x, hud.notes_remaining.size.y),
            ),

            // NEW: Scroll Speed display
            "⚡ Scroll Speed" => to_screen(
                hud.scroll_speed.position,
                Vec2::new(hud.scroll_speed.size.x, hud.scroll_speed.size.y),
            ),

            // NEW: Time Left display
            "⏱️ Time Left" => to_screen(
                hud.time_left.position,
                Vec2::new(hud.time_left.size.x, hud.time_left.size.y),
            ),

            _ => Rect::NOTHING,
        }
    }

    fn apply_movement(&self, id: &str, skin: &mut Skin, dx: f32, dy: f32) {
        match id {
            "Notes - Default" | "Receptors - Default" => {
                skin.gameplay.playfield.position.x += dx;
                skin.gameplay.playfield.position.y += dy;
            }
            "📊 Hit Bar" => {
                skin.hud.hit_bar.position.x += dx;
                skin.hud.hit_bar.position.y += dy;
            }
            "Score Display" => {
                skin.hud.score.position.x += dx;
                skin.hud.score.position.y += dy;
            }
            "Combo Counter" => {
                skin.hud.combo.position.x += dx;
                skin.hud.combo.position.y += dy;
            }
            "Accuracy" => {
                skin.hud.accuracy.position.x += dx;
                skin.hud.accuracy.position.y += dy;
            }
            "NPS Display" => {
                skin.hud.nps.position.x += dx;
                skin.hud.nps.position.y += dy;
            }
            // Flash - All moves ALL judgement flashes together
            "Flash - All" => {
                skin.hud.judgement.marv.position.x += dx;
                skin.hud.judgement.marv.position.y += dy;
                skin.hud.judgement.perfect.position.x += dx;
                skin.hud.judgement.perfect.position.y += dy;
                skin.hud.judgement.great.position.x += dx;
                skin.hud.judgement.great.position.y += dy;
                skin.hud.judgement.good.position.x += dx;
                skin.hud.judgement.good.position.y += dy;
                skin.hud.judgement.bad.position.x += dx;
                skin.hud.judgement.bad.position.y += dy;
                skin.hud.judgement.miss.position.x += dx;
                skin.hud.judgement.miss.position.y += dy;
                skin.hud.judgement.ghost_tap.position.x += dx;
                skin.hud.judgement.ghost_tap.position.y += dy;
            }
            // Each judgement flash moves independently
            "Flash - Marvelous" => {
                skin.hud.judgement.marv.position.x += dx;
                skin.hud.judgement.marv.position.y += dy;
            }
            "Flash - Perfect" => {
                skin.hud.judgement.perfect.position.x += dx;
                skin.hud.judgement.perfect.position.y += dy;
            }
            "Flash - Great" => {
                skin.hud.judgement.great.position.x += dx;
                skin.hud.judgement.great.position.y += dy;
            }
            "Flash - Good" => {
                skin.hud.judgement.good.position.x += dx;
                skin.hud.judgement.good.position.y += dy;
            }
            "Flash - Bad" => {
                skin.hud.judgement.bad.position.x += dx;
                skin.hud.judgement.bad.position.y += dy;
            }
            "Flash - Miss" => {
                skin.hud.judgement.miss.position.x += dx;
                skin.hud.judgement.miss.position.y += dy;
            }
            "Flash - Ghost Tap" => {
                skin.hud.judgement.ghost_tap.position.x += dx;
                skin.hud.judgement.ghost_tap.position.y += dy;
            }
            // Judgement Panel - SEPARATE from Flash!
            "📋 Judgement Panel" => {
                skin.hud.judgement_panel.position.x += dx;
                skin.hud.judgement_panel.position.y += dy;
            }
            // NEW: Notes Remaining
            "📝 Notes Remaining" => {
                skin.hud.notes_remaining.position.x += dx;
                skin.hud.notes_remaining.position.y += dy;
            }
            // NEW: Scroll Speed
            "⚡ Scroll Speed" => {
                skin.hud.scroll_speed.position.x += dx;
                skin.hud.scroll_speed.position.y += dy;
            }
            // NEW: Time Left
            "⏱️ Time Left" => {
                skin.hud.time_left.position.x += dx;
                skin.hud.time_left.position.y += dy;
            }
            _ => {}
        }
    }
}

