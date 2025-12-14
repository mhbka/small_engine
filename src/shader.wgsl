struct CameraUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    view_position: vec3<f32>
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct PointLight {
    position: vec3<f32>,
    color: vec3<f32>,
}

@group(2) @binding(0)
var<storage, read> point_lights: array<PointLight>; 

@group(2) @binding(1)
var<uniform> point_light_count: u32;

struct InstanceInput {
    @location(5) mat_1: vec4<f32>,
    @location(6) mat_2: vec4<f32>,
    @location(7) mat_3: vec4<f32>,
    @location(8) mat_4: vec4<f32>,
    @location(9) mat_5: vec3<f32>,
    @location(10) mat_6: vec3<f32>,
    @location(11) mat_7: vec3<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) world_normal: vec3<f32>,
    @location(3) world_tangent: vec3<f32>,
    @location(4) world_bitangent: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput
) -> VertexOutput {

    let model_matrix = mat4x4<f32>(
        instance.mat_1,
        instance.mat_2,
        instance.mat_3,
        instance.mat_4
    );
    let normal_matrix = mat3x3<f32>(
        instance.mat_5,
        instance.mat_6,
        instance.mat_7
    );

    let world_position = model_matrix * vec4<f32>(model.position, 1.0);
    
    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_position;
    out.tex_coords = model.tex_coords;
    out.world_position = world_position.xyz;
    out.world_normal = normalize(normal_matrix * model.normal);
    out.world_tangent = normalize(normal_matrix * model.tangent);
    out.world_bitangent = normalize(normal_matrix * model.bitangent);

    return out;
}

@group(0) @binding(0)
var diffuse_texture: texture_2d<f32>;

@group(0) @binding(1)
var diffuse_sampler: sampler;

@group(0) @binding(2)
var normal_texture: texture_2d<f32>;

@group(0) @binding(3)
var normal_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Reconstruct orthonormal TBN matrix from interpolated vectors
    let t_vector = normalize(in.world_tangent);
    let b_vector = normalize(in.world_bitangent);
    let n_vector = normalize(in.world_normal);
    let tangent_matrix = transpose(mat3x3<f32>(t_vector, b_vector, n_vector));

    let object_color = textureSample(diffuse_texture, diffuse_sampler, in.tex_coords);
    let object_normal = textureSample(normal_texture, normal_sampler, in.tex_coords);
    
    // Transform positions to tangent space
    let tangent_position = tangent_matrix * in.world_position;
    let tangent_view_position = tangent_matrix * camera.view_position;
    
    // Ambient lighting
    let ambient_strength = 0.0;
    let ambient_color_base = vec3<f32>(1.0, 1.0, 1.0);
    let ambient_color = ambient_strength * ambient_color_base;
    var result = ambient_color * object_color.xyz;

    // Get tangent-space normal from normal map
    let tangent_normal = normalize(object_normal.xyz * 2.0 - 1.0);

    // Calculate lighting for each point light
    for (var i = 0u; i < point_light_count; i++) {
        let light = point_lights[i];
        
        // Transform light position to tangent space
        let tangent_light_position = tangent_matrix * light.position;
        
        let light_dir = normalize(tangent_light_position - tangent_position);
        let view_dir = normalize(tangent_view_position - tangent_position);
        let half_dir = normalize(view_dir + light_dir);

        // Diffuse
        let diffuse_strength = max(dot(tangent_normal, light_dir), 0.0);
        let diffuse_color = light.color * diffuse_strength;
            
        // Specular
        let spec_strength = pow(max(dot(tangent_normal, half_dir), 0.0), 64.0);
        let spec_color = light.color * spec_strength;

        result += (spec_color) * object_color.xyz;
    }

    return vec4<f32>(result, object_color.a);
}