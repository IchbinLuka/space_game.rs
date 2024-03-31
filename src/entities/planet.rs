use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_mod_outline::OutlineBundle;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::materials::toon::PlanetMaterial;
use crate::states::{AppState, DespawnOnCleanup};
use crate::ui::minimap::{MinimapAssets, ShowOnMinimap, MINIMAP_RANGE, MINIMAP_SIZE};
use crate::{
    components::gravity::GravitySource,
    states::ON_GAME_STARTED,
    utils::{collisions::PLANET_COLLISION_GROUP, materials::default_outline},
};

use super::space_station::{setup_space_station, SpaceStation};
use super::{
    bullet::{BulletTarget, BulletType},
    spaceship::SpaceshipCollisions,
};

pub struct PlanetPlugin;

impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, PlanetAssets>(AppState::MainSceneLoading)
            .add_systems(ON_GAME_STARTED, planet_setup.after(setup_space_station));
    }
}

#[derive(Component, Debug)]
pub struct Planet {
    pub radius: f32,
}

#[derive(AssetCollection, Resource)]
pub struct PlanetAssets {
    #[asset(path = "textures/planet_1.png")]
    texture: Handle<Image>,
}

const PLANET_COUNT: usize = 15;

struct PlanetSpawnConfig {
    color: Color,
    size: f32,
    pos: Vec3,
}

pub fn planet_setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<PlanetMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    planet_assets: Res<PlanetAssets>,
    space_stations: Query<&Transform, With<SpaceStation>>,
    minimap_assets: Res<MinimapAssets>,
) {
    let collision_groups = CollisionGroups::new(PLANET_COLLISION_GROUP, Group::ALL);

    let mut rng = rand::thread_rng();

    let mut planets: Vec<PlanetSpawnConfig> = Vec::with_capacity(PLANET_COUNT);

    for _ in 0..PLANET_COUNT {
        let size = rng.gen_range(7.0..25.0);

        let color = Color::rgb(
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
        );

        // Try 10 times to find a suitable position for the planet, then abort
        for _ in 0..10 {
            let pos = Vec3::new(
                rng.gen_range(-300.0..300.0),
                0.0,
                rng.gen_range(-300.0..300.0),
            );
            // Planet should not be too close to other planets
            if planets
                .iter()
                .any(|x| pos.distance(x.pos) < (size + x.size) * 1.5)
            {
                continue;
            }
            // Planet should not be too close to space stations
            if space_stations
                .iter()
                .any(|station| station.translation.distance(pos) < size * 1.5)
            {
                continue;
            }

            planets.push(PlanetSpawnConfig { color, size, pos });
            break;
        }
    }

    for PlanetSpawnConfig { color, size, pos } in planets {
        let material = materials.add(PlanetMaterial {
            center: pos,
            color,
            texture: planet_assets.texture.clone(),
        });

        let mesh = meshes.add(
            shape::UVSphere {
                sectors: 20,
                radius: size,
                ..default()
            }
            .into(),
        );

        let angvel = Vec3 {
            y: rng.gen_range(-0.1..0.1),
            ..Vec3::ZERO
        };

        commands.spawn((
            DespawnOnCleanup, 
            MaterialMeshBundle {
                mesh,
                material,
                transform: Transform {
                    translation: pos,
                    rotation: Quat::from_euler(
                        EulerRot::XYZ,
                        rng.gen_range(0.0..std::f32::consts::PI),
                        rng.gen_range(0.0..std::f32::consts::PI),
                        rng.gen_range(0.0..std::f32::consts::PI),
                    ),
                    ..default()
                },
                ..default()
            },
            Planet { radius: size },
            Collider::ball(size),
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
                bound_radius: size,
            },
            BulletTarget {
                target_type: BulletType::Both,
                bullet_damage: None,
            },
            ShowOnMinimap {
                sprite: minimap_assets.planet_indicator.clone(),
                size: Some(Vec2::splat(size / MINIMAP_RANGE * MINIMAP_SIZE)),
            },
        ));
    }
}
