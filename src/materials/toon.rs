use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{AsBindGroup, AsBindGroupShaderType, ShaderRef, ShaderType},
    },
    scene::SceneInstance,
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[uniform(0, ToonMaterialUniform)]
pub struct PlanetMaterial {
    pub color: Color,

    #[uniform(3)]
    pub center: Vec3,

    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}

impl AsBindGroupShaderType<ToonMaterialUniform> for PlanetMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<Image>) -> ToonMaterialUniform {
        ToonMaterialUniform {
            color: self.color,
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

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[uniform(0, ToonMaterialUniform)]
pub struct ToonMaterial {
    pub color: Color,

    #[texture(1)]
    #[sampler(2)]
    #[dependency]
    pub texture: Option<Handle<Image>>,

    pub filter_scale: f32,
    pub depth_threshold: f32,
    pub normal_threshold: f32,
    pub depth_normal_threshold_scale: f32,
    pub depth_normal_threshold: f32,
}

#[derive(Debug, Clone, AsBindGroup, ShaderType)]
pub struct ToonMaterialUniform {
    pub filter_scale: f32,
    pub depth_threshold: f32,
    pub normal_threshold: f32,
    pub depth_normal_threshold_scale: f32,
    pub depth_normal_threshold: f32,
    pub use_texture: u32,
    pub color: Color,
}

impl AsBindGroupShaderType<ToonMaterialUniform> for ToonMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<Image>) -> ToonMaterialUniform {
        ToonMaterialUniform {
            filter_scale: self.filter_scale,
            depth_threshold: self.depth_threshold,
            normal_threshold: self.normal_threshold,
            depth_normal_threshold_scale: self.depth_normal_threshold_scale,
            depth_normal_threshold: self.depth_normal_threshold,
            use_texture: if self.texture.is_some() { 1 } else { 0 },
            color: self.color,
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
            color: DEFAULT_COLOR,
        }
    }
}


impl Material for ToonMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/toon.wgsl".into()
    }
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
        if scene_manager.instance_is_ready(**scene_instance) {
            for entity in scene_manager.iter_instance_entities(**scene_instance) {
                if let Ok(handle) = standard_material_query.get(entity) {
                    let Some(material) = standard_materials.get(handle) else {
                        continue;
                    };

                    let outline_material = materials.add(ToonMaterial {
                        color: material.base_color,
                        ..apply_outline.base_material.clone()
                    });

                    commands
                        .entity(entity)
                        .remove::<Handle<StandardMaterial>>()
                        .insert(outline_material);
                }
            }
        }
        commands.entity(entity).remove::<ApplyToonMaterial>();
    }
}

pub struct ToonMaterialPlugin;

impl Plugin for ToonMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ToonMaterial> {
            prepass_enabled: true,
            ..default()
        })
        .add_plugins(MaterialPlugin::<PlanetMaterial> {
            prepass_enabled: true,
            ..default()
        })
        .add_systems(Update, apply_toon_materials);
    }
}
