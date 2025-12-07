use crate::models::stats::HitStats;
use egui::{Color32, CornerRadius, RichText, Sense, Stroke, Vec2};

pub struct LeaderboardCard;

impl LeaderboardCard {
    pub fn render(
        ui: &mut egui::Ui,
        rank: usize,
        accuracy: f64,
        rate: f64,
        timestamp: i64,
        max_combo: i32,
        hit_stats: &HitStats,
        is_practice: bool,
    ) -> egui::Response {
        let available_width = ui.available_width();

        // Couleurs selon le rang
        let (rank_color, bg_color) = match rank {
            0 => (
                Color32::from_rgb(255, 215, 0),
                Color32::from_rgba_unmultiplied(80, 70, 30, 240),
            ), // Gold
            1 => (
                Color32::from_rgb(192, 192, 192),
                Color32::from_rgba_unmultiplied(70, 70, 70, 240),
            ), // Silver
            2 => (
                Color32::from_rgb(205, 127, 50),
                Color32::from_rgba_unmultiplied(70, 50, 30, 240),
            ), // Bronze
            _ => (
                Color32::from_rgb(150, 150, 150),
                Color32::from_rgba_unmultiplied(45, 45, 50, 240),
            ),
        };

        // Practice mode a une teinte différente
        let card_bg = if is_practice {
            Color32::from_rgba_unmultiplied(50, 40, 60, 240) // Purple tint
        } else {
            bg_color
        };

        let frame_response = egui::Frame::default()
            .inner_margin(egui::Margin::symmetric(12, 8))
            .corner_radius(CornerRadius::same(8))
            .fill(card_bg)
            .stroke(Stroke::new(
                1.0,
                Color32::from_rgba_unmultiplied(100, 100, 100, 100),
            ))
            .show(ui, |ui| {
                ui.set_width(available_width - 24.0);

                // === ROW 1: Rank + Accuracy + Practice Badge ===
                ui.horizontal(|ui| {
                    // Rank badge
                    let rank_text = format!("#{}", rank + 1);
                    ui.label(
                        RichText::new(&rank_text)
                            .size(18.0)
                            .strong()
                            .color(rank_color),
                    );

                    // Practice badge
                    if is_practice {
                        ui.add_space(8.0);
                        egui::Frame::default()
                            .inner_margin(egui::Margin::symmetric(6, 2))
                            .corner_radius(CornerRadius::same(4))
                            .fill(Color32::from_rgb(180, 100, 255))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new("PRACTICE")
                                        .size(10.0)
                                        .strong()
                                        .color(Color32::WHITE),
                                );
                            });
                    }

                    // Accuracy (right aligned)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let acc_color = accuracy_color(accuracy);
                        ui.label(
                            RichText::new(format!("{:.2}%", accuracy))
                                .size(20.0)
                                .strong()
                                .color(acc_color),
                        );
                    });
                });

                ui.add_space(4.0);

                // === ROW 2: Rate + Max Combo + Date ===
                ui.horizontal(|ui| {
                    // Rate
                    ui.label(
                        RichText::new(format!("{:.2}x", rate))
                            .size(13.0)
                            .color(Color32::from_rgb(255, 200, 100)),
                    );

                    ui.add_space(12.0);

                    // Max combo
                    ui.label(
                        RichText::new(format!("{}x", max_combo))
                            .size(13.0)
                            .color(Color32::from_rgb(100, 200, 255)),
                    );

                    // Date (right aligned)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let date_str = format_date(timestamp);
                        ui.label(
                            RichText::new(&date_str)
                                .size(11.0)
                                .color(Color32::from_rgb(130, 130, 130)),
                        );
                    });
                });

                ui.add_space(4.0);

                // === ROW 3: Hit Stats (compact) ===
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;

                    // Marv
                    if hit_stats.marv > 0 {
                        render_stat_pill(ui, hit_stats.marv, Color32::from_rgb(0, 255, 255));
                    }
                    // Perfect
                    if hit_stats.perfect > 0 {
                        render_stat_pill(ui, hit_stats.perfect, Color32::from_rgb(255, 255, 100));
                    }
                    // Great
                    if hit_stats.great > 0 {
                        render_stat_pill(ui, hit_stats.great, Color32::from_rgb(100, 255, 100));
                    }
                    // Good
                    if hit_stats.good > 0 {
                        render_stat_pill(ui, hit_stats.good, Color32::from_rgb(100, 180, 255));
                    }
                    // Bad
                    if hit_stats.bad > 0 {
                        render_stat_pill(ui, hit_stats.bad, Color32::from_rgb(200, 150, 100));
                    }
                    // Miss
                    if hit_stats.miss > 0 {
                        render_stat_pill(ui, hit_stats.miss, Color32::from_rgb(255, 80, 80));
                    }
                });
            });

        frame_response.response.interact(Sense::click())
    }
}

fn render_stat_pill(ui: &mut egui::Ui, count: u32, color: Color32) {
    let text = format!("{}", count);
    let width = (text.len() as f32 * 7.0 + 10.0).max(22.0);

    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 16.0), Sense::hover());

    let painter = ui.painter();

    // Background pill
    painter.rect_filled(rect, CornerRadius::same(3), color.gamma_multiply(0.25));

    // Text
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        &text,
        egui::FontId::proportional(11.0),
        color,
    );
}

fn accuracy_color(accuracy: f64) -> Color32 {
    if accuracy >= 99.5 {
        Color32::from_rgb(0, 255, 255) // Cyan - SS
    } else if accuracy >= 98.0 {
        Color32::from_rgb(255, 255, 100) // Yellow - S
    } else if accuracy >= 95.0 {
        Color32::from_rgb(100, 255, 100) // Green - A
    } else if accuracy >= 90.0 {
        Color32::from_rgb(100, 180, 255) // Blue - B
    } else if accuracy >= 80.0 {
        Color32::from_rgb(200, 150, 100) // Orange - C
    } else {
        Color32::from_rgb(255, 100, 100) // Red - D
    }
}

fn format_date(timestamp: i64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let diff = now - timestamp;

    if diff < 60 {
        "Just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else if diff < 604800 {
        format!("{}d ago", diff / 86400)
    } else {
        format!("{}w ago", diff / 604800)
    }
}

