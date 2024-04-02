#import bevy_pbr::forward_io::VertexOutput
#import bevy_render::view::View

struct Material {
    color: vec4<f32>,
};


@group(0) @binding(0) var<uniform> view: View;

@group(2)
@binding(0)
var<uniform> material: Material;


const RADIUS_SQUARED: f32 = 0.25;
const SHARPNESS: f32 = 1000.0;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixel_size: vec2<f32> = vec2<f32>(2.0) / view.viewport.zw;
    
    
    let delta = in.uv - vec2<f32>(0.5, 0.5);
    let dist_squared = dot(delta, delta);

    let difference = RADIUS_SQUARED - dist_squared;
    
    if difference < 0.0 {
        discard;
    }

    let alpha = min(1.0, difference * SHARPNESS);

    return vec4<f32>(material.color.xyz, alpha);
}