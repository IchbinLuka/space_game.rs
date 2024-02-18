use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_mod_outline::OutlineBundle;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::states::AppState;
use crate::{
    components::gravity::GravitySource,
    states::ON_GAME_STARTED,
    utils::{collisions::PLANET_COLLISION_GROUP, materials::default_outline},
    OutlineMaterial,
};

use super::{
    bullet::{BulletTarget, BulletType},
    spaceship::SpaceshipCollisions,
};

pub struct PlanetPlugin;

impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, PlanetAssets>(AppState::MainSceneLoading)
            .add_systems(ON_GAME_STARTED, planet_setup);
    }
}

#[derive(Component)]
pub struct Planet {
    pub radius: f32,
}

#[derive(AssetCollection, Resource)]
struct PlanetAssets {
    #[asset(path = "textures/planet_1.png")]
    texture: Handle<Image>,
}

fn planet_setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<OutlineMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    planet_assets: Res<PlanetAssets>,
) {
    let collision_groups = CollisionGroups::new(PLANET_COLLISION_GROUP, Group::ALL);

    let mesh = meshes.add(
        shape::UVSphere {
            sectors: 20,
            radius: 1.0,
            ..default()
        }
        .into(),
    );

    let mut rng = rand::thread_rng();

    let asteroids = [("d0d0d0", 10.0), ("db4123", 15.0), ("365df7", 7.0)];

    for (color, size) in asteroids {
        let material = materials.add(OutlineMaterial {
            color: Color::hex(color).unwrap(),
            filter_scale: 5.,
            texture: Some(planet_assets.texture.clone()),
            ..default()
        });

        let angvel = Vec3 {
            y: rng.gen_range(-0.1..0.1),
            ..Vec3::ZERO
        };

        commands.spawn((
            MaterialMeshBundle {
                mesh: mesh.clone(),
                material,
                transform: Transform {
                    translation: Vec3::new(
                        rng.gen_range(50.0..250.0),
                        0.0,
                        rng.gen_range(-100.0..100.0),
                    ),
                    rotation: Quat::from_euler(
                        EulerRot::XYZ,
                        rng.gen_range(0.0..std::f32::consts::PI),
                        rng.gen_range(0.0..std::f32::consts::PI),
                        rng.gen_range(0.0..std::f32::consts::PI),
                    ),
                    scale: Vec3::splat(size),
                },
                ..default()
            },
            Planet { radius: size },
            Collider::ball(1.0),
            RigidBody::KinematicVelocityBased,
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
            },
            SpaceshipCollisions {
                collision_damage: 10.0,
            },
            BulletTarget {
                target_type: BulletType::Both,
                bullet_damage: None,
            },
        ));
    }
}
