use egui::{Color32, RichText};

use crate::models::stats::HitStats;

pub struct LeaderboardCard;

impl LeaderboardCard {
    pub fn render(
        ui: &mut egui::Ui,
        rank: usize,
        accuracy: f64,
        rate: f64,
        timestamp: i64,
        hit_stats: &HitStats,
    ) {
        egui::Frame::default()
            .inner_margin(5.0)
            .fill(Color32::from_rgba_unmultiplied(50, 50, 50, 255))
            .show(ui, |ui| {
                ui.set_width(ui.available_rect_before_wrap().width());
                
                // Rank and Accuracy
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("#{}", rank + 1)).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("{:.2}%", accuracy)).heading());
                    });
                });

                // Rate
                ui.label(RichText::new(format!("Rate: {:.1}x", rate)).color(Color32::GOLD));

                // Date
                let date_str = format_date(timestamp);
                ui.label(RichText::new(&date_str).small().weak());

                // Hit stats
                ui.horizontal(|ui| {
                    if hit_stats.marv > 0 {
                        ui.label(RichText::new(format!("M:{} ", hit_stats.marv)).color(Color32::from_rgb(0, 255, 255)));
                    }
                    if hit_stats.perfect > 0 {
                        ui.label(RichText::new(format!("P:{} ", hit_stats.perfect)).color(Color32::YELLOW));
                    }
                    if hit_stats.great > 0 {
                        ui.label(RichText::new(format!("G:{} ", hit_stats.great)).color(Color32::GREEN));
                    }
                    if hit_stats.good > 0 {
                        ui.label(RichText::new(format!("Go:{} ", hit_stats.good)).color(Color32::from_rgb(0, 0, 128)));
                    }
                    if hit_stats.bad > 0 {
                        ui.label(RichText::new(format!("B:{} ", hit_stats.bad)).color(Color32::from_rgb(255, 105, 180)));
                    }
                    if hit_stats.miss > 0 {
                        ui.label(RichText::new(format!("Mi:{} ", hit_stats.miss)).color(Color32::RED));
                    }
                });
            });
    }
}

fn format_date(timestamp: i64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let diff = now - timestamp;
    
    if diff < 3600 {
        format!("{} min ago", diff / 60)
    } else if diff < 86400 {
        format!("{} hours ago", diff / 3600)
    } else if diff < 604800 {
        format!("{} days ago", diff / 86400)
    } else {
        format!("{} days ago", diff / 86400)
    }
}

