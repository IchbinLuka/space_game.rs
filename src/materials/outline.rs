use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct OutlineMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(1)]
    pub settings: OutlineMaterialSettings,
}

#[derive(Debug, Clone, AsBindGroup, ShaderType)]
pub struct OutlineMaterialSettings {
    pub cross_scale: f32,
    pub depth_threshold: f32,
    pub normal_threshold: f32,
    pub depth_normal_threshold_scale: f32,
    pub depth_normal_threshold: f32,
}

impl Default for OutlineMaterialSettings {
    fn default() -> Self {
        Self {
            cross_scale: 5.0,
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

pub struct OutlineMaterialPlugin;

impl Plugin for OutlineMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<OutlineMaterial> {
            prepass_enabled: true,
            ..default()
        });
    }
}
