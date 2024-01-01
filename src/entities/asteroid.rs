use std::{time::Duration, f32::consts::FRAC_PI_2};

use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_mod_outline::OutlineBundle;
use bevy_rapier3d::prelude::*;
use rand::{seq::SliceRandom, Rng, rngs::ThreadRng};

use crate::{components::{colliders::VelocityColliderBundle, despawn_after::DespawnAfter}, AppState, utils::materials::{matte_material, default_outline}};

use super::{bullet::BULLET_COLLISION_GROUP, player::Player};

#[derive(Component)]
pub struct Asteroid;

#[derive(Component)]
pub struct AsteroidField;


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

fn spawn_asteroid_field(
    mut commands: Commands,
    player_query: Query<(&Transform, &Velocity), With<Player>>,
    asteroid_fields: Query<&Transform, With<AsteroidField>>,
    res: Res<AsteroidRes>,
    assets: Res<AsteroidAssets>,
) {
    for (player_transform, player_velocity) in &player_query {
        let spawn_asteroid_field = asteroid_fields.iter().all(|field_transform| {
            let distance = field_transform.translation.distance(player_transform.translation);
            distance > 100.0
        });
        if spawn_asteroid_field {
            let mut rng = rand::thread_rng();
            let position = player_transform.translation + player_velocity.linvel.normalize() * 100.0 + Vec3::new(
                rng.gen_range(-50.0..50.0), 
                0.0, 
                rng.gen_range(-50.0..50.0)
            );
            commands.spawn((
                AsteroidField,
                Transform::from_translation(position), 
                GlobalTransform::default(), 
                InheritedVisibility::VISIBLE, 
            )).with_children(|c| {
                let num_asteroids = rng.gen_range(10..50);
                for _ in 0..num_asteroids {
                    let translation = Vec3::new(
                        rng.gen_range(-20.0..20.0), 
                        0.0, 
                        rng.gen_range(-20.0..20.0)
                    );

                    let rotation = Quat::from_rotation_y(rng.gen_range(0.0..std::f32::consts::PI * 2.0));
                    let scale = Vec3::splat(rng.gen_range(0.7..1.4));
                    let linvel = Vec3::new(
                        rand::random::<f32>() - 0.5, 
                        0.0, 
                        rand::random::<f32>() - 0.5
                    );
                    let angvel = Vec3::Y * (rng.gen_range(-0.5..0.5));


                    c.spawn(AsteroidBundle::random(
                        &mut rng, 
                        &res, 
                        &assets, 
                        Transform {
                            translation,
                            rotation, 
                            scale, 
                        },
                        Velocity {
                            linvel, 
                            angvel, 
                        }
                    ));
                }
            });
        }
    }
}

fn despawn_asteroid_field(
    mut commands: Commands, 
    player_query: Query<&Transform, With<Player>>,
    asteroid_fields: Query<(Entity, &Transform), With<AsteroidField>>,
) {
    for player_transform in &player_query {
        for (entity, field_transform) in &asteroid_fields {
            let distance = field_transform.translation.distance(player_transform.translation);
            if distance > 200.0 {
                info!("Despawning asteroid field");
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}


fn asteroid_collisions(
    mut commands: Commands, 
    query: Query<(Entity, &CollidingEntities, &GlobalTransform), With<Asteroid>>, 
    mut destruction_events: EventWriter<AsteroidDestructionEvent>,

) {
    for (entity, colliding, global_transform) in &query {
        if colliding.is_empty() { continue; }
        commands.entity(entity).despawn_recursive();
        destruction_events.send(AsteroidDestructionEvent {
            transform: global_transform.compute_transform()
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
                outline: default_outline(), 
                ..default()
            }, 
            collision_groups, 
        ));
    }
}

#[derive(Bundle)]
struct AsteroidBundle {
    mesh_bundle: MaterialMeshBundle<StandardMaterial>, 
    asteroid: Asteroid,
    velocity_collider_bundle: VelocityColliderBundle,
    outline_bundle: OutlineBundle,
    collision_groups: CollisionGroups,
}

impl AsteroidBundle {
    const COLLISION_GROUPS: CollisionGroups = CollisionGroups::new(BULLET_COLLISION_GROUP, Group::ALL);

    fn random(rng: &mut ThreadRng, res: &AsteroidRes, assets: &AsteroidAssets,  position: Transform, velocity: Velocity) -> Self {
        let mesh = if rng.gen::<bool>() {
            assets.asteroid_1.clone()
        } else {
            assets.asteroid_2.clone()
        };
        Self {
            mesh_bundle: MaterialMeshBundle {
                mesh,
                material: res.material.clone(),
                transform: position,
                ..default()
            }, 
            asteroid: Asteroid,
            velocity_collider_bundle: VelocityColliderBundle {
                velocity,
                collider: Collider::ball(1.0), 
                ..default()
            }, 
            outline_bundle: OutlineBundle {
                outline: default_outline(), 
                ..default()
            }, 
            collision_groups: Self::COLLISION_GROUPS, 
        }
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
                asteroid_destruction, 
                spawn_asteroid_field,
                despawn_asteroid_field,
            ).run_if(in_state(AppState::Running)))
            .add_event::<AsteroidSpawnEvent>()
            .add_event::<AsteroidDestructionEvent>();
    }
}
