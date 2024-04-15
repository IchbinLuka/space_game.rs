use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_mod_outline::OutlineBundle;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::materials::toon::PlanetMaterial;
use crate::states::{AppState, DespawnOnCleanup};
use crate::ui::minimap::{MinimapAssets, MinimapSize, ShowOnMinimap, MINIMAP_RANGE, MINIMAP_SIZE};
use crate::utils::asset_loading::AppExtension;
use crate::utils::misc::AsCommand;
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
        app.add_collection_to_loading_states::<PlanetAssets>(&[
            AppState::MainSceneLoading,
            AppState::StartScreenLoading,
        ])
        .add_systems(
            ON_GAME_STARTED,
            planet_setup_main_scene.after(setup_space_station),
        )
        .add_systems(Startup, planet_setup);
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

#[derive(Resource)]
pub struct PlanetRes {
    mesh: Handle<Mesh>,
}

const PLANET_COUNT: usize = 15;
const COLLISION_GROUPS: CollisionGroups = CollisionGroups::new(PLANET_COLLISION_GROUP, Group::ALL);

pub struct PlanetSpawnConfig {
    pub color: Color,
    pub size: f32,
    pub pos: Vec3,
}

fn planet_setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(PlanetRes {
        mesh: meshes.add(Sphere { radius: 1.0 }.mesh().uv(32, 18)),
    });
}

pub fn planet_setup_main_scene(
    mut commands: Commands,
    space_stations: Query<&Transform, With<SpaceStation>>,
) {
    let mut rng = rand::thread_rng();

    let mut planets: Vec<PlanetSpawnConfig> = Vec::with_capacity(PLANET_COUNT);

    for _ in 0..PLANET_COUNT {
        let size = rng.gen_range(7.0..25.0);

        let color = Color::hsl(
            rng.gen_range(0.0..360.0),
            rng.gen_range(0.5..1.0),
            rng.gen_range(0.5..0.8),
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

    for config in planets {
        commands.add(spawn_planet.to_command(config));
    }
}

pub fn spawn_planet(
    In(config): In<PlanetSpawnConfig>,
    mut commands: Commands,
    mut materials: ResMut<Assets<PlanetMaterial>>,
    planet_assets: Res<PlanetAssets>,
    minimap_assets: Res<MinimapAssets>,
    planet_res: Res<PlanetRes>,
) {
    let PlanetSpawnConfig { color, size, pos } = config;

    let mut rng = rand::thread_rng();

    let material = materials.add(PlanetMaterial {
        center: Vec4::new(pos.x, pos.y, pos.z, 0.0),
        color,
        texture: planet_assets.texture.clone(),
    });

    let angvel = Vec3 {
        y: rng.gen_range(-0.1..0.1),
        ..Vec3::ZERO
    };

    commands.spawn((
        DespawnOnCleanup,
        MaterialMeshBundle {
            mesh: planet_res.mesh.clone(),
            material,
            transform: Transform {
                translation: pos,
                rotation: Quat::from_euler(
                    EulerRot::XYZ,
                    rng.gen_range(0.0..std::f32::consts::PI),
                    rng.gen_range(0.0..std::f32::consts::PI),
                    rng.gen_range(0.0..std::f32::consts::PI),
                ),
                // ..default()
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
        COLLISION_GROUPS,
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
            size: MinimapSize::Custom(Vec2::splat(size / MINIMAP_RANGE * MINIMAP_SIZE)),
        },
    ));
}
