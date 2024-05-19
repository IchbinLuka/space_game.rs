#import bevy_pbr::forward_io::VertexOutput

#import bevy_render::globals::Globals
#import bevy_pbr::mesh_view_bindings::globals

const COLOR_1: vec4<f32> = vec4<f32>(1.0, 0.0, 0.0, 1.0);
const COLOR_2: vec4<f32> = vec4<f32>(0.0, 1.0, 0.0, 1.0);

struct ShaderSettings {
    period: f32, 
    color_1: vec4<f32>,
    color_2: vec4<f32>,
}

@group(2) @binding(0) var<uniform> settings: ShaderSettings; 

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    if globals.time % settings.period > settings.period / 2.0 {
        return settings.color_1;
    } else {
        return settings.color_2;
    }
}