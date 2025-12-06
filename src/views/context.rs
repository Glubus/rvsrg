//! Render context structures.

#![allow(dead_code)]

use crate::models::engine::PixelSystem;
use wgpu::{BindGroup, Buffer, Device, Queue, RenderPipeline, TextureView};
use wgpu_text::TextBrush;

/// Contains all resources needed to render a game frame.
pub struct GameplayRenderContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub text_brush: &'a mut TextBrush,

    // Pipelines & Buffers
    pub render_pipeline: &'a RenderPipeline,
    pub progress_pipeline: &'a RenderPipeline, // NEW
    pub instance_buffer: &'a Buffer,
    pub receptor_buffer: &'a Buffer,
    pub progress_buffer: &'a Buffer, // NEW

    // Bind Groups (Textures)
    pub note_bind_groups: &'a [BindGroup],
    pub receptor_bind_groups: &'a [BindGroup],
    pub receptor_pressed_bind_groups: &'a [BindGroup],

    // Special note type bind groups
    pub mine_bind_group: Option<&'a BindGroup>,
    pub hold_body_bind_group: Option<&'a BindGroup>,
    pub hold_end_bind_group: Option<&'a BindGroup>,
    pub burst_body_bind_group: Option<&'a BindGroup>,
    pub burst_end_bind_group: Option<&'a BindGroup>,

    pub view: &'a TextureView,
    pub pixel_system: &'a PixelSystem,

    pub screen_width: f32,
    pub screen_height: f32,
    pub fps: f64,
    pub master_volume: f32,
}
