use std::{f32::consts::FRAC_PI_2, time::Duration};

use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_mod_outline::OutlineBundle;
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::{
    components::{colliders::VelocityColliderBundle, despawn_after::DespawnAfter},
    utils::{
        materials::{default_outline, matte_material},
        sets::Set, misc::CollidingEntitiesExtension,
    },
    AppState, particles::ParticleMaterial, ui::score::ScoreEvent, entities::bullet::BulletType,
};

use super::{bullet::{BULLET_COLLISION_GROUP, Bullet}, explosion::ExplosionEvent, spaceship::player::Player};

#[derive(Component)]
pub struct Asteroid;

impl Asteroid {
    const COLLISION_GROUPS: CollisionGroups = CollisionGroups::new(BULLET_COLLISION_GROUP, Group::ALL);
}

#[derive(Component)]
pub struct AsteroidField;


#[derive(Event)]
pub struct AsteroidDestructionEvent {
    pub entity: Entity,
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
            let distance = field_transform
                .translation
                .distance(player_transform.translation);
            distance > 100.0
        });
        if spawn_asteroid_field {
            let mut rng = rand::thread_rng();
            let position = player_transform.translation + 
                player_velocity.linvel.normalize() * 100.0 + 
                Vec3::new(rng.gen_range(-50.0..50.0), 0.0, rng.gen_range(-50.0..50.0));
            commands
                .spawn((
                    AsteroidField,
                    Transform::from_translation(position),
                    GlobalTransform::default(),
                    InheritedVisibility::VISIBLE,
                ))
                .with_children(|c| {
                    let num_asteroids = rng.gen_range(10..50);
                    for _ in 0..num_asteroids {
                        let translation =
                            Vec3::new(rng.gen_range(-20.0..20.0), 0.0, rng.gen_range(-20.0..20.0));

                        let rotation =
                            Quat::from_rotation_y(rng.gen_range(0.0..std::f32::consts::PI * 2.0));
                        let scale = Vec3::splat(rng.gen_range(0.7..1.4));
                        let linvel = Vec3::new(
                            rand::random::<f32>() - 0.5,
                            0.0,
                            rand::random::<f32>() - 0.5,
                        );
                        let angvel = Vec3::Y * (rng.gen_range(-0.5..0.5));

                        let mesh = if rng.gen::<bool>() {
                            assets.asteroid_1.clone()
                        } else {
                            assets.asteroid_2.clone()
                        };

                        c.spawn(AsteroidBundle {
                            mesh_bundle: MaterialMeshBundle {
                                mesh: mesh.clone(),
                                material: res.material.clone(),
                                transform: Transform {
                                    translation,
                                    rotation,
                                    scale,
                                },
                                ..default()
                            },
                            asteroid: Asteroid,
                            velocity_collider_bundle: VelocityColliderBundle {
                                velocity: Velocity { linvel, angvel },
                                collider: Collider::ball(1.0),
                                ..default()
                            },
                            outline_bundle: OutlineBundle {
                                outline: default_outline(),
                                ..default()
                            },
                            collision_groups: Asteroid::COLLISION_GROUPS,
                        });
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
            let distance = field_transform
                .translation
                .distance(player_transform.translation);
            if distance > 200.0 {
                debug!("Despawning asteroid field");
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn asteroid_collisions(
    mut commands: Commands,
    query: Query<(Entity, &CollidingEntities, &GlobalTransform), With<Asteroid>>,
    mut explosions: EventWriter<ExplosionEvent>,
    bullet_query: Query<&Bullet>,
    res: Res<AsteroidRes>,
    time: Res<Time>,
    mut score_events: EventWriter<ScoreEvent>,
) {
    const NUM_DESTRUCTION_PARTICLES: usize = 20;

    for (entity, colliding, global_transform) in &query {
        if colliding.is_empty() {
            continue;
        }
        let mut rng = rand::thread_rng();
        commands.entity(entity).despawn_recursive();

        let transform = global_transform.compute_transform();

        for bullet in colliding.filter_fulfills_query(&bullet_query) {
            if bullet.bullet_type == BulletType::Player {
                score_events.send(ScoreEvent {
                    score: 10,
                    world_pos: transform.translation,
                });
                break;
            }
        }

        explosions.send(ExplosionEvent {
            position: transform.translation,
            ..default()
        });
        for _ in 0..NUM_DESTRUCTION_PARTICLES {
            commands.spawn((
                MaterialMeshBundle {
                    mesh: res.particle_mesh.clone(),
                    material: res.particle_material.clone(),
                    transform: Transform {
                        translation: transform.translation
                            + Vec3::new(
                                rng.gen_range(-1.0..1.0),
                                rng.gen_range(-1.0..1.0),
                                rng.gen_range(-1.0..1.0),
                            ),
                        rotation: Quat::from_rotation_x(-FRAC_PI_2),
                        scale: Vec3::splat(rng.gen_range(2.0..5.0)),
                    },
                    ..default()
                },
                Velocity::linear(
                    Vec3::new(
                        rng.gen_range(-1.0..1.0),
                        rng.gen_range(-1.0..1.0),
                        rng.gen_range(-1.0..1.0),
                    )
                    .normalize()
                        * rng.gen_range(1.0..4.0),
                ),
                RigidBody::KinematicVelocityBased,
                DespawnAfter {
                    time: Duration::from_secs(1),
                    spawn_time: time.elapsed() + Duration::from_millis(rng.gen_range(0..500)),
                },
            ));
        }
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
    particle_material: Handle<ParticleMaterial>,
}

fn asteroid_setup(
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut particle_materials: ResMut<Assets<ParticleMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let material = standard_materials.add(StandardMaterial {
        base_color: Color::hex("747a8c").unwrap(),
        ..matte_material()
    });

    let particle_material = particle_materials.add(ParticleMaterial {
        color: Color::hex("665F64").unwrap(),
    });

    let particle_mesh = meshes.add(shape::Quad::new(Vec2::splat(0.2)).into(),);

    commands.insert_resource(AsteroidRes {
        material,
        particle_mesh,
        particle_material,
    });
}

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, AsteroidAssets>(AppState::MainSceneLoading)
            .add_systems(OnEnter(AppState::MainScene), asteroid_setup)
            .add_systems(
                Update,
                (
                    asteroid_collisions
                        .in_set(Set::ExplosionEvents)
                        .in_set(Set::ScoreEvents),
                    spawn_asteroid_field,
                    despawn_asteroid_field,
                )
                    .run_if(in_state(AppState::MainScene)),
            )
            .add_event::<AsteroidDestructionEvent>();
    }
}
