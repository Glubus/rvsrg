use crate::render::context::RenderContext;
use crate::render::resources::RenderResources;
use crate::shared::snapshot::{GameplaySnapshot, RenderState};
use crate::views::context::GameplayRenderContext;
use wgpu::{Color, CommandEncoder, LoadOp, Operations, RenderPassDescriptor, TextureView};

pub fn draw_game(
    ctx: &RenderContext,
    res: &mut RenderResources,
    encoder: &mut CommandEncoder,
    view: &TextureView,
    state: &RenderState,
    fps: f64,
) {
    match state {
        RenderState::InGame(snapshot) => {
            encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Gameplay Clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            draw_gameplay(ctx, res, encoder, view, snapshot, fps);
        }
        RenderState::Editor(snapshot) => {
            encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Editor Clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            draw_gameplay(ctx, res, encoder, view, &snapshot.game, fps);
        }
        RenderState::Menu(_) => {
            draw_background(ctx, res, encoder, view);
        }
        RenderState::Result(_) => {
            draw_background(ctx, res, encoder, view);
        }
        RenderState::Empty => {
            encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }
    }
}

fn draw_background(
    _ctx: &RenderContext,
    res: &RenderResources,
    encoder: &mut CommandEncoder,
    view: &TextureView,
) {
    if let Some(bg_group) = &res.background_bind_group {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Background Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&res.background_pipeline);
        pass.set_bind_group(0, bg_group, &[]);
        pass.draw(0..6, 0..1);
    } else {
        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Clear Pass (No BG)"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }
}

pub fn draw_gameplay(
    ctx: &RenderContext,
    res: &mut RenderResources,
    encoder: &mut CommandEncoder,
    view: &TextureView,
    snapshot: &GameplaySnapshot,
    fps: f64,
) {
    let mut view_ctx = GameplayRenderContext {
        device: &ctx.device,
        queue: &ctx.queue,
        text_brush: &mut res.text_brush,
        render_pipeline: &res.render_pipeline,
        progress_pipeline: &res.progress_pipeline,
        instance_buffer: &res.instance_buffer,
        receptor_buffer: &res.receptor_buffer,
        progress_buffer: &res.progress_buffer,
        note_bind_groups: &res.note_bind_groups,
        receptor_bind_groups: &res.receptor_bind_groups,
        receptor_pressed_bind_groups: &res.receptor_pressed_bind_groups,
        mine_bind_group: res.mine_bind_group.as_ref(),
        hold_body_bind_group: res.hold_body_bind_group.as_ref(),
        hold_end_bind_group: res.hold_end_bind_group.as_ref(),
        burst_body_bind_group: res.burst_body_bind_group.as_ref(),
        burst_end_bind_group: res.burst_end_bind_group.as_ref(),
        view,
        pixel_system: &res.pixel_system,
        screen_width: ctx.config.width as f32,
        screen_height: ctx.config.height as f32,
        fps,
        master_volume: 1.0,
    };

    // Get colors from new skin structure
    let judgement = &res.skin.hud.judgement;
    let colors = crate::models::stats::JudgementColors {
        marv: judgement.marv.color,
        perfect: judgement.perfect.color,
        great: judgement.great.color,
        good: judgement.good.color,
        bad: judgement.bad.color,
        miss: judgement.miss.color,
        ghost_tap: judgement.ghost_tap.color,
    };

    // Get labels from new skin structure
    let labels = res.skin.get_judgement_labels();

    let _ = res.gameplay_view.render(
        &mut view_ctx,
        encoder,
        snapshot,
        &mut res.score_display,
        &mut res.accuracy_panel,
        &mut res.judgements_panel,
        &mut res.combo_display,
        &mut res.judgement_flash,
        &mut res.hit_bar,
        &mut res.nps_display,
        &mut res.notes_remaining_display,
        &mut res.scroll_speed_display,
        &mut res.time_left_display,
        &colors,
        &labels,
    );
}
