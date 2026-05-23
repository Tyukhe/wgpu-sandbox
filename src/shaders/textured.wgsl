struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct Light {
    position: vec4<f32>,
    color: vec4<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) idx: u32,
}

struct InstanceData {
    matrix: mat4x4<f32>,
    texture_id: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<storage, read> data: array<InstanceData>;
@group(2) @binding(0) var<storage, read> lights: array<Light>;
@group(3) @binding(0) var t_diffuse: binding_array<texture_2d<f32>>;
@group(3) @binding(1) var s_diffuse: sampler;

@vertex
fn vs_main(model: VertexInput, @builtin(instance_index) idx: u32) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * data[idx].matrix * vec4<f32>(model.position, 1.0);
    out.position = model.position;
    out.tex_coords = model.tex_coords;
    out.normal = (data[idx].matrix * vec4<f32>(model.normal, 1.0)).xyz;
    out.idx = idx;
    return out;
}

fn calculate_light(light: Light, surface_pos: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    let light_vec = light.position.xyz - (surface_pos * light.position.w);
    let dist = length(light_vec);
    let light_dir = light_vec / (dist + 0.00001);
    let dot_product = max(dot(normal, light_dir), 0.0);
    let base_color = light.color.rgb * light.color.a * dot_product;
    let point_attenuation = 1.0 / (1.0 + 0.01 * dist + 0.001 * dist * dist);
    let attenuation = mix(1.0, point_attenuation, light.position.w);

    return base_color * attenuation;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let basic_color = textureSample(t_diffuse[data[in.idx].texture_id], s_diffuse, in.tex_coords);
    var result_color = vec3<f32>(0.0);
    for (var i = 0u; i < arrayLength(&lights); i++) {
        result_color += basic_color.xyz * calculate_light(lights[i], in.position, in.normal);
    }
    return vec4<f32>(result_color, 1.0) * 1.2;
}
