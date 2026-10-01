// Fase 0: un triángulo, sin búferes de vértices.
// Los vértices salen del índice (vertex_index) y el color "respira" con el tiempo.

struct Globals {
    time: f32,
    aspect: f32,   // ancho / alto del canvas, para no deformar el triángulo
    _pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOut {
    // Un triángulo equilátero centrado.
    var corners = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.62),
        vec2<f32>(-0.54, -0.31),
        vec2<f32>(0.54, -0.31),
    );
    // Paleta solarpunk: verde, dorado y un azul de cielo limpio.
    var colors = array<vec3<f32>, 3>(
        vec3<f32>(0.35, 0.78, 0.48),
        vec3<f32>(0.95, 0.76, 0.30),
        vec3<f32>(0.36, 0.62, 0.92),
    );

    var p = corners[index];
    // Corrige la proporción para que el triángulo no se estire en pantallas anchas.
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
    // Una pulsación lenta (~6 s por ciclo), como bruma que se mueve.
    let breath = 0.85 + 0.15 * sin(globals.time * 1.05);
    return vec4<f32>(in.color * breath, 1.0);
}
