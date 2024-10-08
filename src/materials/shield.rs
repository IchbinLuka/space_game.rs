use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};
use bevy_rapier3d::{
    dynamics::RigidBody,
    geometry::{ActiveCollisionTypes, Collider, CollidingEntities},
};

use crate::{
    components::health::{Health, Shield},
    entities::bullet::BulletTarget,
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ShieldMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
}

#[derive(Bundle, Default)]
pub struct ShieldBundle {
    pub material_mesh: MaterialMeshBundle<ShieldMaterial>,
    pub not_shadow_caster: NotShadowCaster,
    pub not_shadow_receiver: NotShadowReceiver,
    pub shield: Shield,
    pub health: Health,
    pub bullet_target: BulletTarget,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub colliding_entities: CollidingEntities,
    pub active_collision_types: ActiveCollisionTypes,
}

impl Default for ShieldMaterial {
    fn default() -> Self {
        Self {
            color: Srgba::hex("6fc1fc").unwrap().into(),
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
