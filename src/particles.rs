use bevy::{app::Plugin, render::{mesh::Mesh, render_resource::PrimitiveTopology}};

pub mod fire_particles;



fn particle_system_setup() {
    let mut mesh = Mesh::new(PrimitiveTopology::PointList);
}


pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((fire_particles::FireParticlesPlugin,));
    }
}
