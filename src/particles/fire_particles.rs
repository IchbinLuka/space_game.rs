use bevy::prelude::*;

use super::ParticleMaterial;

pub struct FireParticlesPlugin;

impl Plugin for FireParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_fire_particles);
    }
}

#[derive(Resource)]
pub struct FireParticleRes {
    pub mesh: Handle<Mesh>,
    pub materials: [Handle<ParticleMaterial>; 4],
}

fn setup_fire_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ParticleMaterial>>,
) {
    let mesh = meshes.add(Rectangle::new(0.4, 0.4));

    let colors = [
        Srgba::hex("ef8904").unwrap().into(),
        Srgba::hex("f2600c").unwrap().into(),
        Srgba::hex("e06411").unwrap().into(),
        Srgba::hex("e89404").unwrap().into(),
    ];

    let materials = colors
        .iter()
        .map(|color| materials.add(ParticleMaterial { color: *color }))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    commands.insert_resource(FireParticleRes { mesh, materials })
}
