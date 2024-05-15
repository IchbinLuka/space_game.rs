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

#import "shaders/toon_types.wgsl"::OutlineMaterialUniform;

#import "shaders/toon_functions.wgsl"::toon_fragment;

#ifdef ENVIRONMENT_MAP
#import bevy_pbr::environment_map
#endif


// @group(0) @binding(0) var<uniform> view: View;

struct ShaderSettings {
    cross_scale: f32, 
    depth_threshold: f32,
    normal_threshold: f32,
    depth_normal_threshold_scale: f32,
    depth_normal_threshold: f32,
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
    }
    return max(f32(u32(shadow * f32(QUANTIZE_STEPS))) / f32(QUANTIZE_STEPS), 0.2);
}


@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool, 
) -> @location(0) vec4<f32> {
    let sample_index = 0u;
    var pbr_input = pbr_input_from_vertex_output(in, is_front, true);
    let toon_color = toon_fragment(in, sample_index);
    return vec4<f32>((toon_color.xyz * shadow_multiplier(pbr_input)), toon_color.w);
}