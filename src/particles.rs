use bevy::app::Plugin;

pub mod fire_particles;

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((fire_particles::FireParticlesPlugin,));
    }
}
