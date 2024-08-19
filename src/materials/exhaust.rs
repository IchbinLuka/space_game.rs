use bevy::{
    color::palettes::css, prelude::*, render::{
        render_asset::RenderAssets,
        render_resource::{AsBindGroup, AsBindGroupShaderType, ShaderRef, ShaderType},
        texture::GpuImage,
    }
};
use bevy_asset_loader::asset_collection::{AssetCollection, AssetCollectionApp};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[uniform(0, ExhaustMaterialUniform)]
pub struct ExhaustMaterial {
    #[texture(1)]
    #[sampler(2)]
    #[dependency]
    pub noise_texture: Handle<Image>,
    pub threshold_offset: f32,
    pub speed: f32,
    pub inner_color: Color,
    pub outer_color: Color,
}

impl ExhaustMaterial {
    pub fn _new(inner_color: Color, outer_color: Color) -> Self {
        Self {
            inner_color,
            outer_color,
            ..default()
        }
    }
}

impl Default for ExhaustMaterial {
    fn default() -> Self {
        Self {
            inner_color: css::ORANGE.into(),
            outer_color: css::ORANGE_RED.into(),
            threshold_offset: 0.3,
            speed: 1.0,
            noise_texture: Default::default(),
        }
    }
}

#[derive(Debug, Clone, AsBindGroup, ShaderType)]
pub struct ExhaustMaterialUniform {
    threshold_offset: f32,
    speed: f32,
    inner_color: LinearRgba,
    outer_color: LinearRgba,
}

impl AsBindGroupShaderType<ExhaustMaterialUniform> for ExhaustMaterial {
    fn as_bind_group_shader_type(
        &self,
        _images: &RenderAssets<GpuImage>,
    ) -> ExhaustMaterialUniform {
        ExhaustMaterialUniform {
            inner_color: self.inner_color.into(),
            outer_color: self.outer_color.into(),
            threshold_offset: self.threshold_offset,
            speed: self.speed,
        }
    }
}

impl Material for ExhaustMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/exhaust.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

fn inject_noise_texture(mut materials: ResMut<Assets<ExhaustMaterial>>, res: Res<ExhaustRes>) {
    let inner = materials.bypass_change_detection();
    for (_, material) in inner.iter_mut() {
        material.noise_texture = res.noise_texture.clone();
    }
}

#[derive(Resource, AssetCollection)]
pub struct ExhaustRes {
    #[asset(path = "textures/noise.png")]
    pub noise_texture: Handle<Image>,
    #[asset(path = "exhaust.obj")]
    pub mesh: Handle<Mesh>,
}

pub struct ExhaustPlugin;
impl Plugin for ExhaustPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ExhaustMaterial>::default())
            .init_collection::<ExhaustRes>()
            .add_systems(
                Update,
                inject_noise_texture.run_if(resource_changed::<Assets<ExhaustMaterial>>),
            );
    }
}
