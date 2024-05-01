#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

#import bevy_render::globals::Globals
#import bevy_pbr::mesh_view_bindings::globals


struct MaterialConfig {
    threshold_offset: f32, 
    speed: f32,
    inner_color: vec4<f32>,
    outer_color: vec4<f32>,
}

@group(2) @binding(0) var<uniform> material_config: MaterialConfig;
@group(2) @binding(1) var noise_texture: texture_2d<f32>;
@group(2) @binding(2) var noise_sampler: sampler;

const COLOR_1: vec4<f32> = vec4<f32>(1.0, 0.30, 0.1, 1.0);
const COLOR_2: vec4<f32> = vec4<f32>(0.988, 0.78, 0.1, 1.0);

fn gradient(y: f32) -> f32 {
    return clamp(2.0 * y, 0.0, 1.0);
}

const PI: f32 = 3.14159265359;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = vec2<f32>(in.uv.x, in.uv.y * 0.1 + globals.time % 1.0);
    // var alpha = textureSampleBias(noise_texture, noise_sampler, uv, view.mip_bias).z + in.uv.y;
    // alpha *= gradient(in.uv.y);

    // if alpha < 0.3 {
    //     discard;
    // } else if alpha < 0.5 {
    //     return COLOR_1;
    // } else {
    //     return COLOR_2;
    // }
    // return vec4<f32>(COLOR_1.xyz, clamp(alpha * gradient(in.uv.y), 0.0, 1.0));
    let threshold = 0.4 * textureSampleBias(noise_texture, noise_sampler, vec2<f32>(in.uv.x, globals.time % 1.0), view.mip_bias).x + 
        0.1 * sin(in.uv.x * 20.0 + globals.time * 25.0 * material_config.speed) + 
        0.2;

    if in.uv.y < threshold {
        discard;
    }
    if in.uv.y < threshold + material_config.threshold_offset {
        return material_config.outer_color;
    }
    return material_config.inner_color;
}