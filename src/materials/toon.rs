use bevy::{
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_asset::RenderAssets,
        render_resource::{AsBindGroup, AsBindGroupShaderType, Face, ShaderRef, ShaderType},
        texture::GpuImage,
    },
    scene::SceneInstance,
};

use crate::utils::scene::MaterialBuilder;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[uniform(0, ToonMaterialUniform)]
pub struct PlanetMaterial {
    pub color: Color,

    #[uniform(3)]
    pub center: Vec4,

    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}

impl AsBindGroupShaderType<ToonMaterialUniform> for PlanetMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<GpuImage>) -> ToonMaterialUniform {
        ToonMaterialUniform {
            color: self.color.into(),
            use_texture: 1,
            filter_scale: 5.,
            ..default()
        }
    }
}

impl Material for PlanetMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/planet.wgsl".into()
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
#[reflect(Default, Debug)]
#[bind_group_data(ToonMaterialKey)]
#[uniform(0, ToonMaterialUniform)]
pub struct ToonMaterial {
    pub color: Color,

    #[texture(1)]
    #[sampler(2)]
    #[dependency]
    pub texture: Option<Handle<Image>>,

    pub disable_outline: bool,

    pub filter_scale: f32,
    pub depth_threshold: f32,
    pub normal_threshold: f32,
    pub depth_normal_threshold_scale: f32,
    pub depth_normal_threshold: f32,

    #[reflect(ignore)]
    pub cull_mode: Option<Face>,
}

#[derive(Debug, Clone, AsBindGroup, ShaderType)]
struct ToonMaterialUniform {
    filter_scale: f32,
    depth_threshold: f32,
    normal_threshold: f32,
    depth_normal_threshold_scale: f32,
    depth_normal_threshold: f32,
    use_texture: u32,
    color: LinearRgba,
}

impl AsBindGroupShaderType<ToonMaterialUniform> for ToonMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<GpuImage>) -> ToonMaterialUniform {
        ToonMaterialUniform {
            filter_scale: self.filter_scale,
            depth_threshold: self.depth_threshold,
            normal_threshold: self.normal_threshold,
            depth_normal_threshold_scale: self.depth_normal_threshold_scale,
            depth_normal_threshold: self.depth_normal_threshold,
            use_texture: if self.texture.is_some() { 1 } else { 0 },
            color: self.color.into(),
        }
    }
}

const DEFAULT_COLOR: Color = Color::WHITE;
const DEFAULT_FILTER_SCALE: f32 = 5.0;
const DEFAULT_DEPTH_THRESHOLD: f32 = 0.01;
const DEFAULT_NORMAL_THRESHOLD: f32 = 0.8;
const DEFAULT_DEPTH_NORMAL_THRESHOLD_SCALE: f32 = 20.0;
const DEFAULT_DEPTH_NORMAL_THRESHOLD: f32 = 0.5;

impl Default for ToonMaterial {
    fn default() -> Self {
        Self {
            color: DEFAULT_COLOR,
            texture: None,
            filter_scale: DEFAULT_FILTER_SCALE,
            depth_threshold: DEFAULT_DEPTH_THRESHOLD,
            normal_threshold: DEFAULT_NORMAL_THRESHOLD,
            depth_normal_threshold_scale: DEFAULT_DEPTH_NORMAL_THRESHOLD_SCALE,
            depth_normal_threshold: DEFAULT_DEPTH_NORMAL_THRESHOLD,
            cull_mode: None,
            disable_outline: false,
        }
    }
}

impl Default for ToonMaterialUniform {
    fn default() -> Self {
        Self {
            filter_scale: DEFAULT_FILTER_SCALE,
            depth_threshold: DEFAULT_DEPTH_THRESHOLD,
            normal_threshold: DEFAULT_NORMAL_THRESHOLD,
            depth_normal_threshold_scale: DEFAULT_DEPTH_NORMAL_THRESHOLD_SCALE,
            depth_normal_threshold: DEFAULT_DEPTH_NORMAL_THRESHOLD,
            use_texture: 0,
            color: DEFAULT_COLOR.into(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ToonMaterialKey {
    cull_mode: Option<Face>,
    disable_outline: bool,
}

impl From<&ToonMaterial> for ToonMaterialKey {
    fn from(material: &ToonMaterial) -> Self {
        ToonMaterialKey {
            cull_mode: material.cull_mode,
            disable_outline: material.disable_outline,
        }
    }
}

impl Material for ToonMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/toon.wgsl".into()
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let fragment = descriptor.fragment.as_mut().unwrap();
        if fragment.shader_defs.contains(&"NORMAL_PREPASS".into())
            && fragment.shader_defs.contains(&"DEPTH_PREPASS".into())
            && !key.bind_group_data.disable_outline
        {
            fragment.shader_defs.push("DRAW_OUTLINE".into());
        }

        descriptor.primitive.cull_mode = key.bind_group_data.cull_mode;
        Ok(())
    }
}

pub fn replace_with_toon_materials(
    base_material: ToonMaterial,
) -> Box<MaterialBuilder<ToonMaterial>> {
    Box::new(move |_name: &Name, standard_material: &StandardMaterial| {
        let mut material = base_material.clone();
        material.color = standard_material.base_color;
        material
            .texture
            .clone_from(&standard_material.base_color_texture);
        Some(material)
    })
}

#[derive(Component, Default)]
pub struct ApplyToonMaterial {
    pub base_material: ToonMaterial,
}

fn apply_toon_materials(
    query: Query<(&SceneInstance, Entity, &ApplyToonMaterial), (Changed<SceneInstance>,)>,
    mut commands: Commands,
    scene_manager: Res<SceneSpawner>,
    mut materials: ResMut<Assets<ToonMaterial>>,
    standard_materials: ResMut<Assets<StandardMaterial>>,
    standard_material_query: Query<&Handle<StandardMaterial>>,
) {
    for (scene_instance, entity, apply_outline) in &query {
        if !scene_manager.instance_is_ready(**scene_instance) {
            continue;
        }
        for entity in scene_manager.iter_instance_entities(**scene_instance) {
            if let Ok(handle) = standard_material_query.get(entity) {
                let Some(material) = standard_materials.get(handle) else {
                    continue;
                };
                let outline_material = materials.add(ToonMaterial {
                    color: material.base_color,
                    texture: material.base_color_texture.clone(),
                    ..apply_outline.base_material.clone()
                });

                let Some(mut entity_commands) = commands.get_entity(entity) else {
                    continue;
                };
                entity_commands
                    .insert(outline_material)
                    .remove::<Handle<StandardMaterial>>();
            }
        }
        commands.entity(entity).remove::<ApplyToonMaterial>();
    }
}

pub struct ToonMaterialPlugin;

impl Plugin for ToonMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_reflect::<ToonMaterial>()
            .add_plugins(MaterialPlugin::<ToonMaterial> {
                prepass_enabled: true,
                ..default()
            })
            .add_plugins(MaterialPlugin::<PlanetMaterial> {
                prepass_enabled: true,
                ..default()
            })
            .add_systems(PostUpdate, apply_toon_materials);
    }
}
