#import bevy_pbr::{
    mesh_view_bindings::globals,
    prepass_utils::{
        prepass_depth, 
        prepass_normal
    },
    forward_io::VertexOutput,
}
@group(1) @binding(0)
var<uniform> color: vec4<f32>;

const SCALE: f32 = 5.0;

@fragment
fn fragment(
// #ifdef MULTISAMPLED
//     @builtin(sample_index) sample_index: u32,
// #endif
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
// #ifndef MULTISAMPLED
    let sample_index = 0u;
// #endif

    let half_scale_floor = floor(SCALE * 0.5);
    let half_scale_ceil = ceil(SCALE * 0.5);

    let bottom_left = vec4<f32>(mesh.position.x - half_scale_floor, mesh.position.y - half_scale_floor, mesh.position.z, 1.0);
    let bottom_right = vec4<f32>(mesh.position.x + half_scale_ceil, mesh.position.y - half_scale_floor, mesh.position.z, 1.0);
    let top_left = vec4<f32>(mesh.position.x - half_scale_floor, mesh.position.y + half_scale_ceil, mesh.position.z, 1.0);
    let top_right = vec4<f32>(mesh.position.x + half_scale_ceil, mesh.position.y + half_scale_ceil, mesh.position.z, 1.0);

    let depth0 = prepass_depth(bottom_left, sample_index);
    let depth1 = prepass_depth(top_right, sample_index);
    let depth2 = prepass_depth(bottom_right, sample_index);
    let depth3 = prepass_depth(top_left, sample_index);

    let normal0 = prepass_normal(bottom_left, sample_index);
    let normal1 = prepass_normal(top_right, sample_index);
    let normal2 = prepass_normal(bottom_right, sample_index);
    let normal3 = prepass_normal(top_left, sample_index);

    let normal_diff0 = normal1 - normal0;
    let normal_diff1 = normal3 - normal2;

    let edge_normal = sqrt(dot(normal_diff0, normal_diff0) + dot(normal_diff1, normal_diff1));

    let edge_depth = sqrt(pow(depth1 - depth0, 2.0) + pow(depth3 - depth2, 2.0)) * 100.0;
    
    if max(edge_normal, edge_depth) > 0.8 {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }

    return color;

}