use crate::models::engine::InstanceRaw; // Assurez-vous que ce modèle est accessible via models
use crate::shaders::constants::MAIN_SHADER_SRC;
use std::path::Path;
use wgpu::{BindGroupLayout, Device, Queue, RenderPipeline, Sampler, Texture, TextureFormat};
use wgpu_text::{BrushBuilder, TextBrush};

// --- GESTION DES TEXTURES ---

pub fn load_texture_from_path(
    device: &Device,
    queue: &Queue,
    path: &Path,
) -> Option<(Texture, u32, u32)> {
    let img = match image::open(path) {
        Ok(i) => i,
        Err(e) => {
            log::warn!("Failed to load texture {:?}: {}", path, e);
            return None;
        }
    };

    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    let texture_size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: path.to_str(),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        texture_size,
    );

    Some((texture, width, height))
}

pub fn create_default_texture(
    device: &Device,
    queue: &Queue,
    color: [u8; 4],
    label: &str,
) -> Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &color,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4),
            rows_per_image: Some(1),
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );

    texture
}

// --- GESTION DES PIPELINES ---

pub fn create_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

pub fn create_sampler(device: &Device) -> Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}

pub fn create_render_pipeline(
    device: &Device,
    bind_group_layout: &BindGroupLayout,
    format: TextureFormat,
) -> RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(MAIN_SHADER_SRC)),
    });

    let instance_desc = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 5,
                format: wgpu::VertexFormat::Float32x2,
            }, // Offset
            wgpu::VertexAttribute {
                offset: 8,
                shader_location: 6,
                format: wgpu::VertexFormat::Float32x2,
            }, // Scale
        ],
    };

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[instance_desc],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
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
    })
}

// --- GESTION DU TEXTE ---

pub fn load_text_brush(
    device: &Device,
    width: u32,
    height: u32,
    format: TextureFormat,
    font_path: Option<&Path>,
) -> TextBrush {
    use wgpu_text::glyph_brush::ab_glyph::FontArc;

    let font = if let Some(path) = font_path {
        match std::fs::read(path) {
            Ok(data) => FontArc::try_from_vec(data).ok(),
            Err(_) => None,
        }
    } else {
        None
    };

    // Fallback si la police n'est pas trouvée
    let final_font = font.unwrap_or_else(|| {
        log::warn!(
            "Font not found or failed to load, using default (Hack: Empty Font might panic if used)"
        );
        // Pour éviter le panic, idéalement on devrait avoir une police par défaut incluse en bytes (include_bytes!)
        // Ici on retourne une police vide qui risque de ne rien afficher, mais évite le crash immédiat si géré par wgpu_text
        FontArc::try_from_vec(vec![]).unwrap_or_else(|_| panic!("Fatal: No font available"))
    });

    BrushBuilder::using_font(final_font).build(device, width, height, format)
}

