//! Canvas-based graphs for the result screen (histogram + timeline).

use crate::models::engine::hit_window::HitWindow;
use crate::models::replay::ReplayData;
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, Ui, Vec2};

/// Renders both histogram and timeline graphs for a replay.
pub fn render_graphs(ui: &mut Ui, replay_data: &ReplayData, hit_window: &HitWindow) {
    ui.vertical(|ui| {
        ui.label(egui::RichText::new("Hit Deviation Distribution").strong());
        egui::Frame::canvas(ui.style())
            .fill(Color32::from_black_alpha(50))
            .stroke(Stroke::new(1.0, Color32::from_gray(60)))
            .show(ui, |ui| {
                let (response, painter) = ui
                    .allocate_painter(Vec2::new(ui.available_width(), 150.0), egui::Sense::hover());
                render_hit_histogram(&painter, &response.rect, replay_data, hit_window);
            });
        ui.add_space(20.0);
        ui.label(egui::RichText::new("Hit Timeline").strong());
        egui::Frame::canvas(ui.style())
            .fill(Color32::from_black_alpha(50))
            .stroke(Stroke::new(1.0, Color32::from_gray(60)))
            .show(ui, |ui| {
                let (response, painter) = ui
                    .allocate_painter(Vec2::new(ui.available_width(), 200.0), egui::Sense::hover());
                render_timeline_graph(&painter, &response.rect, replay_data, hit_window);
            });
    });
}

/// Draws the hit deviation histogram centered around zero.
fn render_hit_histogram(
    painter: &Painter,
    rect: &Rect,
    replay_data: &ReplayData,
    hit_window: &HitWindow,
) {
    let center_x = rect.center().x;
    let bottom_y = rect.bottom() - 20.0;
    let top_y = rect.top() + 10.0;
    let graph_height = bottom_y - top_y;
    let width = rect.width();
    painter.line_segment(
        [Pos2::new(center_x, top_y), Pos2::new(center_x, bottom_y)],
        Stroke::new(1.0, Color32::WHITE.linear_multiply(0.3)),
    );
    let hits: Vec<f64> = replay_data.hits.iter().map(|h| h.timing_ms).collect();
    if hits.is_empty() {
        return;
    }
    let range_ms = hit_window.miss_ms.max(50.0) as f32;
    let bucket_count = 60;
    let mut buckets = vec![0; bucket_count];
    let min_val = -range_ms;
    let max_val = range_ms;
    let step = (max_val - min_val) / bucket_count as f32;
    let mut max_bucket_val = 0;
    for &timing in &hits {
        let t = timing as f32;
        if t >= min_val && t < max_val {
            let idx = ((t - min_val) / step).floor() as usize;
            if idx < bucket_count {
                buckets[idx] += 1;
                if buckets[idx] > max_bucket_val {
                    max_bucket_val = buckets[idx];
                }
            }
        }
    }
    if max_bucket_val == 0 {
        return;
    }
    let bar_width = (width / bucket_count as f32) * 0.8;
    for (i, &count) in buckets.iter().enumerate() {
        if count == 0 {
            continue;
        }
        let x_normalized = (i as f32 + 0.5) / bucket_count as f32;
        let center_bar_x = rect.left() + x_normalized * width;
        let bar_height = (count as f32 / max_bucket_val as f32) * graph_height;
        let bar_rect = Rect::from_min_max(
            Pos2::new(center_bar_x - bar_width / 2.0, bottom_y - bar_height),
            Pos2::new(center_bar_x + bar_width / 2.0, bottom_y),
        );
        let bucket_time = min_val + (i as f32 * step);
        let color = get_color_for_timing(bucket_time as f64, hit_window);
        painter.rect_filled(bar_rect, 1.0, color.linear_multiply(0.8));
    }
    let font_id = FontId::monospace(10.0);
    let text_color = Color32::from_gray(180);
    painter.text(
        Pos2::new(center_x, bottom_y + 2.0),
        Align2::CENTER_TOP,
        "0ms",
        font_id.clone(),
        Color32::WHITE,
    );
    painter.text(
        Pos2::new(rect.left(), bottom_y + 2.0),
        Align2::LEFT_TOP,
        format!("-{:.0}ms", range_ms),
        font_id.clone(),
        text_color,
    );
    painter.text(
        Pos2::new(rect.right(), bottom_y + 2.0),
        Align2::RIGHT_TOP,
        format!("+{:.0}ms", range_ms),
        font_id.clone(),
        text_color,
    );
}

/// Draws the per-note timeline of hit offsets.
fn render_timeline_graph(
    painter: &Painter,
    rect: &Rect,
    replay_data: &ReplayData,
    hit_window: &HitWindow,
) {
    if replay_data.hits.is_empty() {
        return;
    }
    let center_y = rect.center().y;
    let width = rect.width() - 40.0;
    let graph_rect = Rect::from_min_max(rect.min, Pos2::new(rect.left() + width, rect.bottom()));
    painter.line_segment(
        [
            Pos2::new(graph_rect.left(), center_y),
            Pos2::new(graph_rect.right(), center_y),
        ],
        Stroke::new(1.0, Color32::WHITE.linear_multiply(0.3)),
    );
    let scale_y = (graph_rect.height() / 2.0) / hit_window.miss_ms as f32 * 0.9;
    let font_id = FontId::monospace(10.0);
    let draw_guide = |ms: f64, color: Color32, _label: &str| {
        let y_offset = ms as f32 * scale_y;
        let stroke = Stroke::new(1.0, color.linear_multiply(0.15));
        let y_top = center_y - y_offset;
        painter.line_segment(
            [
                Pos2::new(graph_rect.left(), y_top),
                Pos2::new(graph_rect.right(), y_top),
            ],
            stroke,
        );
        painter.text(
            Pos2::new(graph_rect.right() + 5.0, y_top),
            Align2::LEFT_CENTER,
            format!("+{}ms", ms),
            font_id.clone(),
            color,
        );
        let y_bottom = center_y + y_offset;
        painter.line_segment(
            [
                Pos2::new(graph_rect.left(), y_bottom),
                Pos2::new(graph_rect.right(), y_bottom),
            ],
            stroke,
        );
        painter.text(
            Pos2::new(graph_rect.right() + 5.0, y_bottom),
            Align2::LEFT_CENTER,
            format!("-{}ms", ms),
            font_id.clone(),
            color,
        );
    };
    draw_guide(hit_window.marv_ms, Color32::from_rgb(0, 255, 255), "Marv");
    draw_guide(hit_window.perfect_ms, Color32::YELLOW, "Perf");
    draw_guide(hit_window.great_ms, Color32::GREEN, "Great");
    painter.text(
        Pos2::new(graph_rect.right() + 5.0, center_y),
        Align2::LEFT_CENTER,
        "0ms",
        font_id.clone(),
        Color32::WHITE,
    );
    let min_idx = replay_data.hits.first().map(|h| h.note_index).unwrap_or(0);
    let max_idx = replay_data.hits.last().map(|h| h.note_index).unwrap_or(1);
    let range_idx = (max_idx - min_idx).max(1) as f32;
    for hit in &replay_data.hits {
        let x_ratio = (hit.note_index - min_idx) as f32 / range_idx;
        let x = graph_rect.left() + x_ratio * width;
        let y_offset = hit.timing_ms as f32 * scale_y;
        let y = center_y - y_offset;
        let color = get_color_for_timing(hit.timing_ms, hit_window);
        painter.circle_filled(Pos2::new(x, y), 2.0, color);
    }
}

fn get_color_for_timing(timing: f64, hit_window: &HitWindow) -> Color32 {
    let abs_timing = timing.abs();
    if abs_timing <= hit_window.marv_ms {
        Color32::from_rgb(0, 255, 255)
    } else if abs_timing <= hit_window.perfect_ms {
        Color32::YELLOW
    } else if abs_timing <= hit_window.great_ms {
        Color32::GREEN
    } else if abs_timing <= hit_window.good_ms {
        Color32::from_rgb(0, 0, 128)
    } else if abs_timing <= hit_window.bad_ms {
        Color32::from_rgb(255, 105, 180)
    } else {
        Color32::RED
    }
}
