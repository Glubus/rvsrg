use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct QuadInstance {
    pub center: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
}

pub fn quad_from_rect(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: [f32; 4],
    screen_width: f32,
    screen_height: f32,
) -> QuadInstance {
    let center = [
        ((x + width / 2.0) / screen_width) * 2.0 - 1.0,
        -(((y + height / 2.0) / screen_height) * 2.0 - 1.0),
    ];
    let size = [(width / screen_width) * 2.0, (height / screen_height) * 2.0];

    QuadInstance {
        center,
        size,
        color,
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ProgressInstance {
    pub center: [f32; 2],
    pub size: [f32; 2],
    pub filled_color: [f32; 4],
    pub empty_color: [f32; 4],
    pub progress: f32,
    pub mode: u32,
    pub padding: [f32; 2], // Pad to align to 16 bytes if needed, or just because stride
}

// 2 (center) + 2 (size) + 4 (filled) + 4 (empty) + 1 (progress) + 1 (mode) + 2 (padding) = 16 floats * 4 bytes = 64 bytes?
// center: 8 bytes
// size: 8 bytes
// filled: 16 bytes
// empty: 16 bytes
// progress: 4 bytes
// mode: 4 bytes
// padding: 8 bytes
// Total: 64 bytes. Good.

pub fn progress_from_rect(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    filled_color: [f32; 4],
    empty_color: [f32; 4],
    progress: f32,
    mode: u32,
    screen_width: f32,
    screen_height: f32,
) -> ProgressInstance {
    let center = [
        ((x + width / 2.0) / screen_width) * 2.0 - 1.0,
        -(((y + height / 2.0) / screen_height) * 2.0 - 1.0),
    ];
    let size = [(width / screen_width) * 2.0, (height / screen_height) * 2.0];

    ProgressInstance {
        center,
        size,
        filled_color,
        empty_color,
        progress,
        mode,
        padding: [0.0, 0.0],
    }
}
