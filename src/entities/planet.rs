use bevy::prelude::*;
use bevy_mod_outline::OutlineBundle;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::{AppState, components::gravity::GravitySource, utils::materials::{matte_material, default_outline}};

pub struct PlanetPlugin;

impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Running), planet_setup);
    }
}


fn planet_setup(
    mut commands: Commands, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    
    let mesh = meshes.add(shape::UVSphere {
        sectors: 20, 
        radius: 1.0, 
        ..default()
    }.into());

    let mut rng = rand::thread_rng();

    let asteroids = [
        ("549335", 30.0), 
        ("f77d36", 50.0), 
        ("365df7", 40.0)
    ];

    for (color, size) in asteroids {

        let material = materials.add(StandardMaterial {
            base_color: Color::hex(color).unwrap(), 
            ..matte_material()
        });

        commands.spawn((
            MaterialMeshBundle {
                mesh: mesh.clone(),
                material,
                transform: Transform::from_xyz(
                    rng.gen_range(-100.0..100.0), 
                    0.0, 
                    rng.gen_range(-100.0..100.0)
                ).with_scale(Vec3::splat(size)),
                ..default()
            }, 
            Collider::ball(1.0), 
            RigidBody::Fixed, 
            Velocity::default(), 
            ActiveCollisionTypes::all(), 
            ActiveEvents::COLLISION_EVENTS, 
            Sensor, 
            GravitySource {
                mass: size * 1000.0, 
                ..default()
            }, 
            OutlineBundle {
                outline: default_outline(), 
                ..default()
            }
        ));
    }
}