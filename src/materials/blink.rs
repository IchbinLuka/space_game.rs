use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{AsBindGroup, AsBindGroupShaderType, ShaderRef, ShaderType},
        texture::GpuImage,
    },
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[uniform(0, BlinkMaterialUniform)]
pub struct BlinkMaterial {
    pub period: f32,
    pub color_1: Color,
    pub color_2: Color,
}

#[derive(Debug, Clone, AsBindGroup, ShaderType)]
struct BlinkMaterialUniform {
    pub period: f32,
    pub color_1: LinearRgba,
    pub color_2: LinearRgba,
}

impl AsBindGroupShaderType<BlinkMaterialUniform> for BlinkMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<GpuImage>) -> BlinkMaterialUniform {
        BlinkMaterialUniform {
            period: self.period,
            color_1: self.color_1.into(),
            color_2: self.color_2.into(),
        }
    }
}

impl Material for BlinkMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/blink_material.wgsl".into()
    }
}

pub struct BlinkMaterialPlugin;
impl Plugin for BlinkMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<BlinkMaterial>::default());
    }
}
