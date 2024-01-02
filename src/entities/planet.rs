use bevy::prelude::*;
use bevy_mod_outline::OutlineBundle;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::{AppState, components::gravity::GravitySource, utils::materials::{matte_material, default_outline}};

use super::bullet::BULLET_COLLISION_GROUP;

pub struct PlanetPlugin;

impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Running), planet_setup);
    }
}

#[derive(Component)]
pub struct Planet {
    pub radius: f32,
}


fn planet_setup(
    mut commands: Commands, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let collision_groups = CollisionGroups::new(BULLET_COLLISION_GROUP | Group::GROUP_3, Group::ALL);
    
    let mesh = meshes.add(shape::UVSphere {
        sectors: 20, 
        radius: 1.0, 
        ..default()
    }.into());

    let mut rng = rand::thread_rng();

    let asteroids = [
        ("549335", 20.0), 
        ("f77d36", 30.0), 
        ("365df7", 15.0)
    ];

    for (color, size) in asteroids {

        let material = materials.add(StandardMaterial {
            base_color: Color::hex(color).unwrap(), 
            ..matte_material()
        });

        let angvel = Vec3 { y: rng.gen_range(-0.1..0.1), ..Vec3::ZERO };

        commands.spawn((
            MaterialMeshBundle {
                mesh: mesh.clone(),
                material,
                transform: Transform::from_xyz(
                    rng.gen_range(50.0..250.0), 
                    0.0, 
                    rng.gen_range(-100.0..100.0)
                ).with_scale(Vec3::splat(size)),
                ..default()
            }, 
            Planet {
                radius: size, 
            }, 
            Collider::ball(1.0), 
            RigidBody::Fixed, 
            Velocity {
                angvel, 
                ..default()
            }, 
            collision_groups,
            ActiveCollisionTypes::all(), 
            ActiveEvents::COLLISION_EVENTS, 
            GravitySource {
                mass: size * 500.0, 
                ..default()
            }, 
            OutlineBundle {
                outline: default_outline(), 
                ..default()
            }
        ));
    }
}