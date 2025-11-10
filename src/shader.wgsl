struct CameraUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}

@group(2) @binding(0)
var<uniform> light: Light;

struct InstanceInput {
    @location(5) mat_1: vec4<f32>,
    @location(6) mat_2: vec4<f32>,
    @location(7) mat_3: vec4<f32>,
    @location(8) mat_4: vec4<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>
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

    var world_position = model_matrix * vec4<f32>(model.position, 1.0);
    
    out.world_normal = (model_matrix * vec4<f32>(model.normal, 1.0)).xyz;
    out.clip_position = camera.view_proj * world_position;
    out.world_position = world_position.xyz;

    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    let ambient_strength = 0.1;
    let ambient_color_base = vec3<f32>(1.0, 1.0, 1.0);
    let ambient_color = ambient_strength * ambient_color_base;

    let view_position = (camera.view * vec4<f32>(in.world_position, 1.0)).xyz;
    let view_rotation = mat3x3<f32>(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz);
    let view_normal = normalize(view_rotation * in.world_normal);
    let light_pos_view = (camera.view * vec4<f32>(light.position, 1.0)).xyz;

    let light_dir = normalize(light_pos_view - view_position);
    let diffuse_strength = max(dot(view_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;
    let result = (ambient_color + diffuse_color) * object_color.xyz;

    return vec4<f32>(result, object_color.a);
}
 