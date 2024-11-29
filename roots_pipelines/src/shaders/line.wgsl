//====================================================================
// Uniforms

struct Camera {
    projection: mat4x4<f32>,
    position: vec3<f32>,
}

@group(0) @binding(0) var<uniform> camera: Camera;

//====================================================================

struct VertexIn {
    // Vertex
    @builtin(vertex_index) index: u32,
    @location(0) vertex_position: vec3<f32>

    // Instance
    @location(1) color: vec4<f32>,
    @location(2) pos1: vec3<f32>,
    @location(3) pos2: vec3<f32>,
    @location(4) thickness: f32,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

//====================================================================

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;

    if in.index < 4 {
        out.clip_position = in.vertex_position * in.thickness + in.pos1;
    }
    else {
        out.clip_position = in.vertex_position * in.thickness + in.pos2;
    }

    out.color = in.color;
}

//====================================================================

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return in.color;
}

//====================================================================

