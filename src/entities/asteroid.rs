use std::{time::Duration, f32::consts::FRAC_PI_2};

use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_mod_outline::{OutlineBundle, OutlineVolume};
use bevy_rapier3d::prelude::*;
use rand::{seq::SliceRandom, Rng};

use crate::{components::{colliders::VelocityColliderBundle, despawn_after::DespawnAfter}, AppState, utils::materials::matte_material};

use super::bullet::BULLET_COLLISION_GROUP;

#[derive(Component)]
pub struct Asteroid;


#[derive(Event)]
pub struct AsteroidSpawnEvent {
    pub position: Transform,
    pub velocity: Velocity,
    pub size: f32,
}

#[derive(Event)]
pub struct AsteroidDestructionEvent {
    pub transform: Transform,
}


fn asteroid_collisions(
    mut commands: Commands, 
    query: Query<(Entity, &CollidingEntities, &Transform), With<Asteroid>>, 
    mut destruction_events: EventWriter<AsteroidDestructionEvent>,

) {
    for (entity, colliding, transform) in &query {
        if colliding.is_empty() { continue; }
        commands.entity(entity).despawn_recursive();
        destruction_events.send(AsteroidDestructionEvent {
            transform: *transform
        });
    }
}

fn asteroid_destruction(
    mut destruction_events: EventReader<AsteroidDestructionEvent>, 
    mut commands: Commands,
    res: Res<AsteroidRes>,
    time: Res<Time>,
) {
    const NUM_DESTRUCTION_PARTICLES: usize = 20;
    let mut rng = rand::thread_rng();
    for event in destruction_events.read() {
        for _ in 0..NUM_DESTRUCTION_PARTICLES {
            commands.spawn((
                MaterialMeshBundle {
                    mesh: res.particle_mesh.clone(),
                    material: res.material.clone(),
                    transform: Transform {
                        translation: event.transform.translation + Vec3::new(
                            rng.gen_range(-1.0..1.0), 
                            rng.gen_range(-1.0..1.0), 
                            rng.gen_range(-1.0..1.0)
                        ),
                        rotation: Quat::from_rotation_x(-FRAC_PI_2),
                        scale: Vec3::splat(rng.gen_range(2.0..5.0)),
                    }, 
                    ..default()
                }, 
                Velocity::linear(Vec3::new(
                    rng.gen_range(-1.0..1.0), 
                    rng.gen_range(-1.0..1.0), 
                    rng.gen_range(-1.0..1.0)
                ).normalize() * rng.gen_range(1.0..4.0)),
                RigidBody::KinematicVelocityBased, 
                DespawnAfter {
                    time: Duration::from_secs(1), 
                    spawn_time: time.elapsed() + Duration::from_millis(rng.gen_range(0..500))
                },
            ));
        }
    }
}

fn asteroid_spawn(
    mut commands: Commands,
    mut spawn_events: EventReader<AsteroidSpawnEvent>, 
    assets: Res<AsteroidAssets>,
    res: Res<AsteroidRes>,
) {
    let collision_groups = CollisionGroups::new(BULLET_COLLISION_GROUP, Group::ALL);

    if spawn_events.is_empty() { return; }
    let mut rng = rand::thread_rng();
    let asteroids = [assets.asteroid_1.clone(), assets.asteroid_2.clone()];
    for event in spawn_events.read() {
        let mesh = asteroids.choose(&mut rng).unwrap();
        commands.spawn((
            MaterialMeshBundle {
                mesh: mesh.clone(),
                material: res.material.clone(),
                transform: event.position,
                ..default()
            }, 
            Asteroid,
            VelocityColliderBundle {
                velocity: event.velocity,
                collider: Collider::ball(1.0), 
                ..default()
            }, 
            OutlineBundle {
                outline: OutlineVolume {
                    visible: true, 
                    width: 4.0,
                    colour: Color::BLACK, 
                }, 
                ..default()
            }, 
            collision_groups, 
        ));
    }
}

#[derive(AssetCollection, Resource)]
struct AsteroidAssets {
    #[asset(path = "asteroid1.obj")]
    asteroid_1: Handle<Mesh>, 
    #[asset(path = "asteroid2.obj")]
    asteroid_2: Handle<Mesh>,
}


#[derive(Resource)]
struct AsteroidRes {
    material: Handle<StandardMaterial>, 
    particle_mesh: Handle<Mesh>,
}

fn asteroid_setup(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::hex("747a8c").unwrap(), 
        ..matte_material()
    });

    let particle_mesh = meshes.add(shape::Circle {
        radius: 0.1, 
        ..default()
    }.into());

    commands.insert_resource(AsteroidRes { material, particle_mesh });
}

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_collection_to_loading_state::<_, AsteroidAssets>(AppState::Loading)
            .add_systems(OnEnter(AppState::Running), asteroid_setup)
            .add_systems(Update, (
                asteroid_spawn, 
                asteroid_collisions, 
                asteroid_destruction
            ).run_if(in_state(AppState::Running)))
            .add_event::<AsteroidSpawnEvent>()
            .add_event::<AsteroidDestructionEvent>();
    }
}
