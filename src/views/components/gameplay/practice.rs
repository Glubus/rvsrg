//! Practice Mode UI overlay - progress bar with checkpoints.

use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};

/// Affiche l'overlay du mode Practice avec le graphe de progression et les checkpoints.
pub struct PracticeOverlay;

impl PracticeOverlay {
    /// Rend l'overlay practice mode.
    ///
    /// - `current_time`: temps actuel en ms
    /// - `map_duration`: durée totale de la map en ms
    /// - `checkpoints`: timestamps des checkpoints en ms
    pub fn render(
        ui: &mut Ui,
        current_time: f64,
        map_duration: f64,
        checkpoints: &[f64],
        screen_width: f32,
    ) {
        // Position en haut de l'écran
        let bar_height = 8.0;
        let bar_width = screen_width * 0.6;
        let bar_x = (screen_width - bar_width) / 2.0;
        let bar_y = 50.0;

        let bar_rect =
            Rect::from_min_size(Pos2::new(bar_x, bar_y), Vec2::new(bar_width, bar_height));

        let painter = ui.painter();

        // Background de la barre
        painter.rect_filled(bar_rect, 4.0, Color32::from_rgba_unmultiplied(0, 0, 0, 180));

        // Progression actuelle
        if map_duration > 0.0 {
            let progress = (current_time / map_duration).clamp(0.0, 1.0) as f32;
            let progress_rect =
                Rect::from_min_size(bar_rect.min, Vec2::new(bar_width * progress, bar_height));
            painter.rect_filled(progress_rect, 4.0, Color32::from_rgb(100, 200, 255));
        }

        // Bordure de la barre
        painter.rect_stroke(
            bar_rect,
            4.0,
            Stroke::new(1.5, Color32::from_rgb(150, 150, 150)),
            egui::StrokeKind::Outside,
        );

        // Checkpoints
        for &cp_time in checkpoints {
            if map_duration > 0.0 {
                let cp_progress = (cp_time / map_duration).clamp(0.0, 1.0) as f32;
                let cp_x = bar_x + bar_width * cp_progress;

                // Ligne verticale pour le checkpoint
                painter.line_segment(
                    [
                        Pos2::new(cp_x, bar_y - 4.0),
                        Pos2::new(cp_x, bar_y + bar_height + 4.0),
                    ],
                    Stroke::new(2.5, Color32::from_rgb(255, 200, 50)),
                );

                // Petit triangle/marqueur au-dessus
                let triangle_size = 6.0;
                painter.add(egui::Shape::convex_polygon(
                    vec![
                        Pos2::new(cp_x, bar_y - 4.0),
                        Pos2::new(cp_x - triangle_size / 2.0, bar_y - 4.0 - triangle_size),
                        Pos2::new(cp_x + triangle_size / 2.0, bar_y - 4.0 - triangle_size),
                    ],
                    Color32::from_rgb(255, 200, 50),
                    Stroke::NONE,
                ));
            }
        }

        // Label "PRACTICE MODE"
        let label_pos = Pos2::new(bar_x + bar_width / 2.0, bar_y + bar_height + 12.0);
        painter.text(
            label_pos,
            egui::Align2::CENTER_TOP,
            "PRACTICE MODE",
            egui::FontId::proportional(14.0),
            Color32::from_rgb(255, 200, 50),
        );

        // Instructions (touches)
        let instructions = "[  Checkpoint    ]  Retry    P  Toggle";
        let instr_pos = Pos2::new(bar_x + bar_width / 2.0, bar_y + bar_height + 28.0);
        painter.text(
            instr_pos,
            egui::Align2::CENTER_TOP,
            instructions,
            egui::FontId::proportional(11.0),
            Color32::from_rgba_unmultiplied(200, 200, 200, 200),
        );
    }
}
