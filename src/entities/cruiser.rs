use std::f32::consts::{FRAC_PI_2, PI};
use std::ops::Range;
use std::time::Duration;

use bevy::animation::RepeatAnimation;
use bevy::ecs::system::{EntityCommand, RunSystemOnce};
use bevy::ecs::world::Command;
use bevy::pbr::NotShadowReceiver;
use bevy::prelude::*;
use bevy::scene::SceneInstance;
use bevy_asset_loader::loading_state::config::{ConfigureLoadingState, LoadingStateConfig};
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_mod_outline::OutlineBundle;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use space_game_common::EnemyType;

use crate::components::health::HasShield;
use crate::components::{
    colliders::VelocityColliderBundle, despawn_after::DespawnTimer, health::Health,
};
use crate::entities::spaceship::bot::SpawnSquad;
use crate::materials::exhaust::{ExhaustMaterial, ExhaustRes};
use crate::materials::shield::{ShieldBundle, ShieldMaterial};
use crate::materials::toon::{replace_with_toon_materials, ToonMaterial};
use crate::states::main_scene::GameTime;
use crate::states::{game_running, AppState, DespawnOnCleanup, ON_GAME_STARTED};
use crate::ui::game_hud::{ScoreGameEvent, SpawnEnemyIndicator};
use crate::ui::health_bar_3d::SpawnHealthBar;
use crate::ui::minimap::{MinimapAssets, ShowOnMinimap};
use crate::utils::collisions::CRUISER_COLLISION_GROUP;
use crate::utils::materials::default_outline;
use crate::utils::math::sphere_intersection;
use crate::utils::misc::{AsCommand, CollidingEntitiesExtension, Comparef32};
use crate::utils::scene::{AnimationRoot, ReplaceMaterialPlugin};
use crate::utils::sets::Set;

use super::bullet::{Bullet, BulletTarget, BulletType};
use super::explosion::ExplosionEvent;
use super::planet::Planet;
use super::space_station::SpaceStation;
use super::spaceship::bot::EnemyTarget;
use super::spaceship::{IsBot, SpaceshipCollisions};
use super::turret::Turret;
use super::Enemy;

const CRUISER_HITBOX_SIZE: Vec3 = Vec3::new(3.5, 3., 13.);
const CRUISER_SPEED: f32 = 2.0;
const COLLISION_GROUPS: CollisionGroups = CollisionGroups::new(CRUISER_COLLISION_GROUP, Group::ALL);

// Turret contants
const TURRET_ROTATION_BOUNDS: (f32, f32) = (-1., 1.);

#[derive(Component)]
pub struct Cruiser {
    enemy_spawn_cooldown: Timer,
    travel_timer: Timer,
}

#[derive(Component)]
pub struct CruiserShield;

#[derive(Component)]
struct ShieldDisabled;

#[derive(Component)]
struct CruiserTrail;

#[derive(AssetCollection, Resource)]
struct CruiserAssets {
    #[asset(path = "cruiser.glb#Scene0")]
    pub cruiser_model: Handle<Scene>,
    #[asset(path = "cruiser.glb#Animation0")]
    pub cruiser_animation: Handle<AnimationClip>,
}

#[derive(Resource)]
struct CruiserRes {
    exhaust_material: Handle<ExhaustMaterial>,
}

#[derive(Component, Deref, DerefMut)]
struct ShieldRegenerate(pub Timer);

struct DeactivateShield;

impl EntityCommand for DeactivateShield {
    fn apply(self, id: Entity, world: &mut World) {
        world
            .entity_mut(id)
            .insert(Visibility::Hidden)
            .insert(ColliderDisabled)
            .insert(ShieldDisabled)
            .insert(ShieldRegenerate(Timer::from_seconds(5.0, TimerMode::Once)));

        let Some(parent) = world.entity(id).get::<Parent>() else {
            return;
        };
        world.entity_mut(parent.get()).remove::<HasShield>();
    }
}

struct ActivateShield;

impl EntityCommand for ActivateShield {
    fn apply(self, id: Entity, world: &mut World) {
        world
            .entity_mut(id)
            .insert(Visibility::Visible)
            .remove::<ColliderDisabled>()
            .remove::<ShieldDisabled>();

        let Some(parent) = world.entity(id).get::<Parent>() else {
            return;
        };
        world.entity_mut(parent.get()).insert(HasShield);
    }
}

#[derive(Component)]
pub struct CruiserTurret;

const CRUISER_SPAWN_COOLDOWN: f32 = 30.0;

#[derive(Resource, Deref, DerefMut)]
struct CruiserSpawnTimer(Timer);

fn spawn_cruisers(
    mut spawn_cruiser_events: EventWriter<SpawnCruiserEvent>,
    mut last_cruiser_spawn: ResMut<CruiserSpawnTimer>,
    time: Res<Time>,
    game_time: Res<GameTime>,
) {
    last_cruiser_spawn.set_duration(Duration::from_secs_f32(f32::max(
        30.0 - 5.0 / 60.0 * game_time.elapsed_secs(),
        5.0,
    )));
    last_cruiser_spawn.tick(time.delta());
    if last_cruiser_spawn.just_finished() {
        spawn_cruiser_events.send(SpawnCruiserEvent);
    }
}

fn cruiser_spawn_setup(
    mut commands: Commands,
    mut exhaust_materials: ResMut<Assets<ExhaustMaterial>>,
    res: Option<Res<CruiserRes>>,
) {
    let mut timer = Timer::from_seconds(CRUISER_SPAWN_COOLDOWN, TimerMode::Repeating);
    timer.tick(Duration::from_secs_f32(CRUISER_SPAWN_COOLDOWN - 2.0));
    commands.insert_resource(CruiserSpawnTimer(timer));

    if res.is_none() {
        commands.insert_resource(CruiserRes {
            exhaust_material: exhaust_materials.add(ExhaustMaterial {
                inner_color: Srgba::hex("c0eff9").unwrap().into(),
                outer_color: Srgba::hex("3ad8fc").unwrap().into(),
                ..default()
            }),
        });
    }
}

#[derive(Event)]
pub struct SpawnCruiserEvent;

struct NoGoZone {
    center: Vec3,
    radius: f32,
}

fn spawn_cruiser_events(
    mut spawn_events: EventReader<SpawnCruiserEvent>,
    mut commands: Commands,
    space_stations: Query<(&Transform, &SpaceshipCollisions), With<SpaceStation>>,
    planets: Query<(&Transform, &Planet)>,
    cruisers: Query<&Transform, With<Cruiser>>,
) {
    let mut rng = rand::thread_rng();

    if spawn_events.is_empty() {
        return;
    }

    let num_space_stations = space_stations.iter().len();

    if num_space_stations == 0 {
        warn!("Could not spawn cruiser, no space stations found");
        return;
    }

    let mut no_go_zones: Vec<NoGoZone> = Vec::new();

    for (transform, space_ship_collisions) in &space_stations {
        no_go_zones.push(NoGoZone {
            center: transform.translation,
            radius: space_ship_collisions.bound_radius,
        });
    }

    for (transform, planet) in &planets {
        no_go_zones.push(NoGoZone {
            center: transform.translation,
            radius: planet.radius,
        });
    }

    for transform in &cruisers {
        no_go_zones.push(NoGoZone {
            center: transform.translation,
            radius: CRUISER_HITBOX_SIZE.z,
        });
    }

    for _ in spawn_events.read() {
        let (station_transform, _) = space_stations
            .iter()
            .nth(rng.gen_range(0..num_space_stations))
            .unwrap();

        for _ in 0..10 {
            info!("Trying to spawn cruiser");
            const START_OFFSET_RANGE: Range<f32> = -40.0..40.0;
            const START_DISTANCE: f32 = 200.0;

            let dest = station_transform.translation
                + Vec3::new(
                    rng.gen_range(START_OFFSET_RANGE),
                    0.0,
                    rng.gen_range(START_OFFSET_RANGE),
                );

            let delta_normalized =
                Vec3::new(rng.gen_range(-1.0..1.0), 0.0, rng.gen_range(-1.0..1.0)).normalize();

            let start = dest + delta_normalized * START_DISTANCE;

            // Check if path intersects with one of the no-go zones
            if no_go_zones.iter().any(|z| {
                let intersection =
                    sphere_intersection(z.center, z.radius + 10.0, start, dest - start);
                let Some(intersection) = intersection else {
                    return false;
                };
                intersection < 1.0
            }) {
                info!("Path intersects with no-go zone, retrying");
                continue;
            }

            commands.add(spawn_cruiser.to_command((start, dest)));
            break;
        }
    }
}

fn spawn_cruiser(
    In((start_pos, destination)): In<(Vec3, Vec3)>,
    mut commands: Commands,
    assets: Res<CruiserAssets>,
    minimap_res: Res<MinimapAssets>,
) {
    let Vec3 { x, y, z } = CRUISER_HITBOX_SIZE;

    let delta = destination - start_pos;
    let direction = delta.normalize();

    commands.spawn((
        SceneBundle {
            scene: assets.cruiser_model.clone(),
            transform: Transform {
                translation: start_pos,
                rotation: Quat::from_rotation_y(
                    -direction.cross(Vec3::Z).y.signum() * direction.angle_between(Vec3::Z) + PI,
                ),
                ..default()
            },
            ..default()
        },
        VelocityColliderBundle {
            velocity: Velocity {
                linvel: direction * CRUISER_SPEED,
                ..default()
            },
            collider: Collider::cuboid(x, y, z),
            ..default()
        },
        Cruiser {
            enemy_spawn_cooldown: Timer::from_seconds(10.0, TimerMode::Repeating),
            travel_timer: Timer::from_seconds(delta.length() / CRUISER_SPEED, TimerMode::Once),
        },
        BulletTarget {
            target_type: BulletType::Player,
            bullet_damage: Some(20.),
        },
        OutlineBundle {
            outline: default_outline(),
            ..default()
        },
        Health::new(100.0),
        SpaceshipCollisions {
            collision_damage: 5.0,
            ..default()
        },
        Enemy,
        DespawnOnCleanup,
        COLLISION_GROUPS,
        ShowOnMinimap {
            sprite: minimap_res.cruiser_indicator.clone(),
            size: 0.1.into(),
        },
    ));
}

struct FinishCruiser {
    pub cruiser: Entity,
}

impl Command for FinishCruiser {
    fn apply(self, world: &mut World) {
        world.run_system_once_with(self.cruiser, finish_cruiser)
    }
}

fn finish_cruiser(
    In(cruiser): In<Entity>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ShieldMaterial>>,
) {
    let shield = commands
        .spawn((
            ShieldBundle {
                material_mesh: MaterialMeshBundle {
                    mesh: meshes.add(Sphere { radius: 10. }),
                    material: materials.add(ShieldMaterial::default()),
                    transform: Transform::from_scale(Vec3 { z: 2., ..Vec3::ONE }),
                    ..default()
                },
                rigid_body: RigidBody::Fixed,
                active_collision_types: ActiveCollisionTypes::KINEMATIC_STATIC,
                bullet_target: BulletTarget {
                    target_type: BulletType::Player,
                    bullet_damage: Some(10.0),
                },
                health: Health::new(100.0),
                collider: Collider::ball(10.),
                ..default()
            },
            CruiserShield,
            SpaceshipCollisions {
                collision_damage: 10.0,
                ..default()
            },
            COLLISION_GROUPS,
        ))
        .id();

    commands.entity(cruiser).add_child(shield);

    commands.add(SpawnHealthBar {
        entity: cruiser,
        scale: 1.,
        offset: Vec2::ZERO,
        shield_entity: Some(shield),
    });
    commands.add(SpawnEnemyIndicator { enemy: cruiser });

    commands.entity(shield).add(ActivateShield);
}

fn cruiser_animation_start(
    query: Query<&AnimationRoot, (With<Cruiser>, Added<AnimationRoot>)>,
    mut animation_players: Query<&mut AnimationPlayer>,
    assets: Res<CruiserAssets>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    mut commands: Commands,
) {
    for root in &query {
        for entity in &root.player_entites {
            let Ok(mut player) = animation_players.get_mut(*entity) else {
                continue;
            };
            let (graph, animation_index) =
                AnimationGraph::from_clip(assets.cruiser_animation.clone());
            player
                .play(animation_index)
                .set_repeat(RepeatAnimation::Never);
            commands
                .entity(entity.clone())
                .insert(animation_graphs.add(graph));
        }
    }
}

fn cruiser_animations(
    query: Query<(&AnimationRoot, Entity), With<Cruiser>>,
    mut commands: Commands,
    animation_players: Query<&AnimationPlayer>,
) {
    for (root, entity) in &query {
        for player in &root.player_entites {
            let Ok(player) = animation_players.get(*player) else {
                continue;
            };
            if player.all_finished() {
                commands.entity(entity).remove::<AnimationRoot>();
                commands.add(FinishCruiser { cruiser: entity });
            }
        }
    }
}

const CRUISER_TRAIL_LENGTH: f32 = 200.;

fn cruiser_trail_update(mut trails: Query<(&mut Transform, &DespawnTimer), With<CruiserTrail>>) {
    for (mut transform, despawn_timer) in &mut trails {
        let progress = despawn_timer.0.elapsed_secs() / despawn_timer.0.duration().as_secs_f32();
        transform.scale.y = 1.0 - progress;
        transform.translation.z = CRUISER_TRAIL_LENGTH / 2. * (1.0 - progress);
    }
}

#[allow(clippy::too_many_arguments)]
fn cruiser_scene_setup(
    mut cruisers: Query<&SceneInstance, (With<Cruiser>, Changed<SceneInstance>)>,
    names: Query<(&Name, &GlobalTransform), Without<Cruiser>>,
    mut commands: Commands,
    scene_manager: Res<SceneSpawner>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut toon_materials: ResMut<Assets<ToonMaterial>>,
    cruiser_res: Res<CruiserRes>,
    exhaust_res: Res<ExhaustRes>,
) {
    for scene in &mut cruisers {
        if !scene_manager.instance_is_ready(**scene) {
            continue;
        }

        let mut rng = rand::thread_rng();

        for entity in scene_manager.iter_instance_entities(**scene) {
            let Ok((name, global_transform)) = names.get(entity) else {
                continue;
            };
            if name.starts_with("exhaust.") {
                let trail = commands
                    .spawn((
                        DespawnOnCleanup,
                        MaterialMeshBundle {
                            material: toon_materials.add(ToonMaterial {
                                color: Srgba::hex("2ae0ed").unwrap().into(),
                                ..default()
                            }),
                            mesh: meshes.add(Cylinder {
                                radius: 1.,
                                half_height: CRUISER_TRAIL_LENGTH / 2.,
                            }),
                            transform: Transform {
                                rotation: Quat::from_rotation_x(FRAC_PI_2),
                                translation: Vec3::new(0., 0., CRUISER_TRAIL_LENGTH / 2.),
                                ..default()
                            },
                            ..default()
                        },
                        CruiserTrail,
                        NotShadowReceiver,
                        DespawnTimer::new(Duration::from_millis(400)),
                    ))
                    .id();

                let exhaust = commands
                    .spawn((
                        MaterialMeshBundle {
                            material: cruiser_res.exhaust_material.clone(),
                            mesh: exhaust_res.mesh.clone(),
                            transform: Transform {
                                rotation: Quat::from_rotation_x(FRAC_PI_2),
                                scale: Vec3::new(1.5, 0.5, 1.5),
                                ..default()
                            },
                            ..default()
                        },
                        OutlineBundle::default(),
                    ))
                    .id();

                commands.entity(entity).add_child(trail).add_child(exhaust);
            } else if name.starts_with("turret_bone") {
                let mut bullet_timer = Timer::from_seconds(1.0, TimerMode::Repeating);
                bullet_timer.tick(Duration::from_millis(rng.gen_range(0..500)));

                commands.entity(entity).insert((
                    Turret {
                        bullet_timer,
                        base_orientation: *global_transform.compute_transform().forward(),
                        bullet_type: BulletType::Bot,
                        rotation_bounds: TURRET_ROTATION_BOUNDS,
                    },
                    CruiserTurret,
                ));
            }
        }
    }
}

fn cruiser_shield_death(
    query: Query<(Entity, &Health), (With<CruiserShield>, Without<ShieldDisabled>)>,
    mut commands: Commands,
) {
    for (entity, health) in &query {
        if health.is_dead() {
            commands.entity(entity).add(DeactivateShield);
        }
    }
}

fn cruiser_shield_regenerate(
    mut query: Query<(Entity, &mut Health, &mut ShieldRegenerate), With<CruiserShield>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut health, mut timer) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished() {
            commands.entity(entity).add(ActivateShield);
        }
        if timer.finished() {
            health.heal(10.0 * time.delta_seconds());
        }
    }
}

fn cruiser_death(
    query: Query<(&Health, &Transform, Entity), (With<Cruiser>, Changed<Health>)>,
    mut explosion_events: EventWriter<ExplosionEvent>,
    mut score_events: EventWriter<ScoreGameEvent>,
    mut commands: Commands,
) {
    for (health, transform, entity) in &query {
        if !health.is_dead() {
            continue;
        }
        let forward = transform.forward();
        explosion_events.send_batch([
            ExplosionEvent {
                position: transform.translation,
                radius: 10.,
                ..default()
            },
            ExplosionEvent {
                position: transform.translation + forward * 7.,
                radius: 10.,
                ..default()
            },
            ExplosionEvent {
                position: transform.translation - forward * 7.,
                radius: 10.,
                ..default()
            },
        ]);
        score_events.send(ScoreGameEvent {
            enemy: EnemyType::Cruiser,
            world_pos: transform.translation,
        });
        commands.entity(entity).despawn_recursive();
    }
}

fn cruiser_shield_collisions(
    mut shield_query: Query<(&CollidingEntities, &mut ShieldRegenerate), With<CruiserShield>>,
    bullet_query: Query<&Bullet>,
) {
    for (colliding, mut regen) in &mut shield_query {
        for bullet in colliding.filter_fulfills_query(&bullet_query) {
            if bullet.bullet_type != BulletType::Player {
                continue;
            }
            info!("Resetting timer");
            regen.reset();
        }
    }
}

fn cruiser_spawn_bots(
    mut commands: Commands,
    time: Res<Time>,
    mut cruisers: Query<(&Transform, &mut Cruiser)>,
    bots: Query<Entity, IsBot>,
    enemy_targets: Query<&Transform, With<EnemyTarget>>,
) {
    const MAX_BOT_COUNT: usize = 5;

    if bots.iter().count() >= MAX_BOT_COUNT {
        return;
    }

    for (transform, mut spawn_cooldown) in &mut cruisers {
        spawn_cooldown.enemy_spawn_cooldown.tick(time.delta());

        let Some(nearest_target) = enemy_targets
            .iter()
            .min_by_key(|t| Comparef32((t.translation - transform.translation).length()))
        else {
            continue;
        };

        if nearest_target.translation.distance(transform.translation) > 100. {
            continue;
        }

        if spawn_cooldown.enemy_spawn_cooldown.just_finished() {
            commands.add(SpawnSquad {
                squad_size: 3,
                leader_pos: transform.translation,
            });
        }
    }
}

fn cruiser_movement(mut cruisers: Query<(&mut Velocity, &mut Cruiser)>, time: Res<Time>) {
    for (mut velocity, mut cruiser) in &mut cruisers {
        cruiser.travel_timer.tick(time.delta());

        // If the cruiser has reached the destination, stop moving
        if cruiser.travel_timer.just_finished() {
            velocity.linvel = Vec3::ZERO;
        }
    }
}

pub struct CruiserPLugin;

impl Plugin for CruiserPLugin {
    fn build(&self, app: &mut App) {
        app.configure_loading_state(
            LoadingStateConfig::new(AppState::MainSceneLoading).load_collection::<CruiserAssets>(),
        )
        .add_event::<SpawnCruiserEvent>()
        .add_systems(ON_GAME_STARTED, cruiser_spawn_setup)
        .add_plugins(ReplaceMaterialPlugin::<Cruiser, _>::new(
            replace_with_toon_materials(ToonMaterial::default()),
        ))
        .add_systems(
            Update,
            (
                cruiser_shield_death,
                cruiser_shield_regenerate,
                cruiser_death.in_set(Set::ExplosionEvents),
                cruiser_spawn_bots,
                cruiser_shield_collisions,
                cruiser_scene_setup,
                cruiser_animation_start,
                cruiser_animations,
                cruiser_trail_update,
                cruiser_movement,
                spawn_cruiser_events,
                spawn_cruisers,
            )
                .run_if(game_running()),
        );
    }
}
