#import bevy_pbr::{
    forward_io::VertexOutput,
    mesh_view_bindings::view, 
}

@group(2) @binding(0) var<uniform> color: vec4<f32>;

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let view_direction = normalize(input.world_position.xyz - view.world_position);
    return vec4<f32>(color.xyz, max(1.0 + 2.0 * dot(view_direction, input.world_normal), 0.05));
}