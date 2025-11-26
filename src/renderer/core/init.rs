use super::Renderer;
use crate::models::engine::{InstanceRaw, NUM_COLUMNS, PixelSystem, PlayfieldConfig};
use crate::models::settings::SettingsState;
use crate::models::skin::Skin;
use crate::renderer::pipeline::{create_bind_group_layout, create_render_pipeline, create_sampler};
use crate::renderer::text::load_text_brush;
use crate::renderer::texture::{create_default_texture, load_texture_from_path};
use crate::shaders::constants::{BACKGROUND_SHADER_SRC, QUAD_SHADER_SRC};
use crate::shared::snapshot::RenderState;
use crate::views::components::{
    AccuracyDisplay, ComboDisplay, HitBarDisplay, JudgementFlash, JudgementPanel, PlayfieldDisplay,
    ScoreDisplay,
};
use crate::views::gameplay::GameplayView;
use egui_wgpu::{Renderer as EguiRenderer, RendererOptions};
use egui_winit::State as EguiState;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use winit::window::Window;

impl Renderer {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let egui_ctx = egui::Context::default();
        let egui_state = EguiState::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let egui_renderer = EguiRenderer::new(&device, surface_format, RendererOptions::default());
        let preferred_present_modes = [
            wgpu::PresentMode::Immediate,
            wgpu::PresentMode::Mailbox,
            wgpu::PresentMode::FifoRelaxed,
            wgpu::PresentMode::Fifo,
        ];
        let present_mode = preferred_present_modes
            .into_iter()
            .find(|mode| surface_caps.present_modes.contains(mode))
            .unwrap_or(surface_caps.present_modes[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let settings = SettingsState::load();
        if let Err(e) = crate::models::skin::init_skin_structure() {
            eprintln!("Warning: Failed to init skin structure: {}", e);
        }
        let mut skin = Skin::load(&settings.current_skin).unwrap_or_else(|_| {
            Skin::load("default").unwrap()
        });
        let num_columns = NUM_COLUMNS;
        skin.load_key_mode(num_columns);

        let load_egui_texture = |path_opt: Option<PathBuf>, name: &str| -> Option<egui::TextureHandle> {
            if let Some(path) = path_opt {
                if path.exists() {
                    if let Ok(image) = image::open(&path) {
                        let size = [image.width() as usize, image.height() as usize];
                        let image_buffer = image.to_rgba8();
                        let pixels = image_buffer.as_flat_samples();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                        return Some(egui_ctx.load_texture(name, color_image, Default::default()));
                    }
                }
            }
            None
        };
        let song_button_texture = load_egui_texture(skin.song_button.clone(), "song_button");
        let song_button_selected_texture = load_egui_texture(skin.song_button_selected.clone(), "song_button_selected");
        let difficulty_button_texture = load_egui_texture(skin.difficulty_button.clone(), "difficulty_button");
        let difficulty_button_selected_texture = load_egui_texture(skin.difficulty_button_selected.clone(), "difficulty_button_selected");

        let bind_group_layout = create_bind_group_layout(&device);
        let sampler = create_sampler(&device);
        
        let receptor_color = skin.colors.receptor_color;
        let receptor_default_color = [(receptor_color[0]*255.) as u8, (receptor_color[1]*255.) as u8, (receptor_color[2]*255.) as u8, (receptor_color[3]*255.) as u8];
        let mut receptor_bind_groups = Vec::new();
        let mut receptor_pressed_bind_groups = Vec::new();
        for col in 0..num_columns {
             let receptor_texture = if let Some(path) = skin.get_receptor_image(num_columns, col) {
                load_texture_from_path(&device, &queue, &path).map(|(t, _, _)| t).unwrap_or_else(|| create_default_texture(&device, &queue, receptor_default_color, "Def Receptor"))
            } else { create_default_texture(&device, &queue, receptor_default_color, "Def Receptor") };
            let receptor_view = receptor_texture.create_view(&wgpu::TextureViewDescriptor::default());
            receptor_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("Receptor BG"), layout: &bind_group_layout, entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&receptor_view) }, wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) }] }));
            
            let pressed_texture = if let Some(path) = skin.get_receptor_pressed_image(num_columns, col).or_else(|| skin.get_receptor_image(num_columns, col)) {
                 load_texture_from_path(&device, &queue, &path).map(|(t, _, _)| t).unwrap_or_else(|| create_default_texture(&device, &queue, receptor_default_color, "Def Pressed"))
            } else { create_default_texture(&device, &queue, receptor_default_color, "Def Pressed") };
            let pressed_view = pressed_texture.create_view(&wgpu::TextureViewDescriptor::default());
            receptor_pressed_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("Pressed BG"), layout: &bind_group_layout, entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&pressed_view) }, wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) }] }));
        }

        let note_color = skin.colors.note_color;
        let note_default_color = [(note_color[0]*255.) as u8, (note_color[1]*255.) as u8, (note_color[2]*255.) as u8, (note_color[3]*255.) as u8];
        let mut note_bind_groups = Vec::new();
        for col in 0..num_columns {
             let note_texture = if let Some(path) = skin.get_note_image(num_columns, col) {
                load_texture_from_path(&device, &queue, &path).map(|(t, _, _)| t).unwrap_or_else(|| create_default_texture(&device, &queue, note_default_color, "Def Note"))
            } else { create_default_texture(&device, &queue, note_default_color, "Def Note") };
            let note_view = note_texture.create_view(&wgpu::TextureViewDescriptor::default());
            note_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("Note BG"), layout: &bind_group_layout, entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&note_view) }, wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) }] }));
        }

        let render_pipeline = create_render_pipeline(&device, &bind_group_layout, config.format);
        let background_sampler = create_sampler(&device);
        let background_bind_group_layout = create_bind_group_layout(&device);
        let background_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("BG Shader"), source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(BACKGROUND_SHADER_SRC)) });
        let background_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("BG Layout"), bind_group_layouts: &[&background_bind_group_layout], push_constant_ranges: &[] });
        let background_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { cache: None, label: Some("BG Pipeline"), layout: Some(&background_pipeline_layout), vertex: wgpu::VertexState { compilation_options: Default::default(), module: &background_shader, entry_point: Some("vs_main"), buffers: &[] }, fragment: Some(wgpu::FragmentState { compilation_options: Default::default(), module: &background_shader, entry_point: Some("fs_main"), targets: &[Some(wgpu::ColorTargetState { format: config.format, blend: Some(wgpu::BlendState::ALPHA_BLENDING), write_mask: wgpu::ColorWrites::ALL })] }), primitive: wgpu::PrimitiveState::default(), depth_stencil: None, multisample: wgpu::MultisampleState::default(), multiview: None });
        
        let buffer_size = (1000 * std::mem::size_of::<InstanceRaw>()) as u64;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("Inst Buffer"), size: buffer_size, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let receptor_buffer_size = (num_columns * std::mem::size_of::<InstanceRaw>()) as u64;
        let receptor_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("Receptor Buffer"), size: receptor_buffer_size, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        
        let quad_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("Quad Shader"), source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(QUAD_SHADER_SRC)) });
        let quad_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("Quad Layout"), bind_group_layouts: &[], push_constant_ranges: &[] });
        let quad_vertex_layout = wgpu::VertexBufferLayout { array_stride: 32, step_mode: wgpu::VertexStepMode::Instance, attributes: &[wgpu::VertexAttribute { offset: 0, shader_location: 0, format: wgpu::VertexFormat::Float32x2 }, wgpu::VertexAttribute { offset: 8, shader_location: 1, format: wgpu::VertexFormat::Float32x2 }, wgpu::VertexAttribute { offset: 16, shader_location: 2, format: wgpu::VertexFormat::Float32x4 }] };
        let quad_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { cache: None, label: Some("Quad Pipeline"), layout: Some(&quad_pipeline_layout), vertex: wgpu::VertexState { compilation_options: Default::default(), module: &quad_shader, entry_point: Some("vs_main"), buffers: &[quad_vertex_layout] }, fragment: Some(wgpu::FragmentState { compilation_options: Default::default(), module: &quad_shader, entry_point: Some("fs_main"), targets: &[Some(wgpu::ColorTargetState { format: config.format, blend: Some(wgpu::BlendState::ALPHA_BLENDING), write_mask: wgpu::ColorWrites::ALL })] }), primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleStrip, ..Default::default() }, depth_stencil: None, multisample: wgpu::MultisampleState::default(), multiview: None });
        let quad_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("Quad Buffer"), size: (100 * 32) as u64, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });

        let font_path = skin.get_font_path().unwrap_or(PathBuf::from("assets/font.ttf"));
        let text_brush = load_text_brush(&device, size.width, size.height, config.format, &font_path);
        let pixel_system = PixelSystem::new(size.width, size.height);

        let mut playfield_config = PlayfieldConfig::new();
        playfield_config.column_width_pixels = skin.config.column_width_px;
        playfield_config.note_width_pixels = skin.config.note_width_px;
        playfield_config.note_height_pixels = skin.config.note_height_px;
        playfield_config.receptor_width_pixels = skin.config.receptor_width_px;
        playfield_config.receptor_height_pixels = skin.config.receptor_height_px;
        playfield_config.receptor_spacing_pixels = skin.config.receptor_spacing_px;
        playfield_config.x_offset_pixels = 0.0;
        playfield_config.y_offset_pixels = 0.0;

        let judgement_colors = crate::models::stats::JudgementColors {
            marv: skin.colors.marv,
            perfect: skin.colors.perfect,
            great: skin.colors.great,
            good: skin.colors.good,
            bad: skin.colors.bad,
            miss: skin.colors.miss,
            ghost_tap: skin.colors.ghost_tap,
        };

        let hit_bar = HitBarDisplay::new(0.0, 0.0, 100.0, skin.config.hit_bar_height_px);
        let mut score_display = ScoreDisplay::new(0.0, 0.0);
        score_display.set_size(skin.config.score_text_size);
        let mut accuracy_panel = AccuracyDisplay::new(0.0, 0.0);
        accuracy_panel.set_size(skin.config.accuracy_text_size);
        let mut judgements_panel = JudgementPanel::new(0.0, 0.0, judgement_colors);
        judgements_panel.set_size(skin.config.judgement_text_size);
        let mut combo_display = ComboDisplay::new(0.0, 0.0);
        combo_display.set_size(skin.config.combo_text_size);
        let judgement_flash = JudgementFlash::new(0.0, 0.0);

        let playfield_component = PlayfieldDisplay::new(playfield_config);

        let song_select_screen = Some(crate::views::components::menu::song_select::SongSelectScreen::new());

        Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            note_bind_groups,
            receptor_bind_groups,
            receptor_pressed_bind_groups,
            instance_buffer,
            receptor_buffer,
            current_state: RenderState::Empty,
            text_brush,
            last_fps_update: Instant::now(),
            fps: 0.0,
            pixel_system,
            skin,
            gameplay_view: GameplayView::new(playfield_component),
            hit_bar,
            score_display,
            accuracy_panel,
            judgements_panel,
            combo_display,
            judgement_flash,
            background_texture: None,
            background_bind_group: None,
            background_pipeline: Some(background_pipeline),
            background_sampler,
            current_background_path: None,
            quad_pipeline,
            quad_buffer,
            egui_ctx,
            egui_state,
            egui_renderer,
            settings,
            last_key_pressed: None,
            editor_status_text: None,
            editor_values_text: None,
            leaderboard_scores_loaded: false,
            current_leaderboard_hash: None,
            song_select_screen, 
            result_screen: None,
            song_button_texture,
            song_button_selected_texture,
            difficulty_button_texture,
            difficulty_button_selected_texture,
        }
    }
}