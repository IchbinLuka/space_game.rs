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

#import bevy_render::view::View

#ifdef ENVIRONMENT_MAP
#import bevy_pbr::environment_map
#endif


// @group(0) @binding(0) var<uniform> view: View;

@group(1) @binding(0)
var<uniform> settings: OutlineMaterialUniform;
@group(1) @binding(1) var texture: texture_2d<f32>;
@group(1) @binding(2) var texture_sampler: sampler;

struct ShaderSettings {
    cross_scale: f32, 
    depth_threshold: f32,
    normal_threshold: f32,
    depth_normal_threshold_scale: f32,
    depth_normal_threshold: f32,
}

struct OutlineMaterialUniform {
    filter_scale: f32,
    depth_threshold: f32,
    normal_threshold: f32,
    depth_normal_threshold_scale: f32,
    depth_normal_threshold: f32,
    use_texture: u32,
    color: vec4<f32>, 
}

const QUANTIZE_STEPS: u32 = 3u;


// Mostly taken from bevy_pbr::pbr_functions::apply_pbr_lighting
fn shadow_multiplier(in: PbrInput) -> f32 {
    if (in.flags & MESH_FLAGS_SHADOW_RECEIVER_BIT) == 0u {
        return 1.0;
    }

    let view_z = dot(vec4<f32>(
        view.inverse_view[0].z,
        view.inverse_view[1].z,
        view.inverse_view[2].z,
        view.inverse_view[3].z
    ), in.world_position);
    let cluster_index = fragment_cluster_index(in.frag_coord.xy, view_z, in.is_orthographic);
    let offset_and_counts = unpack_offset_and_counts(cluster_index);

    let n_directional_lights = lights.n_directional_lights;
    var shadow: f32 = 1.0;
    var direct_light: vec3<f32> = vec3<f32>(0.0);
    for (var i: u32 = 0u; i < n_directional_lights; i = i + 1u) {
        let light = &lights.directional_lights[i];

        // if ((*light).render_layers & view.render_layers) == 0u {
        //     continue;
        // }
        shadow *= fetch_directional_shadow(i, in.world_position, in.world_normal, view_z);
        // if light_shadow < 0.8 {
        //     shadow *= 0.3;
        //     break;
        // }
    }
    return max(f32(u32(shadow * f32(QUANTIZE_STEPS))) / f32(QUANTIZE_STEPS), 0.2);
}


@fragment
fn fragment(
// #ifdef MULTISAMPLED
//     @builtin(sample_index) sample_index: u32,
// #endif
    in: VertexOutput,
    @builtin(front_facing) is_front: bool, 
) -> @location(0) vec4<f32> {
// #ifndef MULTISAMPLED
    let sample_index = 0u;
// #endif
    

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

    var pbr_input = pbr_input_from_vertex_output(in, is_front, true);

    // pbr_input.material.base_color = out_color;
    // pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    // out_color = apply_pbr_lighting(pbr_input);

    // out_color = vec4<f32>(vec4<u32>(out_color * f32(QUANTIZE_STEPS))) / f32(QUANTIZE_STEPS);

    return out_color * shadow_multiplier(pbr_input);
}