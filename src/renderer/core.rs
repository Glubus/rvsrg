use crate::models::menu::MenuState;
use crate::models::engine::{GameEngine, InstanceRaw, NUM_COLUMNS, PixelSystem, PlayfieldConfig};
use crate::shaders::constants::{BACKGROUND_SHADER_SRC, QUAD_SHADER_SRC};
use crate::models::skin::Skin;
use crate::views::components::{
    AccuracyDisplay, ComboDisplay, HitBarDisplay, JudgementFlash, JudgementPanel, PlayfieldDisplay,
    ScoreDisplay,
};
use crate::views::gameplay::GameplayView;
use crate::views::menu::MenuView;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use wgpu_text::TextBrush;
use winit::window::Window;

use super::pipeline::{create_bind_group_layout, create_render_pipeline, create_sampler};
use super::text::load_text_brush;
use super::texture::{create_default_texture, load_texture_from_path};

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    note_bind_groups: Vec<wgpu::BindGroup>,
    receptor_bind_groups: Vec<wgpu::BindGroup>,
    instance_buffer: wgpu::Buffer,
    receptor_buffer: wgpu::Buffer,
    pub engine: GameEngine,
    text_brush: TextBrush,
    frame_count: u64,
    last_fps_update: Instant,
    fps: f64,
    pixel_system: PixelSystem,
    pub skin: Skin,
    gameplay_view: GameplayView,
    menu_view: MenuView,
    score_display: ScoreDisplay,
    accuracy_panel: AccuracyDisplay,
    judgements_panel: JudgementPanel,
    combo_display: ComboDisplay,
    judgement_flash: JudgementFlash,
    hit_bar: HitBarDisplay,
    pub menu_state: Arc<Mutex<MenuState>>,
    // Background pour le menu
    background_texture: Option<wgpu::Texture>,
    background_bind_group: Option<wgpu::BindGroup>,
    background_pipeline: Option<wgpu::RenderPipeline>,
    background_sampler: wgpu::Sampler,
    current_background_path: Option<String>,
    // Pipeline pour les quads colorés (panels, cards)
    quad_pipeline: wgpu::RenderPipeline,
    quad_buffer: wgpu::Buffer,
}

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

        // Charger le skin par défaut
        let num_columns = NUM_COLUMNS;
        let skin_temp = Skin::load_default(num_columns).unwrap_or_else(|e| {
            eprintln!(
                "Warning: Failed to load default skin: {}. Using fallback.",
                e
            );
            Self::create_fallback_skin()
        });

        // Créer le bind group layout et le sampler
        let bind_group_layout = create_bind_group_layout(&device);
        let sampler = create_sampler(&device);

        // Charger les textures pour les receptors
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
            let receptor_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
            });

            receptor_bind_groups.push(receptor_bind_group);
        }

        // Charger les textures pour les notes
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
            let note_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
            });

            note_bind_groups.push(note_bind_group);
        }

        // Créer le pipeline de rendu
        let render_pipeline = create_render_pipeline(&device, &bind_group_layout, config.format);

        // Créer le pipeline et le sampler pour le background
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

        // Créer les buffers
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

        // Créer le pipeline pour les quads colorés (panels, cards)
        let quad_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Quad Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(QUAD_SHADER_SRC)),
        });

        let quad_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Quad Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        #[repr(C)]
        #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
        struct QuadInstance {
            center: [f32; 2], // Centre du quad en coordonnées normalisées [-1, 1]
            size: [f32; 2],   // Taille du quad en coordonnées normalisées
            color: [f32; 4],  // Couleur RGBA
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
                    offset: (std::mem::size_of::<[f32; 2]>() + std::mem::size_of::<[f32; 2]>())
                        as wgpu::BufferAddress,
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

        // Buffer pour les quads (panels, cards)
        let quad_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Quad Buffer"),
            size: (100 * std::mem::size_of::<QuadInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Charger la police
        let skin = skin_temp;
        let font_path = if let Some(skin_font_path) = skin.get_font_path() {
            eprintln!("Font found in skin: {:?}", skin_font_path);
            if skin_font_path.exists() {
                skin_font_path
            } else {
                eprintln!(
                    "Skin font not found at {:?}, using fallback",
                    skin_font_path
                );
                PathBuf::from("assets/font.ttf")
            }
        } else {
            eprintln!("No font configured in skin, using fallback");
            PathBuf::from("assets/font.ttf")
        };

        let text_brush =
            load_text_brush(&device, size.width, size.height, config.format, &font_path);

        let pixel_system = PixelSystem::new(size.width, size.height);
        let playfield_config = PlayfieldConfig::new();
        let judgement_colors = skin.get_judgement_colors();

        // Calculer les positions des components
        let screen_width = size.width as f32;
        let screen_height = size.height as f32;
        let playfield_component = PlayfieldDisplay::new(playfield_config);
        let (_playfield_x, _playfield_width) = playfield_component.get_bounds(&pixel_system);
        let playfield_screen_width = _playfield_width * screen_height / 2.0;
        let playfield_center_x = screen_width / 2.0;
        let left_x =
            ((screen_width / 2.0) - playfield_screen_width - (screen_width * 0.15).min(200.0))
                .max(20.0);
        let playfield_right_x = playfield_center_x + (playfield_screen_width / 2.0);
        let score_x = playfield_right_x + 20.0;
        let hit_line_y_screen = screen_height / 2.0;
        let combo_y = hit_line_y_screen - 80.0;
        let judgement_y = combo_y + 30.0;
        let hitbar_y = combo_y + 60.0;
        let hitbar_width = playfield_screen_width * 0.8;

        let gameplay_view = GameplayView::new(playfield_component);
        let menu_view = MenuView::new();

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
            gameplay_view,
            menu_view,
            hit_bar: HitBarDisplay::new(
                playfield_center_x - hitbar_width / 2.0,
                hitbar_y,
                hitbar_width,
                20.0,
            ),
            score_display: ScoreDisplay::new(score_x, screen_height * 0.05),
            accuracy_panel: AccuracyDisplay::new(left_x, screen_height * 0.1),
            judgements_panel: crate::views::components::JudgementPanel::new(
                left_x,
                screen_height * 0.15,
                judgement_colors,
            ),
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
        }
    }

    /// Met à jour le background du menu si la sélection a changé
    fn update_menu_background(&mut self) {
        let selected_beatmapset = {
            if let Ok(menu_state) = self.menu_state.lock() {
                menu_state
                    .get_selected_beatmapset()
                    .and_then(|(bs, _)| bs.image_path.as_ref().map(|s| s.clone()))
            } else {
                None
            }
        };

        if let Some(image_path) = selected_beatmapset {
            // Vérifier si le background a changé
            if self.current_background_path.as_ref() != Some(&image_path) {
                self.current_background_path = Some(image_path.clone());

                // Charger la nouvelle texture
                let path = std::path::Path::new(&image_path);
                if path.exists() {
                    if let Some((texture, _, _)) =
                        load_texture_from_path(&self.device, &self.queue, path)
                    {
                        let texture_view =
                            texture.create_view(&wgpu::TextureViewDescriptor::default());
                        let bind_group_layout = create_bind_group_layout(&self.device);
                        let bind_group =
                            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                                label: Some("Background Bind Group"),
                                layout: &bind_group_layout,
                                entries: &[
                                    wgpu::BindGroupEntry {
                                        binding: 0,
                                        resource: wgpu::BindingResource::TextureView(&texture_view),
                                    },
                                    wgpu::BindGroupEntry {
                                        binding: 1,
                                        resource: wgpu::BindingResource::Sampler(
                                            &self.background_sampler,
                                        ),
                                    },
                                ],
                            });

                        self.background_texture = Some(texture);
                        self.background_bind_group = Some(bind_group);
                    }
                } else {
                    // Si le fichier n'existe pas, on garde l'ancien background ou on le vide
                    self.background_texture = None;
                    self.background_bind_group = None;
                }
            }
        } else {
            // Pas de background sélectionné
            if self.current_background_path.is_some() {
                self.current_background_path = None;
                self.background_texture = None;
                self.background_bind_group = None;
            }
        }
    }

    fn create_fallback_skin() -> Skin {
        Skin {
            config: crate::models::skin::SkinConfig {
                skin: crate::models::skin::SkinInfo {
                    name: "Fallback".to_string(),
                    version: "1.0.0".to_string(),
                    author: "System".to_string(),
                    font: None,
                },
                images: crate::models::skin::ImagePaths {
                    receptor: None,
                    receptor_0: None,
                    receptor_1: None,
                    receptor_2: None,
                    receptor_3: None,
                    receptor_4: None,
                    receptor_5: None,
                    receptor_6: None,
                    receptor_7: None,
                    receptor_8: None,
                    receptor_9: None,
                    note: None,
                    note_0: None,
                    note_1: None,
                    note_2: None,
                    note_3: None,
                    note_4: None,
                    note_5: None,
                    note_6: None,
                    note_7: None,
                    note_8: None,
                    note_9: None,
                    miss_note: None,
                    background: None,
                },
                keys: None,
                colors: None,
                ui_positions: None,
            },
            base_path: PathBuf::from("skins/default"),
            key_to_column: {
                let mut map = std::collections::HashMap::new();
                map.insert("KeyD".to_string(), 0);
                map.insert("KeyF".to_string(), 1);
                map.insert("KeyJ".to_string(), 2);
                map.insert("KeyK".to_string(), 3);
                map
            },
        }
    }

    pub fn load_map(&mut self, map_path: PathBuf) {
        // Get rate from menu_state
        let rate = {
            if let Ok(menu_state) = self.menu_state.lock() {
                menu_state.rate
            } else {
                1.0
            }
        };
        self.engine = GameEngine::from_map(map_path, rate);

        // Rate is already applied in from_map via sink.set_speed()
    }

    pub fn stop_audio(&mut self) {
        if let Ok(sink) = self.engine.audio_sink.lock() {
            sink.stop();
            sink.clear();
        }
    }

    pub fn decrease_note_size(&mut self) {
        self.gameplay_view
            .playfield_component_mut()
            .config
            .decrease_note_size();
    }

    pub fn increase_note_size(&mut self) {
        self.gameplay_view
            .playfield_component_mut()
            .config
            .increase_note_size();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.text_brush
                .resize_view(new_size.width as f32, new_size.height as f32, &self.queue);
            self.pixel_system
                .update_size(new_size.width, new_size.height);
            self.update_component_positions();
        }
    }

    fn update_component_positions(&mut self) {
        let screen_width = self.config.width as f32;
        let screen_height = self.config.height as f32;

        let (_playfield_x, _playfield_width) = self
            .gameplay_view
            .playfield_component()
            .get_bounds(&self.pixel_system);
        let playfield_screen_width = _playfield_width * screen_height / 2.0;
        let playfield_center_x = screen_width / 2.0;
        let left_x =
            ((screen_width / 2.0) - playfield_screen_width - (screen_width * 0.15).min(200.0))
                .max(20.0);
        let playfield_right_x = playfield_center_x + (playfield_screen_width / 2.0);
        let score_x = playfield_right_x + 20.0;
        let hit_line_y_screen = screen_height / 2.0;
        let combo_y = hit_line_y_screen - 80.0;
        let judgement_y = combo_y + 30.0;
        let hitbar_y = combo_y + 60.0;
        let hitbar_width = playfield_screen_width * 0.8;

        self.combo_display.set_position(playfield_center_x, combo_y);
        self.judgement_flash
            .set_position(playfield_center_x, judgement_y);
        self.hit_bar.set_geometry(
            playfield_center_x - hitbar_width / 2.0,
            hitbar_y,
            hitbar_width,
            20.0,
        );
        self.score_display
            .set_position(score_x, screen_height * 0.05);
        self.accuracy_panel
            .set_position(left_x, screen_height * 0.1);
        self.judgements_panel
            .set_position(left_x, screen_height * 0.15);
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let in_menu = {
            if let Ok(menu_state) = self.menu_state.lock() {
                menu_state.in_menu
            } else {
                false
            }
        };

        if in_menu {
            // Mettre à jour le background si nécessaire
            self.update_menu_background();

            // Calcul des FPS pour le menu
            self.frame_count += 1;
            let now = Instant::now();
            let elapsed = now.duration_since(self.last_fps_update).as_secs_f64();
            if elapsed >= 0.5 {
                self.fps = self.frame_count as f64 / elapsed;
                self.frame_count = 0;
                self.last_fps_update = now;
            }

            self.menu_view.render(
                &self.device,
                &self.queue,
                &mut self.text_brush,
                &self.menu_state,
                self.config.width as f32,
                self.config.height as f32,
                self.fps,
                &view,
                self.background_pipeline.as_ref(),
                self.background_bind_group.as_ref(),
                &self.quad_pipeline,
                &self.quad_buffer,
            )?;

            self.queue.submit(std::iter::once(
                self.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor::default())
                    .finish(),
            ));
            output.present();
            return Ok(());
        }

        // Calcul des FPS pour le gameplay
        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_fps_update).as_secs_f64();
        if elapsed >= 0.5 {
            self.fps = self.frame_count as f64 / elapsed;
            self.frame_count = 0;
            self.last_fps_update = now;
        }

        self.gameplay_view.render(
            &self.device,
            &self.queue,
            &mut self.text_brush,
            &self.render_pipeline,
            &self.instance_buffer,
            &self.receptor_buffer,
            &self.note_bind_groups,
            &self.receptor_bind_groups,
            &mut self.engine,
            &self.pixel_system,
            &mut self.score_display,
            &mut self.accuracy_panel,
            &mut self.judgements_panel,
            &mut self.combo_display,
            &mut self.judgement_flash,
            &mut self.hit_bar,
            self.config.width as f32,
            self.config.height as f32,
            self.fps,
            &view,
        )?;

        output.present();
        Ok(())
    }
}
