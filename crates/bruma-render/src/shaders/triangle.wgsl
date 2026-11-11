// Phase 0: a single triangle, no vertex buffers.
// Vertices come from the vertex index and the color "breathes" over time.

struct Globals {
    time: f32,
    aspect: f32,   // canvas width / height, so the triangle is not stretched
    _pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOut {
    // A centered equilateral triangle.
    var corners = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.62),
        vec2<f32>(-0.54, -0.31),
        vec2<f32>(0.54, -0.31),
    );
    // Solarpunk palette: green, gold and a clean-sky blue.
    var colors = array<vec3<f32>, 3>(
        vec3<f32>(0.35, 0.78, 0.48),
        vec3<f32>(0.95, 0.76, 0.30),
        vec3<f32>(0.36, 0.62, 0.92),
    );

    var p = corners[index];
    // Correct the aspect ratio so the triangle does not stretch on wide screens.
    if (globals.aspect > 1.0) {
        p.x = p.x / globals.aspect;
    } else {
        p.y = p.y * globals.aspect;
    }

    var out: VertexOut;
    out.position = vec4<f32>(p, 0.0, 1.0);
    out.color = colors[index];
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    // A slow pulse (~6 s per cycle), like drifting haze.
    let breath = 0.85 + 0.15 * sin(globals.time * 1.05);
    return vec4<f32>(in.color * breath, 1.0);
}
