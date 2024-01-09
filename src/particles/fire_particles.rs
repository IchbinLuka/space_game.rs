use bevy::prelude::*;

pub struct FireParticlesPlugin;

impl Plugin for FireParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_fire_particles);
    }
}

#[derive(Resource)]
pub struct FireParticleRes {
    pub mesh: Handle<Mesh>,
    pub materials: [Handle<StandardMaterial>; 4],
}

fn setup_fire_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(shape::Circle::new(0.2).into());

    let colors = [
        Color::hex("ef8904").unwrap(),
        Color::hex("f2600c").unwrap(),
        Color::hex("cc2804").unwrap(),
        Color::hex("e89404").unwrap(),
    ];

    let materials = colors
        .iter()
        .map(|color| {
            materials.add(StandardMaterial {
                emissive: *color,
                base_color: *color,
                ..default()
            })
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    commands.insert_resource(FireParticleRes { mesh, materials })
}
