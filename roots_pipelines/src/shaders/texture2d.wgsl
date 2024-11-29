//====================================================================
// Uniforms

struct Camera {
    projection: mat4x4<f32>,
    position: vec3<f32>,
}

@group(0) @binding(0) var<uniform> camera: Camera;

@group(1) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;


//====================================================================

struct VertexIn {
    // Vertex
    @location(0) vertex_position: vec2<f32>,
    @location(1) uv: vec2<f32>,

    // Instance
    @location(2) color: vec4<f32>,
    @location(3) size: vec2<f32>,
    @location(4) position: vec3<f32>
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

//====================================================================

@vertex
fn vs_main(in: VertexIn) -> VertexOut {
    var out: VertexOut;
    
    let vertex_pos = 
        vec3<f32>(in.vertex_position * in.size, 0.) 
        + in.position;

    out.clip_position =
        camera.projection
        * vec4<f32>(vertex_pos, 1.);

    out.uv = in.uv;
    out.color = in.color;

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let tex_color = textureSample(texture, texture_sampler, in.uv);
    
    return tex_color * in.color;
}

//====================================================================
