use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{AsBindGroup, AsBindGroupShaderType, ShaderRef, ShaderType},
    },
    scene::SceneInstance,
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[uniform(0, OutlineMaterialUniform)]
pub struct OutlineMaterial {
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
pub struct OutlineMaterialUniform {
    pub filter_scale: f32,
    pub depth_threshold: f32,
    pub normal_threshold: f32,
    pub depth_normal_threshold_scale: f32,
    pub depth_normal_threshold: f32,
    pub use_texture: u32,
    pub color: Color,
}

impl AsBindGroupShaderType<OutlineMaterialUniform> for OutlineMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<Image>) -> OutlineMaterialUniform {
        OutlineMaterialUniform {
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

impl Default for OutlineMaterial {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            texture: None,
            filter_scale: 5.0,
            depth_threshold: 0.01,
            normal_threshold: 0.8,
            depth_normal_threshold_scale: 20.0,
            depth_normal_threshold: 0.5,
        }
    }
}

#[derive(Debug, Clone, AsBindGroup, ShaderType)]
pub struct OutlineMaterialSettings {
    pub filter_scale: f32,
    pub depth_threshold: f32,
    pub normal_threshold: f32,
    pub depth_normal_threshold_scale: f32,
    pub depth_normal_threshold: f32,
}

impl Default for OutlineMaterialSettings {
    fn default() -> Self {
        Self {
            filter_scale: 5.0,
            depth_threshold: 0.01,
            normal_threshold: 0.8,
            depth_normal_threshold_scale: 20.0,
            depth_normal_threshold: 0.5,
        }
    }
}

impl Material for OutlineMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/outline.wgsl".into()
    }
}

#[derive(Component, Default)]
pub struct ApplyOutlineMaterial {
    pub base_material: OutlineMaterial,
}

fn apply_outline_materials(
    query: Query<(&SceneInstance, Entity, &ApplyOutlineMaterial), (Changed<SceneInstance>,)>,
    mut commands: Commands,
    scene_manager: Res<SceneSpawner>,
    mut materials: ResMut<Assets<OutlineMaterial>>,
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

                    let outline_material = materials.add(OutlineMaterial {
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
        commands.entity(entity).remove::<ApplyOutlineMaterial>();
    }
}

pub struct OutlineMaterialPlugin;

impl Plugin for OutlineMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<OutlineMaterial> {
            prepass_enabled: true,
            ..default()
        })
        .add_systems(Update, apply_outline_materials);
    }
}
