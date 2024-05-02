use bevy::{prelude::*, render::render_resource::{AsBindGroup, ShaderRef}};



#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ShieldMaterial {
    #[uniform(0)]
    pub color: Color, 
}

impl Default for ShieldMaterial {
    fn default() -> Self {
        Self {
            color: Color::hex("6fc1fc").unwrap(),
        }
    }

}

impl Material for ShieldMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/shield.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

pub struct ShieldMaterialPlugin;
impl Plugin for ShieldMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ShieldMaterial>::default());
    }
}