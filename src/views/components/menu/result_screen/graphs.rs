use crate::models::engine::US_PER_MS;
use crate::models::engine::hit_window::HitWindow;
use crate::models::replay::ReplayResult;
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, Ui, Vec2};

/// Helper to convert µs to ms for display
#[inline]
fn us_to_ms(us: i64) -> f64 {
    us as f64 / US_PER_MS as f64
}

pub fn render_graphs(ui: &mut Ui, replay_result: &ReplayResult, hit_window: &HitWindow) {
    ui.vertical(|ui| {
        ui.label(egui::RichText::new("Hit Deviation Distribution").strong());
        egui::Frame::canvas(ui.style())
            .fill(Color32::from_black_alpha(50))
            .stroke(Stroke::new(1.0, Color32::from_gray(60)))
            .show(ui, |ui| {
                let (response, painter) = ui
                    .allocate_painter(Vec2::new(ui.available_width(), 150.0), egui::Sense::hover());
                render_hit_histogram(&painter, &response.rect, replay_result, hit_window);
            });
        ui.add_space(20.0);
        ui.label(egui::RichText::new("Hit Timeline").strong());
        egui::Frame::canvas(ui.style())
            .fill(Color32::from_black_alpha(50))
            .stroke(Stroke::new(1.0, Color32::from_gray(60)))
            .show(ui, |ui| {
                let (response, painter) = ui
                    .allocate_painter(Vec2::new(ui.available_width(), 200.0), egui::Sense::hover());
                render_timeline_graph(&painter, &response.rect, replay_result, hit_window);
            });
    });
}

fn render_hit_histogram(
    painter: &Painter,
    rect: &Rect,
    replay_result: &ReplayResult,
    hit_window: &HitWindow,
) {
    let center_x = rect.center().x;
    let bottom_y = rect.bottom() - 20.0;
    let top_y = rect.top() + 10.0;
    let graph_height = bottom_y - top_y;
    let width = rect.width();

    // Ligne centrale (0ms)
    painter.line_segment(
        [Pos2::new(center_x, top_y), Pos2::new(center_x, bottom_y)],
        Stroke::new(1.0, Color32::WHITE.linear_multiply(0.3)),
    );

    // Collecter les timings depuis hit_timings (convert µs to ms)
    let hits: Vec<f64> = replay_result
        .hit_timings
        .iter()
        .map(|h| h.timing_ms())
        .collect();

    if hits.is_empty() {
        return;
    }

    // Convert miss_us to ms for display range
    let miss_ms = us_to_ms(hit_window.miss_us);
    let range_ms = (miss_ms as f32).max(50.0);
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
        let color = get_color_for_timing_ms(bucket_time as f64, hit_window);
        painter.rect_filled(bar_rect, 1.0, color.linear_multiply(0.8));
    }

    // Labels
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

fn render_timeline_graph(
    painter: &Painter,
    rect: &Rect,
    replay_result: &ReplayResult,
    hit_window: &HitWindow,
) {
    if replay_result.hit_timings.is_empty() {
        return;
    }

    let center_y = rect.center().y;
    let width = rect.width() - 40.0;
    let graph_rect = Rect::from_min_max(rect.min, Pos2::new(rect.left() + width, rect.bottom()));

    // Ligne centrale (0ms)
    painter.line_segment(
        [
            Pos2::new(graph_rect.left(), center_y),
            Pos2::new(graph_rect.right(), center_y),
        ],
        Stroke::new(1.0, Color32::WHITE.linear_multiply(0.3)),
    );

    // Convert miss_us to ms for scale
    let miss_ms = us_to_ms(hit_window.miss_us) as f32;
    let scale_y = (graph_rect.height() / 2.0) / miss_ms * 0.9;
    let font_id = FontId::monospace(10.0);

    // Helper to draw guides (takes ms value for display)
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
            format!("+{:.0}ms", ms),
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
            format!("-{:.0}ms", ms),
            font_id.clone(),
            color,
        );
    };

    // Convert µs thresholds to ms for display
    draw_guide(
        us_to_ms(hit_window.marv_us),
        Color32::from_rgb(0, 255, 255),
        "Marv",
    );
    draw_guide(us_to_ms(hit_window.perfect_us), Color32::YELLOW, "Perf");
    draw_guide(us_to_ms(hit_window.great_us), Color32::GREEN, "Great");

    painter.text(
        Pos2::new(graph_rect.right() + 5.0, center_y),
        Align2::LEFT_CENTER,
        "0ms",
        font_id.clone(),
        Color32::WHITE,
    );

    // Use note_time_us for X axis (more precise than note_index)
    let min_time = replay_result
        .hit_timings
        .first()
        .map(|h| us_to_ms(h.note_time_us))
        .unwrap_or(0.0);
    let max_time = replay_result
        .hit_timings
        .last()
        .map(|h| us_to_ms(h.note_time_us))
        .unwrap_or(1.0);
    let time_range = (max_time - min_time).max(1.0);

    for hit in &replay_result.hit_timings {
        let note_time_ms = us_to_ms(hit.note_time_us);
        let x_ratio = (note_time_ms - min_time) as f32 / time_range as f32;
        let x = graph_rect.left() + x_ratio * width;

        let timing_ms = hit.timing_ms();
        // Invert timing data so Early (if positive) plots to Negative region (Bottom/Left)
        let display_timing = -timing_ms;

        let y_offset = display_timing as f32 * scale_y;
        let y = center_y - y_offset;

        let color = get_color_for_timing_ms(timing_ms, hit_window);
        painter.circle_filled(Pos2::new(x, y), 2.0, color);
    }
}

/// Get color based on timing offset (in ms) compared to hit window thresholds (in µs)
fn get_color_for_timing_ms(timing_ms: f64, hit_window: &HitWindow) -> Color32 {
    // Convert timing from ms to µs for comparison with window thresholds
    let timing_us = (timing_ms.abs() * US_PER_MS as f64) as i64;

    if timing_us <= hit_window.marv_us {
        Color32::from_rgb(0, 255, 255)
    } else if timing_us <= hit_window.perfect_us {
        Color32::YELLOW
    } else if timing_us <= hit_window.great_us {
        Color32::GREEN
    } else if timing_us <= hit_window.good_us {
        Color32::from_rgb(0, 0, 128)
    } else if timing_us <= hit_window.bad_us {
        Color32::from_rgb(255, 105, 180)
    } else {
        Color32::RED
    }
}
