struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) @interpolate(flat) filled_color: vec4<f32>,
    @location(1) @interpolate(flat) empty_color: vec4<f32>,
    @location(2) @interpolate(flat) progress: f32,
    @location(3) @interpolate(flat) mode: u32, // 0 = Bar, 1 = Circle
    @location(4) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    @location(0) @interpolate(flat) center: vec2<f32>,
    @location(1) @interpolate(flat) size: vec2<f32>,
    @location(2) @interpolate(flat) filled_color: vec4<f32>,
    @location(3) @interpolate(flat) empty_color: vec4<f32>,
    @location(4) @interpolate(flat) progress: f32,
    @location(5) @interpolate(flat) mode: u32,
) -> VertexOutput {
    let u = f32(in_vertex_index % 2u);
    let v = f32(in_vertex_index / 2u);

    var output: VertexOutput;
    let corner_x = center.x + (u - 0.5) * size.x;
    let corner_y = center.y + (v - 0.5) * size.y;

    output.clip_position = vec4<f32>(corner_x, corner_y, 0.0, 1.0);
    output.filled_color = filled_color;
    output.empty_color = empty_color;
    output.progress = progress;
    output.mode = mode;
    output.uv = vec2<f32>(u, v); // 0..1

    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var alpha = 0.0;
    var use_filled = false;

    // Bar Mode
    if (in.mode == 0u) {
        if (in.uv.x <= in.progress) {
            use_filled = true;
        } else {
            use_filled = false;
        }
        alpha = 1.0;
    }
    // Circle Mode
    else if (in.mode == 1u) {
        let centered_uv = in.uv - vec2<f32>(0.5, 0.5);
        let dist = length(centered_uv);
        
        // Simple circle mask
        if (dist > 0.5) {
            discard;
        }

        // Calculate angle (0 at top (0,-1), clockwise)
        // atan2(y, x) returns angle in radians from -PI to PI
        // We want 0 at top (which is y=-0.5 in UV space if v goes down? No wgpu v goes down?)
        // Let's assume standard UV: 0,0 top-left, 1,1 bottom-right.
        // Center is 0.5, 0.5.
        // Top is 0.5, 0.0. Vector (0, -0.5).
        // Right is 1.0, 0.5. Vector (0.5, 0).
        // atan2(y, x)
        
        // Let's rotate coordinates so -PI/2 is top.
        // Angle in shader: atan2(y, x).
        // We want progress 0..1 mapping to angle 0..2PI starting from top.
        
        let angle_raw = atan2(centered_uv.y, centered_uv.x); 
        // With y down (UV 0 to 1), y positive is down.
        // x positive is right.
        // atan2(0.5, 0) = PI/2 (Bottom)
        // atan2(-0.5, 0) = -PI/2 (Top)
        // atan2(0, 0.5) = 0 (Right)
        // atan2(0, -0.5) = PI (Left)
        
        // We want Start at Top (-PI/2), go Clockwise.
        // Clockwise in screen space (y down) means angle increases?
        // Top (-PI/2) -> Right (0) -> Bottom (PI/2) -> Left (PI) -> Top/Left (-PI)
        
        // Normalize angle to 0..1 starting from Top Clockwise.
        // target_angle = (angle_raw + PI/2) / (2 * PI) ? 
        // Top: -PI/2 + PI/2 = 0.
        // Right: 0 + PI/2 = PI/2. -> 0.25 (Correct)
        // Bottom: PI/2 + PI/2 = PI. -> 0.5 (Correct)
        // Left: PI + PI/2 = 3PI/2. -> 0.75 (Correct)
        // But atan2 returns -PI..PI.
        // If angle_raw is -PI (-3.14), target should be close to 1.
        // (-PI + PI/2) = -PI/2 = -1.57. Negative?
        
        var angle = angle_raw + 1.57079632679; // + PI/2
        if (angle < 0.0) {
            angle = angle + 6.28318530718; // + 2PI
        }
        let angle_norm = angle / 6.28318530718;
        
        if (angle_norm <= in.progress) {
            use_filled = true;
        } else {
            use_filled = false;
        }
    }

    if (use_filled) {
        return in.filled_color;
    } else {
        return in.empty_color;
    }
}
