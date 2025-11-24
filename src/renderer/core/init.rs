use super::Renderer;
use crate::models::engine::{GameEngine, InstanceRaw, NUM_COLUMNS, PixelSystem, PlayfieldConfig};
use crate::models::menu::MenuState;
use crate::models::skin::Skin;
use crate::models::settings::GameSettings;
use crate::shaders::constants::{BACKGROUND_SHADER_SRC, QUAD_SHADER_SRC};
use crate::views::components::{
    AccuracyDisplay, ComboDisplay, HitBarDisplay, JudgementFlash, JudgementPanel, PlayfieldDisplay,
    ScoreDisplay,
};
use crate::views::gameplay::GameplayView;
use crate::views::result::ResultView;
// Note: Ajuste les chemins 'super::super' selon l'emplacement exact de tes fichiers
use crate::renderer::pipeline::{create_bind_group_layout, create_render_pipeline, create_sampler};
use crate::renderer::text::load_text_brush;
use crate::renderer::texture::{create_default_texture, load_texture_from_path};

use egui_wgpu::{Renderer as EguiRenderer, RendererOptions};
use egui_winit::State as EguiState;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use winit::window::Window;
impl Renderer {
    pub async fn new(window: Arc<Window>, menu_state: Arc<Mutex<MenuState>>) -> Self {
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

        // Initialiser la structure des skins
        if let Err(e) = crate::models::skin::init_skin_structure() {
            eprintln!("Warning: Failed to initialize skin structure: {}", e);
        }

        let num_columns = NUM_COLUMNS;
        let skin_temp = Skin::load_default(num_columns).unwrap_or_else(|e| {
            eprintln!(
                "Warning: Failed to load default skin: {}. Using fallback.",
                e
            );
            Self::create_fallback_skin()
        });

        let bind_group_layout = create_bind_group_layout(&device);
        let sampler = create_sampler(&device);

        // --- Chargement des Textures (Receptors & Notes) ---
        // (Tu peux extraire ces boucles dans des fonctions privées si tu veux encore plus découper)
        let receptor_color = skin_temp.get_receptor_color();
        let receptor_default_color = [
            (receptor_color[0] * 255.0) as u8,
            (receptor_color[1] * 255.0) as u8,
            (receptor_color[2] * 255.0) as u8,
            (receptor_color[3] * 255.0) as u8,
        ];

        let mut receptor_bind_groups = Vec::new();
        for col in 0..num_columns {
            let receptor_texture = if let Some(path) = skin_temp.get_receptor_path(col) {
                load_texture_from_path(&device, &queue, &path)
                    .map(|(tex, _, _)| tex)
                    .unwrap_or_else(|| {
                        create_default_texture(
                            &device,
                            &queue,
                            receptor_default_color,
                            &format!("Receptor {} Default", col),
                        )
                    })
            } else {
                create_default_texture(
                    &device,
                    &queue,
                    receptor_default_color,
                    &format!("Receptor {} Default", col),
                )
            };

            let receptor_texture_view =
                receptor_texture.create_view(&wgpu::TextureViewDescriptor::default());
            receptor_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Receptor Bind Group {}", col)),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&receptor_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            }));
        }

        let note_color = skin_temp.get_note_color();
        let note_default_color = [
            (note_color[0] * 255.0) as u8,
            (note_color[1] * 255.0) as u8,
            (note_color[2] * 255.0) as u8,
            (note_color[3] * 255.0) as u8,
        ];

        let mut note_bind_groups = Vec::new();
        for col in 0..num_columns {
            let note_texture = if let Some(path) = skin_temp.get_note_path(col) {
                load_texture_from_path(&device, &queue, &path)
                    .map(|(tex, _, _)| tex)
                    .unwrap_or_else(|| {
                        create_default_texture(
                            &device,
                            &queue,
                            note_default_color,
                            &format!("Note {} Default", col),
                        )
                    })
            } else {
                create_default_texture(
                    &device,
                    &queue,
                    note_default_color,
                    &format!("Note {} Default", col),
                )
            };

            let note_texture_view =
                note_texture.create_view(&wgpu::TextureViewDescriptor::default());
            note_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("Note Bind Group {}", col)),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&note_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            }));
        }

        let render_pipeline = create_render_pipeline(&device, &bind_group_layout, config.format);

        // --- Background Setup ---
        let background_sampler = create_sampler(&device);
        let background_bind_group_layout = create_bind_group_layout(&device);
        let background_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Background Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(BACKGROUND_SHADER_SRC)),
        });

        let background_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Background Pipeline Layout"),
                bind_group_layouts: &[&background_bind_group_layout],
                push_constant_ranges: &[],
            });

        let background_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            cache: None,
            label: Some("Background Render Pipeline"),
            layout: Some(&background_pipeline_layout),
            vertex: wgpu::VertexState {
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                module: &background_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                module: &background_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // --- Buffers ---
        let buffer_size = (1000 * std::mem::size_of::<InstanceRaw>()) as u64;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let receptor_buffer_size = (num_columns * std::mem::size_of::<InstanceRaw>()) as u64;
        let receptor_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Receptor Buffer"),
            size: receptor_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // --- Quad Pipeline (Pour les panels) ---
        let quad_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Quad Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(QUAD_SHADER_SRC)),
        });

        let quad_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Quad Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        // Définition de la structure locale pour l'initialisation du layout
        #[repr(C)]
        #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
        struct QuadInstance {
            center: [f32; 2],
            size: [f32; 2],
            color: [f32; 4],
        }

        let quad_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<QuadInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 2]>() * 2) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        };

        let quad_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            cache: None,
            label: Some("Quad Render Pipeline"),
            layout: Some(&quad_pipeline_layout),
            vertex: wgpu::VertexState {
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                module: &quad_shader,
                entry_point: Some("vs_main"),
                buffers: &[quad_vertex_layout],
            },
            fragment: Some(wgpu::FragmentState {
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                module: &quad_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Buffer pour les quads (panneaux, graphiques, etc.)
        // Taille initiale par défaut (sera redimensionné lors du chargement d'une map)
        let quad_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Quad Buffer"),
            size: (100 * std::mem::size_of::<QuadInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // --- Font Loading ---
        let skin = skin_temp;
        let font_path = if let Some(skin_font_path) = skin.get_font_path() {
            if skin_font_path.exists() {
                skin_font_path
            } else {
                PathBuf::from("assets/font.ttf")
            }
        } else {
            PathBuf::from("assets/font.ttf")
        };

        let text_brush =
            load_text_brush(&device, size.width, size.height, config.format, &font_path);
        let pixel_system = PixelSystem::new(size.width, size.height);

        // --- UI Setup ---
        let playfield_config = PlayfieldConfig::new();
        let judgement_colors = skin.get_judgement_colors();

        // Calculs de positionnement initial
        let screen_width = size.width as f32;
        let screen_height = size.height as f32;
        let playfield_component = PlayfieldDisplay::new(playfield_config);
        let (_, _playfield_width) = playfield_component.get_bounds(&pixel_system);
        let playfield_screen_width = _playfield_width * screen_height / 2.0;
        let playfield_center_x = screen_width / 2.0;
        let left_x =
            ((screen_width / 2.0) - playfield_screen_width - (screen_width * 0.15).min(200.0))
                .max(20.0);
        let playfield_right_x = playfield_center_x + (playfield_screen_width / 2.0);
        let score_x = playfield_right_x + 20.0;
        let combo_y = (screen_height / 2.0) - 80.0;
        let judgement_y = combo_y + 30.0;
        let hitbar_y = combo_y + 60.0;
        let hitbar_width = playfield_screen_width * 0.8;

        Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            note_bind_groups,
            receptor_bind_groups,
            instance_buffer,
            receptor_buffer,
            engine: GameEngine::new(),
            text_brush,
            frame_count: 0,
            last_fps_update: Instant::now(),
            fps: 0.0,
            pixel_system,
            skin,
            gameplay_view: GameplayView::new(playfield_component),
            result_view: ResultView::new(screen_width, screen_height),
            hit_bar: HitBarDisplay::new(
                playfield_center_x - hitbar_width / 2.0,
                hitbar_y,
                hitbar_width,
                20.0,
            ),
            score_display: ScoreDisplay::new(score_x, screen_height * 0.05),
            accuracy_panel: AccuracyDisplay::new(left_x, screen_height * 0.1),
            judgements_panel: JudgementPanel::new(left_x, screen_height * 0.15, judgement_colors),
            combo_display: ComboDisplay::new(playfield_center_x, combo_y),
            judgement_flash: JudgementFlash::new(playfield_center_x, judgement_y),
            menu_state,
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
            settings: GameSettings::new(),
            leaderboard_scores_loaded: false,
            current_leaderboard_hash: None,
            song_select_screen: None,
        }
    }
}
