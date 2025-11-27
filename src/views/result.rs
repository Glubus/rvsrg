use crate::models::stats::{HitStats, Judgement};
use crate::views::components::common::{QuadInstance, quad_from_rect};
use bytemuck::cast_slice;
use wgpu::{Buffer, Device, Queue, RenderPipeline, SurfaceError, TextureView};
use wgpu_text::{glyph_brush::Section, TextBrush};

pub struct ResultView {
    screen_width: f32,
    screen_height: f32,
}

impl ResultView {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            screen_width,
            screen_height,
        }
    }

    pub fn update_size(&mut self, screen_width: f32, screen_height: f32) {
        self.screen_width = screen_width;
        self.screen_height = screen_height;
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        text_brush: &mut TextBrush,
        view: &TextureView,
        quad_pipeline: &RenderPipeline,
        quad_buffer: &Buffer,
        hit_stats: &HitStats,
        replay_data: &crate::models::replay::ReplayData,
        score: u32,
        accuracy: f64,
        max_combo: u32,
        hit_window: &crate::models::engine::hit_window::HitWindow,
    ) -> Result<(), SurfaceError> {
        // Build quads for the background and panels.
        let mut quads = Vec::new();

        // Panneau gauche (stats)
        let left_panel_width = self.screen_width * 0.35;
        
        // Panneau droit (graph)
        let graph_x = left_panel_width + 20.0;
        let graph_width = self.screen_width - graph_x - 20.0;

        // Background sombre
        quads.push(quad_from_rect(
            0.0,
            0.0,
            self.screen_width,
            self.screen_height,
            [0.1, 0.1, 0.1, 1.0],
            self.screen_width,
            self.screen_height,
        ));

        quads.push(quad_from_rect(
            20.0,
            20.0,
            left_panel_width - 40.0,
            self.screen_height - 40.0,
            [0.15, 0.15, 0.15, 1.0],
            self.screen_width,
            self.screen_height,
        ));

        quads.push(quad_from_rect(
            graph_x,
            20.0,
            graph_width,
            self.screen_height - 40.0,
            [0.12, 0.12, 0.12, 1.0],
            self.screen_width,
            self.screen_height,
        ));

        // Ajouter les quads du graphe
        let graph_quads = self.create_graph_quads(
            graph_x,
            20.0,
            graph_width,
            self.screen_height - 40.0,
            replay_data,
            hit_stats,
            hit_window,
        );
        quads.extend(graph_quads);

        // Rendre les quads
        if !quads.is_empty() {
            queue.write_buffer(quad_buffer, 0, cast_slice(&quads));

            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Result Screen Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                render_pass.set_pipeline(quad_pipeline);
                render_pass.set_vertex_buffer(0, quad_buffer.slice(..));
                render_pass.draw(0..4, 0..quads.len() as u32);
            }
            queue.submit(std::iter::once(encoder.finish()));
        }

        // Generate text sections.
        let mut text_sections = Vec::new();

        // Titre
        text_sections.push(Section {
            screen_position: (self.screen_width / 2.0, 40.0),
            bounds: (self.screen_width, self.screen_height),
            text: vec![
                wgpu_text::glyph_brush::Text::new("Results")
                    .with_scale(48.0)
                    .with_color([1.0, 1.0, 1.0, 1.0]),
            ],
            ..Default::default()
        });

        // Left-hand stats column.
        let left_x = 40.0;
        let mut y = 120.0;

        // Score
        text_sections.push(Section {
            screen_position: (left_x, y),
            bounds: (left_panel_width, self.screen_height),
            text: vec![
                wgpu_text::glyph_brush::Text::new("Score")
                    .with_scale(32.0)
                    .with_color([0.8, 0.8, 0.8, 1.0]),
            ],
            ..Default::default()
        });
        y += 50.0;
        let score_text = format!("{}", score);
        text_sections.push(Section {
            screen_position: (left_x, y),
            bounds: (left_panel_width, self.screen_height),
            text: vec![
                wgpu_text::glyph_brush::Text::new(&score_text)
                    .with_scale(40.0)
                    .with_color([1.0, 1.0, 1.0, 1.0]),
            ],
            ..Default::default()
        });
        y += 80.0;

        // Accuracy
        text_sections.push(Section {
            screen_position: (left_x, y),
            bounds: (left_panel_width, self.screen_height),
            text: vec![
                wgpu_text::glyph_brush::Text::new("Accuracy")
                    .with_scale(32.0)
                    .with_color([0.8, 0.8, 0.8, 1.0]),
            ],
            ..Default::default()
        });
        y += 50.0;
        let accuracy_text = format!("{:.2}%", accuracy);
        text_sections.push(Section {
            screen_position: (left_x, y),
            bounds: (left_panel_width, self.screen_height),
            text: vec![
                wgpu_text::glyph_brush::Text::new(&accuracy_text)
                    .with_scale(40.0)
                    .with_color([1.0, 1.0, 1.0, 1.0]),
            ],
            ..Default::default()
        });
        y += 80.0;

        // Max Combo
        text_sections.push(Section {
            screen_position: (left_x, y),
            bounds: (left_panel_width, self.screen_height),
            text: vec![
                wgpu_text::glyph_brush::Text::new("Max Combo")
                    .with_scale(32.0)
                    .with_color([0.8, 0.8, 0.8, 1.0]),
            ],
            ..Default::default()
        });
        y += 50.0;
        let combo_text = format!("{}", max_combo);
        text_sections.push(Section {
            screen_position: (left_x, y),
            bounds: (left_panel_width, self.screen_height),
            text: vec![
                wgpu_text::glyph_brush::Text::new(&combo_text)
                    .with_scale(40.0)
                    .with_color([1.0, 1.0, 1.0, 1.0]),
            ],
            ..Default::default()
        });
        y += 100.0;

        // Jugements
        text_sections.push(Section {
            screen_position: (left_x, y),
            bounds: (left_panel_width, self.screen_height),
            text: vec![
                wgpu_text::glyph_brush::Text::new("Judgements")
                    .with_scale(32.0)
                    .with_color([0.8, 0.8, 0.8, 1.0]),
            ],
            ..Default::default()
        });
        y += 50.0;

        let judgements = [
            (Judgement::Marv, hit_stats.marv, [0.0, 1.0, 1.0, 1.0]),
            (Judgement::Perfect, hit_stats.perfect, [1.0, 1.0, 0.0, 1.0]),
            (Judgement::Great, hit_stats.great, [0.0, 1.0, 0.0, 1.0]),
            (Judgement::Good, hit_stats.good, [0.0, 0.0, 0.5, 1.0]),
            (Judgement::Bad, hit_stats.bad, [1.0, 0.41, 0.71, 1.0]),
            (Judgement::Miss, hit_stats.miss, [1.0, 0.0, 0.0, 1.0]),
        ];

        // Store labels in a Vec to keep lifetimes simple.
        let mut judgement_labels = Vec::new();
        for (judgement, count, _) in judgements.iter() {
            let label = format!("{:?}: {}", judgement, count);
            judgement_labels.push(label);
        }

        for (i, (_, _, color)) in judgements.iter().enumerate() {
            text_sections.push(Section {
                screen_position: (left_x + 20.0, y),
                bounds: (left_panel_width, self.screen_height),
                text: vec![
                    wgpu_text::glyph_brush::Text::new(&judgement_labels[i])
                        .with_scale(24.0)
                        .with_color(*color),
                ],
                ..Default::default()
            });
            y += 35.0;
        }

        // Graph on the right.
        let graph_area_x = graph_x + 20.0;
        let graph_area_y = 40.0;
        let graph_area_width = graph_width - 40.0;
        let graph_area_height = self.screen_height - 80.0;
        
        self.render_timing_graph(
            &mut text_sections,
            graph_area_x,
            graph_area_y,
            graph_area_width,
            graph_area_height,
            replay_data,
        );
        
        // Configure point labels (building Section here avoids lifetime juggling).
        let hits_text = format!("{} hits recorded", replay_data.hits.len());
        let graph_y = graph_area_y + 50.0;
        let graph_height = graph_area_height - 100.0;
        text_sections.push(Section {
            screen_position: (graph_area_x, graph_y + graph_height + 40.0),
            bounds: (graph_area_width, graph_area_height),
            text: vec![
                wgpu_text::glyph_brush::Text::new(&hits_text)
                    .with_scale(18.0)
                    .with_color([0.6, 0.6, 0.6, 1.0]),
            ],
            ..Default::default()
        });

        // Instructions
        text_sections.push(Section {
            screen_position: (self.screen_width / 2.0, self.screen_height - 40.0),
            bounds: (self.screen_width, self.screen_height),
            text: vec![
                wgpu_text::glyph_brush::Text::new("Press ESC or Enter to return to menu")
                    .with_scale(20.0)
                    .with_color([0.6, 0.6, 0.6, 1.0]),
            ],
            ..Default::default()
        });

        text_brush
            .queue(device, queue, text_sections)
            .map_err(|_| SurfaceError::Lost)?;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Result Screen Text Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            text_brush.draw(&mut render_pass);
        }

        queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }

    fn render_timing_graph(
        &self,
        text_sections: &mut Vec<Section>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        replay_data: &crate::models::replay::ReplayData,
    ) {
        if replay_data.hits.is_empty() {
            text_sections.push(Section {
                screen_position: (x + width / 2.0, y + height / 2.0),
                bounds: (width, height),
                text: vec![
                    wgpu_text::glyph_brush::Text::new("No data available")
                        .with_scale(24.0)
                        .with_color([0.5, 0.5, 0.5, 1.0]),
                ],
                ..Default::default()
            });
            return;
        }

        // Titre du graphique
        text_sections.push(Section {
            screen_position: (x, y),
            bounds: (width, height),
            text: vec![
                wgpu_text::glyph_brush::Text::new("Timing Graph")
                    .with_scale(28.0)
                    .with_color([1.0, 1.0, 1.0, 1.0]),
            ],
            ..Default::default()
        });

        let graph_y = y + 50.0;
        let graph_height = height - 100.0;
        let graph_padding = 40.0;

        // Axis legend.
        text_sections.push(Section {
            screen_position: (x, graph_y + graph_height + 10.0),
            bounds: (width, height),
            text: vec![
                wgpu_text::glyph_brush::Text::new("Time →")
                    .with_scale(18.0)
                    .with_color([0.7, 0.7, 0.7, 1.0]),
            ],
            ..Default::default()
        });

        text_sections.push(Section {
            screen_position: (x - 30.0, graph_y + graph_height / 2.0),
            bounds: (width, height),
            text: vec![
                wgpu_text::glyph_brush::Text::new("↑\nTiming\n(ms)")
                    .with_scale(16.0)
                    .with_color([0.7, 0.7, 0.7, 1.0]),
            ],
            ..Default::default()
        });
    }

    /// Creates quads for the timing graph background.
    pub fn create_graph_quads(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        replay_data: &crate::models::replay::ReplayData,
        _hit_stats: &HitStats,
        hit_window: &crate::models::engine::hit_window::HitWindow,
    ) -> Vec<QuadInstance> {
        let mut quads = Vec::new();

        if replay_data.hits.is_empty() {
            return quads;
        }

        let graph_padding = 40.0;
        let graph_x = x + graph_padding;
        let graph_y = y + 50.0;
        let graph_width = width - graph_padding * 2.0;
        let graph_height = height - 100.0;

        if replay_data.hits.is_empty() {
            return quads;
        }

        // Deduplicate hits per note_index (keep the earliest) to avoid duplicate points.
        use std::collections::HashMap;
        let mut unique_hits: HashMap<usize, &crate::models::replay::ReplayHit> = HashMap::new();
        for hit in &replay_data.hits {
            unique_hits.entry(hit.note_index).or_insert(hit);
        }
        let deduplicated_hits: Vec<_> = unique_hits.values().cloned().collect();

        // Find min/max indices to normalize the X axis using note_index.
        let (min_time, max_time) = deduplicated_hits.iter().fold(
            (deduplicated_hits[0].note_index, deduplicated_hits[0].note_index),
            |(min, max), hit| {
                (min.min(hit.note_index), max.max(hit.note_index))
            },
        );

        let timing_range = 200.0; // Covers -100ms to +100ms.
        let timing_min = -timing_range;
        let timing_max = timing_range;

        // Draw the center line representing 0 ms offset.
        let center_y = graph_y + graph_height / 2.0;
        quads.push(quad_from_rect(
            graph_x,
            center_y - 1.0,
            graph_width,
            2.0,
            [0.3, 0.3, 0.3, 1.0],
            self.screen_width,
            self.screen_height,
        ));

        // Colors for each judgement, matching the HitWindow thresholds.
        let get_color_for_timing = |timing: f64| -> [f32; 4] {
            let abs_timing = timing.abs();
            if abs_timing <= hit_window.marv_ms {
                [0.0, 1.0, 1.0, 1.0] // Marv - cyan
            } else if abs_timing <= hit_window.perfect_ms {
                [1.0, 1.0, 0.0, 1.0] // Perfect - yellow
            } else if abs_timing <= hit_window.great_ms {
                [0.0, 1.0, 0.0, 1.0] // Great - green
            } else if abs_timing <= hit_window.good_ms {
                [0.0, 0.0, 0.5, 1.0] // Good - dark blue
            } else if abs_timing <= hit_window.bad_ms {
                [1.0, 0.41, 0.71, 1.0] // Bad - pink
            } else {
                [1.0, 0.0, 0.0, 1.0] // Miss - red
            }
        };

        // Render each point using the deduplicated hits.
        let point_size = 3.0;
        
        for hit in &deduplicated_hits {
            // Normalize X position (time axis).
            let time_ratio = if max_time > min_time {
                (hit.note_index - min_time) as f32 / (max_time - min_time) as f32
            } else {
                0.5
            };
            let point_x = graph_x + time_ratio * graph_width;

            // Normalize Y position (timing offset).
            let timing_ratio = ((hit.timing_ms - timing_min) / (timing_max - timing_min)) as f32;
            let timing_ratio = timing_ratio.clamp(0.0, 1.0);
            let point_y = graph_y + (1.0 - timing_ratio) * graph_height;

            let color = get_color_for_timing(hit.timing_ms);

            quads.push(quad_from_rect(
                point_x - point_size / 2.0,
                point_y - point_size / 2.0,
                point_size,
                point_size,
                color,
                self.screen_width,
                self.screen_height,
            ));
        }

        quads
    }
}

