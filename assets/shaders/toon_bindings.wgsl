#import "shaders/toon_types.wgsl"::OutlineMaterialUniform

@group(1) @binding(0)
var<uniform> settings: OutlineMaterialUniform;
@group(1) @binding(1) var texture: texture_2d<f32>;
@group(1) @binding(2) var texture_sampler: sampler;