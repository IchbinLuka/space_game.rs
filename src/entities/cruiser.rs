use bevy::ecs::system::EntityCommand;
use bevy::prelude::*;
use bevy::scene::SceneInstance;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_mod_outline::OutlineBundle;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::{dynamics::Velocity, geometry::Collider};

use crate::components::health::{DespawnOnDeath, Health, Shield};
use crate::ui::enemy_indicator::SpawnEnemyIndicator;
use crate::ui::health_bar_3d::SpawnHealthBar;
use crate::utils::collisions::CRUISER_COLLISION_GROUP;
use crate::utils::misc::CollidingEntitiesExtension;
use crate::utils::sets::Set;
use crate::OutlineMaterial;
use crate::{
    components::colliders::VelocityColliderBundle, utils::materials::default_outline, AppState,
};

use super::bullet::{Bullet, BulletTarget, BulletType};
use super::explosion::ExplosionEvent;
use super::spaceship::bot::{BotState, SpawnBot};
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

#[derive(AssetCollection, Resource)]
struct CruiserAssets {
    #[asset(path = "cruiser.glb#Scene0")]
    pub cruiser_model: Handle<Scene>,
}

const CRUISER_HITBOX_SIZE: Vec3 = Vec3::new(3.5, 3., 13.);

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

fn cruiser_setup(
    mut commands: Commands,
    assets: Res<CruiserAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    const COLLISION_GROUPS: CollisionGroups =
        CollisionGroups::new(CRUISER_COLLISION_GROUP, Group::ALL);
    let Vec3 { x, y, z } = CRUISER_HITBOX_SIZE;
    let entity = commands
        .spawn((
            SceneBundle {
                scene: assets.cruiser_model.clone(),
                transform: Transform::from_translation(Vec3 {
                    z: 20.0,
                    ..Vec3::ZERO
                }),
                ..default()
            },
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
                enemy_spawn_cooldown: Timer::from_seconds(5.0, TimerMode::Repeating),
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
        ))
        .id();

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

    commands.entity(entity).add_child(shield);

    commands.add(SpawnHealthBar {
        entity,
        scale: 1.,
        offset: Vec2::ZERO,
        shield_entity: Some(shield),
    });
    commands.add(SpawnEnemyIndicator { enemy: entity });
}

#[derive(Component)]
struct UpdatedMaterials;

fn cruiser_material_setup(
    query: Query<
        (&SceneInstance, Entity),
        (
            Changed<SceneInstance>,
            With<Cruiser>,
            Without<UpdatedMaterials>,
        ),
    >,
    mut commands: Commands,
    scene_manager: Res<SceneSpawner>,
    mut materials: ResMut<Assets<OutlineMaterial>>,
    standard_materials: ResMut<Assets<StandardMaterial>>,
    standard_material_query: Query<&Handle<StandardMaterial>>,
) {
    for (scene_instance, entity) in &query {
        if scene_manager.instance_is_ready(**scene_instance) {
            for entity in scene_manager.iter_instance_entities(**scene_instance) {
                if let Ok(handle) = standard_material_query.get(entity) {
                    let Some(material) = standard_materials.get(handle) else {
                        continue;
                    };

                    let outline_material = materials.add(OutlineMaterial {
                        color: material.base_color,
                        ..default()
                    });

                    commands
                        .entity(entity)
                        .remove::<Handle<StandardMaterial>>()
                        .insert(outline_material);
                }
            }
        }
        commands.entity(entity).insert(UpdatedMaterials);
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

#[derive(Component)]
struct EnemySpawnCooldown(pub Timer);

impl Default for EnemySpawnCooldown {
    fn default() -> Self {
        Self(Timer::from_seconds(5.0, TimerMode::Repeating))
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
            commands.add(SpawnBot {
                pos: transform.translation,
                initial_state: BotState::Chasing,
            });
        }
    }
}

pub struct CruiserPLugin;

impl Plugin for CruiserPLugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, CruiserAssets>(AppState::MainSceneLoading)
            .add_systems(OnEnter(AppState::MainScene), cruiser_setup)
            .add_systems(
                Update,
                (
                    cruiser_shield_death,
                    cruiser_shield_regenerate,
                    cruiser_death.in_set(Set::ExplosionEvents),
                    cruiser_spawn_bots,
                    cruiser_shield_collisions,
                    cruiser_material_setup,
                )
                    .run_if(in_state(AppState::MainScene)),
            );
    }
}
