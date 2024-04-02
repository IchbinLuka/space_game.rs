#import bevy_pbr::{
    forward_io::VertexOutput,
    mesh_view_bindings::{
        globals, 
        view, 
        lights, 
    },
    mesh_types::{MESH_FLAGS_SHADOW_RECEIVER_BIT, MESH_FLAGS_TRANSMITTED_SHADOW_RECEIVER_BIT},
}

#import "shaders/toon_functions.wgsl"::toon_fragment
#import "shaders/toon_bindings.wgsl"::{
    settings, 
    texture, 
    texture_sampler
}

@group(2)
@binding(3)
var<uniform> center: vec4<f32>;

fn shadow_multiplier(in: VertexOutput) -> f32 {
    let n_directional_lights = lights.n_directional_lights;
    var shadow: f32 = 1.0;
    var direct_light: vec3<f32> = vec3<f32>(0.0);
    for (var i: u32 = 0u; i < n_directional_lights; i = i + 1u) {
        let light = &lights.directional_lights[i];

        let normal = in.world_position.xyz - center.xyz;

        let dot_prod = dot(normal, (*light).direction_to_light);

        
        if dot_prod < 0.0 {
            if dot_prod > -0.5 {
                shadow *= 0.7;
            } else {
                shadow *= 0.4;
            }
        }

        // if ((*light).render_layers & view.render_layers) == 0u {
        //     continue;
        // }
    }

    return shadow;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>((shadow_multiplier(in) * settings.color).xyz, settings.color.w) * textureSampleBias(texture, texture_sampler, in.uv, view.mip_bias);
}