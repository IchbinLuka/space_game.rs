#import bevy_pbr::{
    mesh_view_bindings::globals,
    prepass_utils::{
        prepass_depth, 
        prepass_normal
    },
    forward_io::VertexOutput,
}
#import bevy_render::view::View


@group(0) @binding(0) var<uniform> view: View;

@group(1) @binding(0)
var<uniform> color: vec4<f32>;
@group(1) @binding(1)
var<uniform> settings: ShaderSettings;

struct ShaderSettings {
    cross_scale: f32, 
    depth_threshold: f32,
    normal_threshold: f32,
    depth_normal_threshold_scale: f32,
    depth_normal_threshold: f32,
}

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

    

    let half_scale_floor = floor(settings.cross_scale * 0.5);
    let half_scale_ceil = ceil(settings.cross_scale * 0.5);

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

    let clip_pos = vec4<f32>(mesh.position.x / view.viewport.z, mesh.position.y / view.viewport.w, 0.0, 1.0) * 2.0 - 1.0;

    let view_space_dir = mesh.world_position.xyz - view.world_position;

    let view_normal = view.view_proj * vec4<f32>(normal0, 0.0);


    let n_dot_v = 1.0 - dot(normal0, -1.0 * normalize(view_space_dir));

    let normal_threshold_1 = saturate((n_dot_v - settings.depth_normal_threshold) / (1.0 - settings.depth_normal_threshold));
    let normal_threshold = normal_threshold_1 * settings.depth_normal_threshold_scale + 1.0;

    let depth_threshold = settings.depth_threshold * depth0 * normal_threshold;

    let edge_normal = dot(normal_diff0, normal_diff0) + dot(normal_diff1, normal_diff1) > settings.normal_threshold * settings.normal_threshold;

    let edge_depth = pow(depth1 - depth0, 2.0) + pow(depth3 - depth2, 2.0) > depth_threshold * depth_threshold;

    if edge_normal || edge_depth {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }

    return color;
}