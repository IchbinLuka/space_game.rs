#import bevy_pbr::{
    mesh_view_bindings::{
        globals, 
        view, 
        lights, 
    },
    lighting::point_light, 
    pbr_fragment::pbr_input_from_vertex_output,
    pbr_functions::{
        alpha_discard, 
    },
    prepass_utils::{
        prepass_depth, 
        prepass_normal
    },
    forward_io::VertexOutput,
    clustered_forward::{
        fragment_cluster_index,
        unpack_offset_and_counts
    }, 
    shadows::fetch_directional_shadow,
    pbr_types::PbrInput,
    mesh_types::{MESH_FLAGS_SHADOW_RECEIVER_BIT, MESH_FLAGS_TRANSMITTED_SHADOW_RECEIVER_BIT},
}

#import "shaders/toon_types.wgsl"::OutlineMaterialUniform

#ifdef ENVIRONMENT_MAP
#import bevy_pbr::environment_map
#endif

#import "shaders/toon_bindings.wgsl"::{
    settings, 
    texture,
    texture_sampler
}

#ifndef WEBGL2
fn toon_outline(position: vec3<f32>, world_pos: vec3<f32>) -> bool {
    let sample_index = 0u;

    let half_scale_floor = floor(settings.filter_scale * 0.5);
    let half_scale_ceil = ceil(settings.filter_scale * 0.5);

    let bottom_left = vec4<f32>(position.x - half_scale_floor, position.y - half_scale_floor, position.z, 1.0);
    let bottom_right = vec4<f32>(position.x + half_scale_ceil, position.y - half_scale_floor, position.z, 1.0);
    let top_left = vec4<f32>(position.x - half_scale_floor, position.y + half_scale_ceil, position.z, 1.0);
    let top_right = vec4<f32>(position.x + half_scale_ceil, position.y + half_scale_ceil, position.z, 1.0);

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

    let clip_pos = vec4<f32>(position.x / view.viewport.z, position.y / view.viewport.w, 0.0, 1.0) * 2.0 - 1.0;

    let view_space_dir = world_pos - view.world_position;

    let view_normal = view.view_proj * vec4<f32>(normal0, 0.0);


    let n_dot_v = 1.0 - dot(normal0, -1.0 * normalize(view_space_dir));

    let normal_threshold_1 = saturate((n_dot_v - settings.depth_normal_threshold) / (1.0 - settings.depth_normal_threshold));
    let normal_threshold = normal_threshold_1 * settings.depth_normal_threshold_scale + 1.0;

    let depth_threshold = settings.depth_threshold * depth0 * normal_threshold;

    let edge_normal = dot(normal_diff0, normal_diff0) + dot(normal_diff1, normal_diff1) > settings.normal_threshold * settings.normal_threshold;

    let edge_depth = pow(depth1 - depth0, 2.0) + pow(depth3 - depth2, 2.0) > depth_threshold * depth_threshold;

    return edge_normal || edge_depth;
}
#endif


fn toon_fragment(in: VertexOutput) -> vec4<f32> {
    var out_color: vec4<f32> = settings.color;

    if settings.use_texture != 0u {
        out_color *= textureSampleBias(texture, texture_sampler, in.uv, view.mip_bias);
    }

    #ifndef WEBGL2
    if toon_outline(in.position.xyz, in.world_position.xyz) {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }
    #endif

    return out_color;
}

fn toon_fragment_multisampled(in: VertexOutput, subsample_radius: f32) -> vec4<f32> {
    var subsample_offsets = array<vec2<f32>, 7>(
        vec2<f32>(-subsample_radius, -subsample_radius),
        vec2<f32>(subsample_radius, -subsample_radius),
        vec2<f32>(-subsample_radius, subsample_radius),
        vec2<f32>(subsample_radius, subsample_radius), 
        vec2<f32>(0.0, 0.0), 
        vec2<f32>(0.0, 0.0), 
        vec2<f32>(0.0, 0.0), 
    );

    let sample_count = 7u;
    var outline_count: u32 = 0u;
    
    #ifndef WEBGL2
    for (var i: u32 = 0u; i < sample_count; i += 1u) {
        let offset = subsample_offsets[i];
        if toon_outline(in.position.xyz + vec3<f32>(offset, 0.0), in.world_position.xyz) {
            outline_count += 1u;
        }
    }
    #endif

    var out_color: vec4<f32> = settings.color;

    if settings.use_texture != 0u {
        out_color *= textureSampleBias(texture, texture_sampler, in.uv, view.mip_bias);
    }

    let outline_value = f32(outline_count) / f32(sample_count);

    return vec4<f32>(0.0, 0.0, 0.0, 1.0) * outline_value + out_color * (1.0 - outline_value);

/*

    let sample_index = 0u;

    let half_scale_floor = floor(settings.filter_scale * 0.5);
    let half_scale_ceil = ceil(settings.filter_scale * 0.5);

    let bottom_left = vec4<f32>(in.position.x - half_scale_floor, in.position.y - half_scale_floor, in.position.z, 1.0);
    let bottom_right = vec4<f32>(in.position.x + half_scale_ceil, in.position.y - half_scale_floor, in.position.z, 1.0);
    let top_left = vec4<f32>(in.position.x - half_scale_floor, in.position.y + half_scale_ceil, in.position.z, 1.0);
    let top_right = vec4<f32>(in.position.x + half_scale_ceil, in.position.y + half_scale_ceil, in.position.z, 1.0);

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

    let clip_pos = vec4<f32>(in.position.x / view.viewport.z, in.position.y / view.viewport.w, 0.0, 1.0) * 2.0 - 1.0;

    let view_space_dir = in.world_position.xyz - view.world_position;

    let view_normal = view.view_proj * vec4<f32>(normal0, 0.0);


    let n_dot_v = 1.0 - dot(normal0, -1.0 * normalize(view_space_dir));

    let normal_threshold_1 = saturate((n_dot_v - settings.depth_normal_threshold) / (1.0 - settings.depth_normal_threshold));
    let normal_threshold = normal_threshold_1 * settings.depth_normal_threshold_scale + 1.0;

    let depth_threshold = settings.depth_threshold * depth0 * normal_threshold;

    let edge_normal = dot(normal_diff0, normal_diff0) + dot(normal_diff1, normal_diff1) > settings.normal_threshold * settings.normal_threshold;

    let edge_depth = pow(depth1 - depth0, 2.0) + pow(depth3 - depth2, 2.0) > depth_threshold * depth_threshold;

    var out_color: vec4<f32> = settings.color;

    if settings.use_texture != 0u {
        out_color *= textureSampleBias(texture, texture_sampler, in.uv, view.mip_bias);
    }

    if edge_normal || edge_depth {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }

    return out_color;*/
}