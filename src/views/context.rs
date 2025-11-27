//! Shared data passed into gameplay rendering components.

use crate::models::engine::PixelSystem;
use wgpu::{BindGroup, Buffer, Device, Queue, RenderPipeline, TextureView};
use wgpu_text::TextBrush;

/// Bundles resources required to draw a single gameplay frame.
pub struct GameplayRenderContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub text_brush: &'a mut TextBrush,

    // Pipelines & Buffers
    pub render_pipeline: &'a RenderPipeline,
    pub instance_buffer: &'a Buffer,
    pub receptor_buffer: &'a Buffer,

    // Bind Groups (Textures)
    pub note_bind_groups: &'a [BindGroup],
    pub receptor_bind_groups: &'a [BindGroup],
    pub receptor_pressed_bind_groups: &'a [BindGroup],

    pub view: &'a TextureView,
    pub pixel_system: &'a PixelSystem,

    pub screen_width: f32,
    pub screen_height: f32,
    pub fps: f64,
    pub master_volume: f32,
}
