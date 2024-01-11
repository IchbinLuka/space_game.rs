#import bevy_pbr::forward_io::VertexOutput


struct Material {
    color: vec4<f32>,
    // radius: f32,
};

@group(1)
@binding(0)
var<uniform> material: Material;

// struct Uniforms {
//     radius: f32,
//     center: vec3<f32>,
//     color: vec4<f32>,
// };

// struct VertexIn {
//     @location(0) position: vec3<f32>, 
// };

// struct VertexOut {
//     @builtin(position) position: vec4<f32>, 
// };

// @vertex
// fn vs(in: VertexIn) -> VertexOut {
//     // Get camera matrix
//     let camera_matrix = mesh.model;

//     let camera_right = vec3<f32>(uni.view_matrx[0][0], uni.view_matrx[1][0], uni.view_matrx[2][0]);
//     let camera_up = vec3<f32>(uni.view_matrx[0][1], uni.view_matrx[1][1], uni.view_matrx[2][1]);

//     return VertexOut(vec4<f32>(uni.center + in.position.x * camera_right + in.position.y * camera_up, 1.0));
// }

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let radius = 2.5;
    let delta = in.world_position.xyz;
    let dist_squared = dot(delta, delta);
    if dist_squared > radius * radius {
        discard;
    }
    return material.color;
}