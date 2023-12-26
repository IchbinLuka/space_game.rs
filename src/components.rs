use bevy::app::{Plugin, App};

pub mod despawn_after;
pub mod gravity;
pub mod colliders;
pub mod movement;

pub struct ComponentsPlugin;

impl Plugin for ComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            despawn_after::DespawnAfterPlugin,
            gravity::GravityPlugin,
            movement::MovementPlugin,
        ));
    }
}