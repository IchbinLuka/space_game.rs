use bevy::prelude::*;

// TODO

#[derive(Component)]
pub struct HealthBar3d;


#[derive(Bundle)]
pub struct HealthBar3dBundle {
    pub health_bar: HealthBar3d,
    sprite: SpriteBundle,
}


pub struct HealthBar3DPlugin;

impl Plugin for HealthBar3DPlugin {
    fn build(&self, _app: &mut App) {
        
    }
}