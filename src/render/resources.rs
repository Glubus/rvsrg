use crate::models::engine::{InstanceRaw, NUM_COLUMNS, PixelSystem, PlayfieldConfig};
use crate::models::settings::SettingsState;
use crate::models::skin::{Skin, UIElementPos};
use crate::render::context::RenderContext;
use crate::render::utils::*;
use crate::shaders::constants::{BACKGROUND_SHADER_SRC, QUAD_SHADER_SRC};
use crate::views::components::{
    AccuracyDisplay, ComboDisplay, HitBarDisplay, JudgementFlash, JudgementPanel, PlayfieldDisplay,
    ScoreDisplay,
};
use crate::views::gameplay::GameplayView;
use std::path::PathBuf;
use wgpu::util::DeviceExt;

pub struct RenderResources {
    pub render_pipeline: wgpu::RenderPipeline,
    pub background_pipeline: wgpu::RenderPipeline,
    pub quad_pipeline: wgpu::RenderPipeline,

    pub instance_buffer: wgpu::Buffer,
    pub receptor_buffer: wgpu::Buffer,
    pub quad_buffer: wgpu::Buffer,

    pub note_bind_groups: Vec<wgpu::BindGroup>,
    pub receptor_bind_groups: Vec<wgpu::BindGroup>,
    pub receptor_pressed_bind_groups: Vec<wgpu::BindGroup>,

    pub background_bind_group: Option<wgpu::BindGroup>,
    pub background_sampler: wgpu::Sampler,
    pub current_background_path: Option<String>,

    pub song_button_texture: Option<egui::TextureHandle>,
    pub song_button_selected_texture: Option<egui::TextureHandle>,
    pub difficulty_button_texture: Option<egui::TextureHandle>,
    pub difficulty_button_selected_texture: Option<egui::TextureHandle>,

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
}

impl RenderResources {
    pub fn new(ctx: &RenderContext, egui_ctx: &egui::Context) -> Self {
        let device = &ctx.device;
        let config = &ctx.config;

        let settings = SettingsState::load();
        let _ = crate::models::skin::init_skin_structure();
        let mut skin =
            Skin::load(&settings.current_skin).unwrap_or_else(|_| Skin::load("default").unwrap());
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

        let song_button_texture = load_egui_tex(skin.song_button.clone(), "song_btn");
        let song_button_selected_texture =
            load_egui_tex(skin.song_button_selected.clone(), "song_btn_sel");
        let difficulty_button_texture = load_egui_tex(skin.difficulty_button.clone(), "diff_btn");
        let difficulty_button_selected_texture =
            load_egui_tex(skin.difficulty_button_selected.clone(), "diff_btn_sel");

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

        let mut receptor_bind_groups = Vec::new();
        let mut receptor_pressed_bind_groups = Vec::new();
        let mut note_bind_groups = Vec::new();

        let receptor_color = skin.colors.receptor_color;
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
            let tex_n = path_n
                .as_ref()
                .and_then(|p| load_texture_from_path(device, &ctx.queue, p).map(|(t, _, _)| t))
                .unwrap_or_else(|| {
                    create_default_texture(
                        device,
                        &ctx.queue,
                        [(skin.colors.note_color[0] * 255.) as u8, 255, 255, 255],
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
        pf_config.column_width_pixels = skin.config.column_width_px;

        let colors = crate::models::stats::JudgementColors {
            marv: skin.colors.marv,
            perfect: skin.colors.perfect,
            great: skin.colors.great,
            good: skin.colors.good,
            bad: skin.colors.bad,
            miss: skin.colors.miss,
            ghost_tap: skin.colors.ghost_tap,
        };

        let mut res = Self {
            render_pipeline,
            background_pipeline,
            quad_pipeline,
            instance_buffer,
            receptor_buffer,
            quad_buffer,
            note_bind_groups,
            receptor_bind_groups,
            receptor_pressed_bind_groups,
            background_bind_group: None,
            background_sampler: bg_sampler,
            current_background_path: None,

            song_button_texture,
            song_button_selected_texture,
            difficulty_button_texture,
            difficulty_button_selected_texture,

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
        };

        // On applique la config initiale
        res.apply_skin_config(config.width as f32, config.height as f32);
        res
    }

    // Helper that previously lived out-of-line, now embedded to avoid linker issues.
    pub fn apply_skin_config(&mut self, screen_width: f32, screen_height: f32) {
        // Same implementation as before, just included directly here.
        self.update_component_positions(screen_width, screen_height);
    }

    pub fn update_component_positions(&mut self, screen_width: f32, screen_height: f32) {
        let config = &self.skin.config;

        // 1. Update playfield assets.
        let pf = self.gameplay_view.playfield_component_mut();
        pf.config.note_width_pixels = config.note_width_px;
        pf.config.note_height_pixels = config.note_height_px;
        pf.config.receptor_width_pixels = config.receptor_width_px;
        pf.config.receptor_height_pixels = config.receptor_height_px;
        pf.config.receptor_spacing_pixels = config.receptor_spacing_px;

        let playfield_width_px = pf.get_total_width_pixels();
        let playfield_center_x = if let Some(pos) = config.playfield_pos {
            pos.x
        } else {
            screen_width / 2.0
        };
        let playfield_offset_y = if let Some(pos) = config.playfield_pos {
            pos.y
        } else {
            0.0
        };
        let x_offset = playfield_center_x - (screen_width / 2.0);

        pf.config.x_offset_pixels = x_offset;
        pf.config.y_offset_pixels = playfield_offset_y;

        // 2. Update HUD resources.
        let default_combo_y = (screen_height / 2.0) - 80.0;
        let default_score_x = playfield_center_x + (playfield_width_px / 2.0) + 120.0;
        let default_score_y = screen_height * 0.05;
        let default_acc_x = playfield_center_x - (playfield_width_px / 2.0) - 150.0;

        let score_pos = config.score_pos.unwrap_or(UIElementPos {
            x: default_score_x,
            y: default_score_y,
        });
        self.score_display.set_position(score_pos.x, score_pos.y);
        self.score_display.set_size(config.score_text_size);

        let combo_pos = config.combo_pos.unwrap_or(UIElementPos {
            x: playfield_center_x,
            y: default_combo_y,
        });
        self.combo_display.set_position(combo_pos.x, combo_pos.y);
        self.combo_display.set_size(config.combo_text_size);

        let acc_pos = config.accuracy_pos.unwrap_or(UIElementPos {
            x: default_acc_x,
            y: screen_height * 0.1,
        });
        self.accuracy_panel.set_position(acc_pos.x, acc_pos.y);
        self.accuracy_panel.set_size(config.accuracy_text_size);

        let judge_pos = config.judgement_pos.unwrap_or(UIElementPos {
            x: default_acc_x,
            y: screen_height * 0.15,
        });
        self.judgements_panel.set_position(judge_pos.x, judge_pos.y);
        self.judgements_panel.set_size(config.judgement_text_size);

        let hitbar_width = playfield_width_px * 0.8;
        let hitbar_pos = config.hit_bar_pos.unwrap_or(UIElementPos {
            x: playfield_center_x - hitbar_width / 2.0,
            y: combo_pos.y + 60.0,
        });
        self.hit_bar.set_geometry(
            hitbar_pos.x,
            hitbar_pos.y,
            hitbar_width,
            config.hit_bar_height_px,
        );

        let flash_pos = config.judgement_flash_pos.unwrap_or(UIElementPos {
            x: playfield_center_x,
            y: combo_pos.y + 30.0,
        });
        self.judgement_flash.set_position(flash_pos.x, flash_pos.y);
    }

    pub fn load_background(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, path_str: &str) {
        if let Some(current) = &self.current_background_path {
            if current == path_str {
                return;
            }
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
