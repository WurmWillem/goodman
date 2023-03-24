
struct CameraUniform {
    pos: vec2<f32>,
};

@group(1) @binding(0) 
var<uniform> camera: CameraUniform;

struct InstanceInput {
    @location(2) data: vec3<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;

    let updated_pos = vec2<f32>(model.position.x + camera.pos.x + instance.data.x, model.position.y + camera.pos.y + instance.data.y);
    out.clip_position = vec4<f32>(updated_pos, model.position.z + instance.data.z, 1.0);

    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}