//! Render resources (pipelines, buffers, bind groups).



use crate::models::engine::{InstanceRaw, NUM_COLUMNS, PixelSystem, PlayfieldConfig};
use crate::models::settings::SettingsState;
use crate::models::skin::Skin;
use crate::render::context::RenderContext;
use crate::render::utils::*;
use crate::shaders::constants::{BACKGROUND_SHADER_SRC, PROGRESS_SHADER_SRC, QUAD_SHADER_SRC};
use crate::views::components::common::primitives::ProgressInstance; // From primitives
use crate::views::components::{
    AccuracyDisplay, ComboDisplay, HitBarDisplay, JudgementFlash, JudgementPanel,
    NotesRemainingDisplay, NpsDisplay, PlayfieldDisplay, ScoreDisplay, ScrollSpeedDisplay,
    TimeLeftDisplay,
};
use crate::views::gameplay::GameplayView;
use std::path::PathBuf;

pub struct RenderResources {
    pub render_pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout, // NEW: Persist for reloads
    pub background_pipeline: wgpu::RenderPipeline,
    pub quad_pipeline: wgpu::RenderPipeline,
    pub progress_pipeline: wgpu::RenderPipeline,

    pub instance_buffer: wgpu::Buffer,
    pub receptor_buffer: wgpu::Buffer,
    pub quad_buffer: wgpu::Buffer,
    pub progress_buffer: wgpu::Buffer, // NEW

    pub note_bind_groups: Vec<wgpu::BindGroup>,
    pub receptor_bind_groups: Vec<wgpu::BindGroup>,
    pub receptor_pressed_bind_groups: Vec<wgpu::BindGroup>,

    // Special note type bind groups
    pub mine_bind_group: Option<wgpu::BindGroup>,
    pub hold_body_bind_group: Option<wgpu::BindGroup>,
    pub hold_end_bind_group: Option<wgpu::BindGroup>,
    pub burst_body_bind_group: Option<wgpu::BindGroup>,
    pub burst_end_bind_group: Option<wgpu::BindGroup>,

    pub background_bind_group: Option<wgpu::BindGroup>,
    pub background_sampler: wgpu::Sampler,
    pub current_background_path: Option<String>,

    pub song_button_texture: Option<egui::TextureHandle>,
    pub song_button_selected_texture: Option<egui::TextureHandle>,
    pub difficulty_button_texture: Option<egui::TextureHandle>,
    pub difficulty_button_selected_texture: Option<egui::TextureHandle>,

    pub beatmap_info_bg_texture: Option<egui::TextureHandle>,
    pub search_panel_bg_texture: Option<egui::TextureHandle>,
    pub search_bar_texture: Option<egui::TextureHandle>,
    pub leaderboard_bg_texture: Option<egui::TextureHandle>,

    pub text_brush: wgpu_text::TextBrush,
    pub pixel_system: PixelSystem,

    pub skin: Skin,
    pub settings: SettingsState,

    pub editor_status_text: Option<String>,
    pub editor_values_text: Option<String>,
    pub leaderboard_scores_loaded: bool,
    pub current_leaderboard_hash: Option<String>,

    pub gameplay_view: GameplayView,
    pub score_display: ScoreDisplay,
    pub accuracy_panel: AccuracyDisplay,
    pub judgements_panel: JudgementPanel,
    pub combo_display: ComboDisplay,
    pub judgement_flash: JudgementFlash,
    pub hit_bar: HitBarDisplay,
    pub nps_display: NpsDisplay,
    // NEW: Separate display components
    pub notes_remaining_display: NotesRemainingDisplay,
    pub scroll_speed_display: ScrollSpeedDisplay,
    pub time_left_display: TimeLeftDisplay,
}

impl RenderResources {
    pub fn reload_textures(&mut self, ctx: &RenderContext, egui_ctx: &egui::Context, skin: &Skin) {
        self.reload_menu_assets(egui_ctx, skin);
        self.reload_gameplay_assets(ctx, skin);
    }

    fn reload_menu_assets(&mut self, egui_ctx: &egui::Context, skin: &Skin) {
        let load_egui_tex = |path: Option<PathBuf>, name: &str| -> Option<egui::TextureHandle> {
            let p = path?;
            if !p.exists() {
                return None;
            }
            let image = match image::open(&p) {
                Ok(img) => img,
                Err(e) => {
                    log::warn!("Failed to load menu texture {:?}: {}", p, e);
                    return None;
                }
            };
            let size = [image.width() as usize, image.height() as usize];
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
            Some(egui_ctx.load_texture(name, color_image, Default::default()))
        };

        // Load menu textures
        self.song_button_texture = load_egui_tex(skin.get_song_button_image(), "song_btn");
        self.song_button_selected_texture =
            load_egui_tex(skin.get_song_button_selected_image(), "song_btn_sel");
        self.difficulty_button_texture =
            load_egui_tex(skin.get_difficulty_button_image(), "diff_btn");
        self.difficulty_button_selected_texture =
            load_egui_tex(skin.get_difficulty_button_selected_image(), "diff_btn_sel");

        self.beatmap_info_bg_texture =
            load_egui_tex(skin.get_beatmap_info_background_image(), "beatmap_info_bg");
        self.search_panel_bg_texture =
            load_egui_tex(skin.get_search_panel_background_image(), "search_panel_bg");
        self.search_bar_texture = load_egui_tex(skin.get_search_bar_image(), "search_bar");
        self.leaderboard_bg_texture =
            load_egui_tex(skin.get_leaderboard_background_image(), "leaderboard_bg");
    }

    fn reload_gameplay_assets(&mut self, ctx: &RenderContext, skin: &Skin) {
        let device = &ctx.device;
        let queue = &ctx.queue;

        let bind_group_layout = &self.bind_group_layout;
        let sampler = create_sampler(device);

        // Helper for single texture bind group
        let create_bind_group_from_path =
            |path: Option<PathBuf>, label: &str| -> Option<wgpu::BindGroup> {
                let p = path?;
                let (tex, _, _) = load_texture_from_path(device, queue, &p)?;
                let view = tex.create_view(&Default::default());
                Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(label),
                    layout: &bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                }))
            };

        // Reload special notes
        self.mine_bind_group =
            create_bind_group_from_path(skin.get_mine_image(NUM_COLUMNS, 0), "Mine BG");
        self.hold_body_bind_group =
            create_bind_group_from_path(skin.get_hold_body_image(NUM_COLUMNS, 0), "Hold Body BG");
        self.hold_end_bind_group =
            create_bind_group_from_path(skin.get_hold_end_image(NUM_COLUMNS, 0), "Hold End BG");
        self.burst_body_bind_group =
            create_bind_group_from_path(skin.get_burst_body_image(NUM_COLUMNS, 0), "Burst Body BG");
        self.burst_end_bind_group =
            create_bind_group_from_path(skin.get_burst_end_image(NUM_COLUMNS, 0), "Burst End BG");

        // Reload columns (Notes & Receptors)
        self.receptor_bind_groups.clear();
        self.receptor_pressed_bind_groups.clear();
        self.note_bind_groups.clear();

        let receptor_color = skin.gameplay.receptors.color;
        let def_col_rec = [
            (receptor_color[0] * 255.) as u8,
            (receptor_color[1] * 255.) as u8,
            (receptor_color[2] * 255.) as u8,
            (receptor_color[3] * 255.) as u8,
        ];

        let note_color = skin.gameplay.notes.note.color;
        let def_col_note = [
            (note_color[0] * 255.) as u8,
            (note_color[1] * 255.) as u8,
            (note_color[2] * 255.) as u8,
            (note_color[3] * 255.) as u8,
        ];

        for col in 0..NUM_COLUMNS {
            // Receptor
            let path = skin.get_receptor_image(NUM_COLUMNS, col);
            let tex = path
                .as_ref()
                .and_then(|p| load_texture_from_path(device, queue, p).map(|(t, _, _)| t))
                .unwrap_or_else(|| {
                    create_default_texture(device, queue, def_col_rec, "Def Receptor")
                });
            let view = tex.create_view(&Default::default());
            self.receptor_bind_groups
                .push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Receptor BG"),
                    layout: &bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                }));

            // Pressed
            let path_p = skin
                .get_receptor_pressed_image(NUM_COLUMNS, col)
                .or(path.clone());
            let tex_p = path_p
                .as_ref()
                .and_then(|p| load_texture_from_path(device, queue, p).map(|(t, _, _)| t))
                .unwrap_or_else(|| {
                    create_default_texture(device, queue, def_col_rec, "Def Pressed")
                });
            let view_p = tex_p.create_view(&Default::default());
            self.receptor_pressed_bind_groups
                .push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Pressed BG"),
                    layout: &bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view_p),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                }));

            // Note
            let path_n = skin.get_note_image(NUM_COLUMNS, col);
            let tex_n = path_n
                .as_ref()
                .and_then(|p| load_texture_from_path(device, queue, p).map(|(t, _, _)| t))
                .unwrap_or_else(|| create_default_texture(device, queue, def_col_note, "Def Note"));
            let view_n = tex_n.create_view(&Default::default());
            self.note_bind_groups
                .push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Note BG"),
                    layout: &bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view_n),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                }));
        }
    }

    pub fn new(ctx: &RenderContext, egui_ctx: &egui::Context) -> Self {
        let device = &ctx.device;
        let config = &ctx.config;

        let settings = SettingsState::load();
        let _ = crate::models::skin::init_skin_structure();
        let mut skin = Skin::load(&settings.current_skin)
            .or_else(|_| Skin::load("default"))
            .unwrap_or_else(|e| {
                log::error!("RESOURCES: Failed to load any skin: {}", e);
                Skin::default()
            });
        skin.load_key_mode(NUM_COLUMNS);

        let load_egui_tex = |path: Option<PathBuf>, name: &str| -> Option<egui::TextureHandle> {
            let p = path?;
            if !p.exists() {
                return None;
            }
            let image = image::open(p).ok()?;
            let size = [image.width() as usize, image.height() as usize];
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
            Some(egui_ctx.load_texture(name, color_image, Default::default()))
        };

        // Load menu textures using new API
        let song_button_texture = load_egui_tex(skin.get_song_button_image(), "song_btn");
        let song_button_selected_texture =
            load_egui_tex(skin.get_song_button_selected_image(), "song_btn_sel");
        let difficulty_button_texture =
            load_egui_tex(skin.get_difficulty_button_image(), "diff_btn");
        let difficulty_button_selected_texture =
            load_egui_tex(skin.get_difficulty_button_selected_image(), "diff_btn_sel");

        let beatmap_info_bg_texture =
            load_egui_tex(skin.get_beatmap_info_background_image(), "beatmap_info_bg");
        let search_panel_bg_texture =
            load_egui_tex(skin.get_search_panel_background_image(), "search_panel_bg");
        let search_bar_texture = load_egui_tex(skin.get_search_bar_image(), "search_bar");
        let leaderboard_bg_texture =
            load_egui_tex(skin.get_leaderboard_background_image(), "leaderboard_bg");

        let bind_group_layout = create_bind_group_layout(device);
        let render_pipeline = create_render_pipeline(device, &bind_group_layout, config.format);
        let sampler = create_sampler(device);

        let bg_sampler = create_sampler(device);
        let bg_layout = create_bind_group_layout(device);
        let bg_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("BG Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(BACKGROUND_SHADER_SRC)),
        });
        let background_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("BG Pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("BG Layout"),
                    bind_group_layouts: &[&bg_layout],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: wgpu::VertexState {
                module: &bg_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &bg_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let quad_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Quad Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(QUAD_SHADER_SRC)),
        });
        let quad_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Quad Pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Quad Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: wgpu::VertexState {
                module: &quad_shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 32,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 8,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 16,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &quad_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (2000 * std::mem::size_of::<InstanceRaw>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let receptor_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Receptor Buffer"),
            size: (NUM_COLUMNS as u64 * std::mem::size_of::<InstanceRaw>() as u64),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let quad_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Quad Buffer"),
            size: (1000 * 32) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // PROGRESS PIPELINE
        let progress_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Progress Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(PROGRESS_SHADER_SRC)),
        });
        let progress_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Progress Pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Progress Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: wgpu::VertexState {
                module: &progress_shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<ProgressInstance>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0, // center
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 8,
                            shader_location: 1, // size
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 16,
                            shader_location: 2, // filled_color
                            format: wgpu::VertexFormat::Float32x4,
                        },
                        wgpu::VertexAttribute {
                            offset: 32,
                            shader_location: 3, // empty_color
                            format: wgpu::VertexFormat::Float32x4,
                        },
                        wgpu::VertexAttribute {
                            offset: 48,
                            shader_location: 4, // progress
                            format: wgpu::VertexFormat::Float32,
                        },
                        wgpu::VertexAttribute {
                            offset: 52,
                            shader_location: 5, // mode
                            format: wgpu::VertexFormat::Uint32,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &progress_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let progress_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Progress Buffer"),
            size: (100 * std::mem::size_of::<ProgressInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut receptor_bind_groups = Vec::new();
        let mut receptor_pressed_bind_groups = Vec::new();
        let mut note_bind_groups = Vec::new();

        // Get colors from new structure
        let receptor_color = skin.gameplay.receptors.color;
        let def_col = [
            (receptor_color[0] * 255.) as u8,
            (receptor_color[1] * 255.) as u8,
            (receptor_color[2] * 255.) as u8,
            (receptor_color[3] * 255.) as u8,
        ];

        for col in 0..NUM_COLUMNS {
            let path = skin.get_receptor_image(NUM_COLUMNS, col);
            let tex = path
                .as_ref()
                .and_then(|p| load_texture_from_path(device, &ctx.queue, p).map(|(t, _, _)| t))
                .unwrap_or_else(|| {
                    create_default_texture(device, &ctx.queue, def_col, "Def Receptor")
                });
            let view = tex.create_view(&Default::default());
            receptor_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Receptor BG"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            }));

            let path_p = skin
                .get_receptor_pressed_image(NUM_COLUMNS, col)
                .or(path.clone());
            let tex_p = path_p
                .as_ref()
                .and_then(|p| load_texture_from_path(device, &ctx.queue, p).map(|(t, _, _)| t))
                .unwrap_or_else(|| {
                    create_default_texture(device, &ctx.queue, def_col, "Def Pressed")
                });
            let view_p = tex_p.create_view(&Default::default());
            receptor_pressed_bind_groups.push(device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("Pressed BG"),
                    layout: &bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view_p),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                },
            ));

            let path_n = skin.get_note_image(NUM_COLUMNS, col);
            let note_color = skin.gameplay.notes.note.color;
            let tex_n = path_n
                .as_ref()
                .and_then(|p| load_texture_from_path(device, &ctx.queue, p).map(|(t, _, _)| t))
                .unwrap_or_else(|| {
                    create_default_texture(
                        device,
                        &ctx.queue,
                        [
                            (note_color[0] * 255.) as u8,
                            (note_color[1] * 255.) as u8,
                            (note_color[2] * 255.) as u8,
                            (note_color[3] * 255.) as u8,
                        ],
                        "Def Note",
                    )
                });
            let view_n = tex_n.create_view(&Default::default());
            note_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Note BG"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view_n),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            }));
        }

        let create_bind_group_from_path =
            |path: Option<PathBuf>, label: &str| -> Option<wgpu::BindGroup> {
                let p = path?;
                let (tex, _, _) = load_texture_from_path(device, &ctx.queue, &p)?;
                let view = tex.create_view(&Default::default());
                Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(label),
                    layout: &bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                }))
            };

        let mine_bind_group =
            create_bind_group_from_path(skin.get_mine_image(NUM_COLUMNS, 0), "Mine BG");
        let hold_body_bind_group =
            create_bind_group_from_path(skin.get_hold_body_image(NUM_COLUMNS, 0), "Hold Body BG");
        let hold_end_bind_group =
            create_bind_group_from_path(skin.get_hold_end_image(NUM_COLUMNS, 0), "Hold End BG");
        let burst_body_bind_group =
            create_bind_group_from_path(skin.get_burst_body_image(NUM_COLUMNS, 0), "Burst Body BG");
        let burst_end_bind_group =
            create_bind_group_from_path(skin.get_burst_end_image(NUM_COLUMNS, 0), "Burst End BG");

        let font_path = skin
            .get_font_path()
            .unwrap_or(PathBuf::from("assets/font.ttf"));
        let text_brush = load_text_brush(
            device,
            config.width,
            config.height,
            config.format,
            Some(&font_path),
        );
        let pixel_system = PixelSystem::new(config.width, config.height);

        let mut pf_config = PlayfieldConfig::new();
        pf_config.column_width_pixels = skin.gameplay.playfield.column_width;

        // Get judgement PANEL colors from judgement_panel config (SEPARATE from flash)
        let colors = crate::models::stats::JudgementColors {
            marv: skin.hud.judgement_panel.marv_color,
            perfect: skin.hud.judgement_panel.perfect_color,
            great: skin.hud.judgement_panel.great_color,
            good: skin.hud.judgement_panel.good_color,
            bad: skin.hud.judgement_panel.bad_color,
            miss: skin.hud.judgement_panel.miss_color,
            ghost_tap: skin.hud.judgement_panel.ghost_tap_color,
        };

        let mut res = Self {
            render_pipeline,
            bind_group_layout, // NEW: Stored
            background_pipeline,
            quad_pipeline,
            progress_pipeline, // NEW
            instance_buffer,
            receptor_buffer,
            quad_buffer,
            progress_buffer, // NEW
            note_bind_groups: Vec::new(),
            receptor_bind_groups: Vec::new(),
            receptor_pressed_bind_groups: Vec::new(),
            background_bind_group: None,
            background_sampler: bg_sampler,
            current_background_path: None,

            song_button_texture: None,
            song_button_selected_texture: None,
            difficulty_button_texture: None,
            difficulty_button_selected_texture: None,

            beatmap_info_bg_texture: None,
            search_panel_bg_texture: None,
            search_bar_texture: None,
            leaderboard_bg_texture: None,

            mine_bind_group: None,
            hold_body_bind_group: None,
            hold_end_bind_group: None,
            burst_body_bind_group: None,
            burst_end_bind_group: None,

            text_brush,
            pixel_system,
            skin,
            settings,

            editor_status_text: None,
            editor_values_text: None,
            leaderboard_scores_loaded: false,
            current_leaderboard_hash: None,

            gameplay_view: GameplayView::new(PlayfieldDisplay::new(pf_config)),
            score_display: ScoreDisplay::new(0., 0.),
            accuracy_panel: AccuracyDisplay::new(0., 0.),
            judgements_panel: JudgementPanel::new(0., 0., colors),
            combo_display: ComboDisplay::new(0., 0.),
            judgement_flash: JudgementFlash::new(0., 0.),
            hit_bar: HitBarDisplay::new(0., 0., 100., 20.),
            nps_display: NpsDisplay::new(0., 0.),
            // NEW: Separate display components
            notes_remaining_display: NotesRemainingDisplay::new(0., 0.),
            scroll_speed_display: ScrollSpeedDisplay::new(0., 0.),
            time_left_display: TimeLeftDisplay::new(0., 0.),
        };

        let skin_clone = res.skin.clone();
        res.reload_textures(ctx, egui_ctx, &skin_clone);

        res.update_component_positions(config.width as f32, config.height as f32);
        res
    }

    pub fn update_component_positions(&mut self, screen_width: f32, screen_height: f32) {
        let hud = &self.skin.hud;
        let gameplay = &self.skin.gameplay;

        // 1. Mise à jour Playfield
        let pf = self.gameplay_view.playfield_component_mut();

        pf.config.note_width_pixels = gameplay.playfield.note_size.x;
        pf.config.note_height_pixels = gameplay.playfield.note_size.y;
        pf.config.receptor_width_pixels = gameplay.playfield.receptor_size.x;
        pf.config.receptor_height_pixels = gameplay.playfield.receptor_size.y;
        pf.config.receptor_spacing_pixels = gameplay.playfield.receptor_spacing;
        pf.config.column_width_pixels = gameplay.playfield.column_width;

        let playfield_width_px = pf.get_total_width_pixels();
        // Centrage: x = 640 est le centre de 1280.
        let x_offset = gameplay.playfield.position.x - (screen_width / 2.0);
        let y_offset = gameplay.playfield.position.y;

        pf.config.x_offset_pixels = x_offset;
        pf.config.y_offset_pixels = y_offset;

        // 2. Mise à jour HUD
        self.score_display
            .set_position(hud.score.position.x, hud.score.position.y);
        self.score_display.set_size(hud.score.scale);

        self.combo_display
            .set_position(hud.combo.position.x, hud.combo.position.y);
        self.combo_display.set_size(hud.combo.scale);

        self.accuracy_panel
            .set_position(hud.accuracy.position.x, hud.accuracy.position.y);
        self.accuracy_panel.set_size(hud.accuracy.scale);

        // Judgement Panel - uses its OWN separate position from judgement_panel config
        self.judgements_panel.set_position(
            hud.judgement_panel.position.x,
            hud.judgement_panel.position.y,
        );
        self.judgements_panel
            .set_size(hud.judgement_panel.text_scale);

        self.nps_display
            .set_position(hud.nps.position.x, hud.nps.position.y);
        self.nps_display.set_size(hud.nps.scale);

        let hitbar_width = playfield_width_px * 0.8;
        self.hit_bar.set_geometry(
            hud.hit_bar.position.x - hitbar_width / 2.0,
            hud.hit_bar.position.y,
            hitbar_width,
            hud.hit_bar.scale,
        );

        // Judgement Flash - uses the marv position as central flash position
        self.judgement_flash
            .set_position(hud.judgement.marv.position.x, hud.judgement.marv.position.y);

        // Set timing indicator option from skin config
        self.judgement_flash.show_timing = hud.judgement.show_timing;

        // NEW: Notes Remaining display (separate from judgement panel)
        self.notes_remaining_display.set_position(
            hud.notes_remaining.position.x,
            hud.notes_remaining.position.y,
        );
        self.notes_remaining_display
            .set_scale(hud.notes_remaining.scale);
        self.notes_remaining_display
            .set_color(hud.notes_remaining.color);
        self.notes_remaining_display
            .set_format(hud.notes_remaining.format.clone());
        self.notes_remaining_display.visible = hud.notes_remaining.visible;

        // NEW: Scroll Speed display (separate from judgement panel)
        self.scroll_speed_display
            .set_position(hud.scroll_speed.position.x, hud.scroll_speed.position.y);
        self.scroll_speed_display.set_scale(hud.scroll_speed.scale);
        self.scroll_speed_display.set_color(hud.scroll_speed.color);
        self.scroll_speed_display
            .set_format(hud.scroll_speed.format.clone());
        self.scroll_speed_display.visible = hud.scroll_speed.visible;

        // NEW: Time Left display
        self.time_left_display
            .set_position(hud.time_left.position.x, hud.time_left.position.y);
        self.time_left_display
            .set_size(hud.time_left.size.x, hud.time_left.size.y);
        self.time_left_display
            .set_text_scale(hud.time_left.text_scale);
        self.time_left_display
            .set_text_color(hud.time_left.text_color);
        self.time_left_display
            .set_progress_color(hud.time_left.progress_color);
        self.time_left_display
            .set_background_color(hud.time_left.background_color);
        self.time_left_display
            .set_format(hud.time_left.format.clone());
        self.time_left_display.visible = hud.time_left.visible;

        // Convert config mode to display mode
        use crate::models::skin::hud::time_left::TimeDisplayMode as ConfigMode;
        use crate::views::components::gameplay::time_left::TimeDisplayMode as DisplayMode;
        let display_mode = match hud.time_left.mode {
            ConfigMode::Bar => DisplayMode::Bar,
            ConfigMode::Circle => DisplayMode::Circle,
            ConfigMode::Text => DisplayMode::Text,
        };
        self.time_left_display.set_mode(display_mode);
    }

    pub fn load_background(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, path_str: &str) {
        if let Some(current) = &self.current_background_path
            && current == path_str
        {
            return;
        }

        let path = std::path::Path::new(path_str);
        if !path.exists() {
            log::warn!("Background not found: {:?}", path);
            return;
        }

        if let Some((texture, _, _)) = load_texture_from_path(device, queue, path) {
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let layout = self.background_pipeline.get_bind_group_layout(0);

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Background BG"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.background_sampler),
                    },
                ],
            });

            self.background_bind_group = Some(bind_group);
            self.current_background_path = Some(path_str.to_string());
            log::info!("RENDER: Background loaded: {:?}", path);
        }
    }
}

