struct CameraUniform {
    view_proj: mat4x4<f32>
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct InstanceInput {
    @location(5) mat_1: vec4<f32>,
    @location(6) mat_2: vec4<f32>,
    @location(7) mat_3: vec4<f32>,
    @location(8) mat_4: vec4<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    var out: VertexOutput;

    let model_matrix = mat4x4<f32>(
        instance.mat_1,
        instance.mat_2,
        instance.mat_3,
        instance.mat_4
    );

    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);

    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
 