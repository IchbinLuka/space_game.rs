use std::f32::consts::FRAC_PI_2;
use std::time::Duration;

use bevy::animation::RepeatAnimation;
use bevy::ecs::system::{Command, EntityCommand, RunSystemOnce};
use bevy::pbr::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::scene::SceneInstance;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_mod_outline::OutlineBundle;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::{dynamics::Velocity, geometry::Collider};

use crate::components::despawn_after::DespawnTimer;
use crate::components::health::{DespawnOnDeath, Health, Shield};
use crate::entities::spaceship::bot::SpawnSquad;
use crate::materials::toon::{ApplyToonMaterial, ToonMaterial};
use crate::ui::enemy_indicator::SpawnEnemyIndicator;
use crate::ui::health_bar_3d::SpawnHealthBar;
use crate::utils::collisions::CRUISER_COLLISION_GROUP;
use crate::utils::misc::CollidingEntitiesExtension;
use crate::utils::sets::Set;

use crate::states::{game_running, AppState, ON_GAME_STARTED};
use crate::{components::colliders::VelocityColliderBundle, utils::materials::default_outline};

use super::bullet::{Bullet, BulletTarget, BulletType};
use super::explosion::ExplosionEvent;
use super::spaceship::IsBot;
use super::spaceship::SpaceshipCollisions;

#[derive(Component)]
pub struct Cruiser {
    enemy_spawn_cooldown: Timer,
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

const CRUISER_HITBOX_SIZE: Vec3 = Vec3::new(3.5, 3., 13.);
const COLLISION_GROUPS: CollisionGroups = CollisionGroups::new(CRUISER_COLLISION_GROUP, Group::ALL);

#[derive(Component)]
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
    }
}

pub struct SpawnCruiser {
    pub pos: Vec3,
}

impl Command for SpawnCruiser {
    fn apply(self, world: &mut World) {
        world.run_system_once_with(self.pos, spawn_cruiser)
    }
}

#[derive(Component, Default)]
pub struct AnimationRoot {
    pub player_entites: Vec<Entity>,
}

fn spawn_cruiser(In(pos): In<Vec3>, mut commands: Commands, assets: Res<CruiserAssets>) {
    let Vec3 { x, y, z } = CRUISER_HITBOX_SIZE;
    commands.spawn((
        SceneBundle {
            scene: assets.cruiser_model.clone(),
            transform: Transform::from_translation(pos),
            ..default()
        },
        ApplyToonMaterial::default(),
        VelocityColliderBundle {
            velocity: Velocity {
                linvel: Vec3 {
                    z: -2.0,
                    ..Vec3::ZERO
                },
                ..default()
            },
            collider: Collider::cuboid(x, y, z),
            ..default()
        },
        Cruiser {
            enemy_spawn_cooldown: Timer::from_seconds(10.0, TimerMode::Repeating),
        },
        BulletTarget {
            target_type: BulletType::Player,
            bullet_damage: Some(20.),
        },
        OutlineBundle {
            outline: default_outline(),
            ..default()
        },
        DespawnOnDeath,
        Health::new(100.0),
        SpaceshipCollisions {
            collision_damage: 5.0,
        },
        COLLISION_GROUPS,
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
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let shield = commands
        .spawn((
            CruiserShield,
            PbrBundle {
                mesh: meshes.add(
                    shape::UVSphere {
                        radius: 10.,
                        ..default()
                    }
                    .into(),
                ),
                material: materials.add(StandardMaterial {
                    base_color: Color::hex("2ae0ed0f").unwrap(),
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                }),
                transform: Transform::from_scale(Vec3 { z: 2., ..Vec3::ONE }),
                ..default()
            },
            NotShadowCaster,
            SpaceshipCollisions {
                collision_damage: 10.0,
            },
            Collider::ball(10.),
            CollidingEntities::default(),
            RigidBody::Fixed,
            ActiveCollisionTypes::KINEMATIC_STATIC,
            BulletTarget {
                target_type: BulletType::Player,
                bullet_damage: Some(10.0),
            },
            Health::new(100.0),
            Shield,
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
}

fn cruiser_setup(mut commands: Commands) {
    commands.add(SpawnCruiser {
        pos: Vec3::new(20., 0., -10.),
    })
}

fn cruiser_animation_start(
    query: Query<&AnimationRoot, (With<Cruiser>, Added<AnimationRoot>)>,
    mut animation_players: Query<&mut AnimationPlayer>,
    cruiser_assets: Res<CruiserAssets>,
) {
    for root in &query {
        for entity in &root.player_entites {
            let Ok(mut player) = animation_players.get_mut(*entity) else {
                continue;
            };
            info!("Playing animation");
            player.play(cruiser_assets.cruiser_animation.clone());
            player.set_repeat(RepeatAnimation::Never);
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
            if player.is_finished() {
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

fn cruiser_trails_setup(
    mut cruisers: Query<(&SceneInstance, Entity), (With<Cruiser>, Changed<SceneInstance>)>,
    names: Query<&Name, Without<Cruiser>>,
    mut animation_players: Query<Entity, With<AnimationPlayer>>,
    mut commands: Commands,
    scene_manager: Res<SceneSpawner>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut toon_materials: ResMut<Assets<ToonMaterial>>,
) {
    for (scene, cruiser) in &mut cruisers {
        if !scene_manager.instance_is_ready(**scene) {
            continue;
        }

        let mut animation_root = AnimationRoot::default();

        for entity in scene_manager.iter_instance_entities(**scene) {
            if let Ok(entity) = animation_players.get_mut(entity) {
                info!("Adding animation player");
                animation_root.player_entites.push(entity);
            }

            let Ok(name) = names.get(entity) else {
                continue;
            };
            if name.starts_with("exhaust.") {
                let trail = commands
                    .spawn((
                        MaterialMeshBundle {
                            material: toon_materials.add(ToonMaterial {
                                color: Color::hex("2ae0ed").unwrap(),
                                ..default()
                            }),
                            mesh: meshes.add(
                                shape::Cylinder {
                                    radius: 1.,
                                    height: CRUISER_TRAIL_LENGTH,
                                    ..default()
                                }
                                .into(),
                            ),
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
                commands.entity(entity).add_child(trail);
            }
        }

        commands.entity(cruiser).insert(animation_root);
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
        timer.0.tick(time.delta());

        if timer.0.just_finished() {
            commands.entity(entity).add(ActivateShield);
        }
        if timer.0.finished() {
            health.heal(10.0 * time.delta_seconds());
        }
    }
}

fn cruiser_death(
    query: Query<(&Health, &Transform), (With<Cruiser>, Changed<Health>)>,
    mut explosion_events: EventWriter<ExplosionEvent>,
) {
    for (health, transform) in &query {
        if health.is_dead() {
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
        }
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
            regen.0.reset();
        }
    }
}

fn cruiser_spawn_bots(
    mut commands: Commands,
    time: Res<Time>,
    mut cruisers: Query<(&Transform, &mut Cruiser)>,
    bots: Query<Entity, IsBot>,
) {
    const MAX_BOT_COUNT: usize = 5;

    if bots.iter().count() >= MAX_BOT_COUNT {
        return;
    }

    for (transform, mut spawn_cooldown) in &mut cruisers {
        spawn_cooldown.enemy_spawn_cooldown.tick(time.delta());

        if spawn_cooldown.enemy_spawn_cooldown.just_finished() {
            commands.add(SpawnSquad {
                squad_size: 3,
                leader_pos: transform.translation,
            });
        }
    }
}

pub struct CruiserPLugin;

impl Plugin for CruiserPLugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, CruiserAssets>(AppState::MainSceneLoading)
            .add_systems(ON_GAME_STARTED, cruiser_setup)
            .add_systems(
                Update,
                (
                    cruiser_shield_death,
                    cruiser_shield_regenerate,
                    cruiser_death.in_set(Set::ExplosionEvents),
                    cruiser_spawn_bots,
                    cruiser_shield_collisions,
                    cruiser_trails_setup,
                    cruiser_animation_start,
                    cruiser_animations,
                    cruiser_trail_update,
                )
                    .run_if(game_running()),
            );
    }
}
