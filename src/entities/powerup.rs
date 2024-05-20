use std::{f32::consts::FRAC_PI_2, time::Duration};

use bevy::{ecs::system::Command, prelude::*};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{
        config::{ConfigureLoadingState, LoadingStateConfig},
        LoadingStateAppExt,
    },
};
use bevy_mod_outline::OutlineBundle;
use bevy_rapier3d::{
    dynamics::RigidBody,
    geometry::{ActiveCollisionTypes, Collider, CollidingEntities, CollisionGroups},
};
use rand::{rngs::ThreadRng, Rng};

use crate::{
    components::{despawn_after::DespawnTimer, health::Health},
    materials::{
        shield::{ShieldBundle, ShieldMaterial},
        toon::{replace_with_toon_materials, ToonMaterial},
    },
    states::{AppState, DespawnOnCleanup},
    utils::{
        materials::default_outline, misc::CollidingEntitiesExtension, scene::ReplaceMaterialPlugin,
    },
};

use super::{
    bullet::{BulletTarget, BulletType},
    spaceship::{
        player::{Player, PlayerInventory},
        SpaceshipBundle,
    },
};

#[derive(Component, Clone, Copy)]
pub enum PowerUp {
    Shield,
    Bomb,
}

#[derive(Component)]
pub struct PlayerShield;

#[derive(Component)]
pub struct ShieldEnabled;

#[derive(Bundle)]
struct PowerupBundle {
    colliding_entities: CollidingEntities,
    collider: Collider,
    rigid_body: RigidBody,
    despawn_on_cleanup: DespawnOnCleanup,
    despawn_timer: DespawnTimer,
    active_collision_types: ActiveCollisionTypes,
    collision_groups: CollisionGroups,
    outline_bundle: OutlineBundle,
}

impl Default for PowerupBundle {
    fn default() -> Self {
        Self {
            colliding_entities: default(),
            collider: default(),
            rigid_body: RigidBody::Fixed,
            despawn_on_cleanup: default(),
            despawn_timer: DespawnTimer::new(Duration::from_secs(20)),
            active_collision_types: ActiveCollisionTypes::KINEMATIC_STATIC,
            collision_groups: SpaceshipBundle::COLLISION_GROUPS,
            outline_bundle: OutlineBundle {
                outline: default_outline(),
                ..default()
            },
        }
    }
}

pub struct SpawnPowerup {
    pub powerup: PowerUp,
    pub pos: Vec3,
}

impl SpawnPowerup {
    pub fn random(pos: Vec3, rng: &mut ThreadRng) -> Self {
        const POWERUPS: [PowerUp; 2] = [PowerUp::Shield, PowerUp::Bomb];
        let powerup = POWERUPS[rng.gen_range(0..POWERUPS.len())];
        Self { powerup, pos }
    }
}

impl Command for SpawnPowerup {
    fn apply(self, world: &mut World) {
        let Some(assets) = world.get_resource::<PowerUpAssets>() else {
            error!("Could not spawn powerup, assets are not loaded");
            return;
        };

        match self.powerup {
            PowerUp::Shield => {
                world.spawn((
                    PowerupBundle {
                        collider: Collider::ball(3.0),
                        ..default()
                    },
                    SceneBundle {
                        transform: Transform {
                            translation: self.pos,
                            rotation: Quat::from_rotation_x(-FRAC_PI_2),
                            scale: Vec3::splat(0.5),
                        },
                        scene: assets.shield.clone(),
                        ..default()
                    },
                    PowerUp::Shield,
                ));
            }
            PowerUp::Bomb => {
                world.spawn((
                    PowerupBundle {
                        collider: Collider::ball(3.0),
                        ..default()
                    },
                    PowerUp::Bomb,
                    SceneBundle {
                        scene: assets.bomb.clone(),
                        transform: Transform::from_translation(self.pos),
                        ..default()
                    },
                ));
            }
        }
    }
}

fn shield_death(
    mut commands: Commands,
    mut removed_shields: RemovedComponents<PlayerShield>,
    player: Query<Entity, With<Player>>,
) {
    if removed_shields.read().next().is_none() {
        return;
    }
    for player_entity in player.iter() {
        commands.entity(player_entity).remove::<ShieldEnabled>();
    }
}

fn powerup_collisions(
    powerups: Query<(&CollidingEntities, &PowerUp, Entity)>,
    player: Query<Entity, With<Player>>,
    player_shields: Query<(), With<PlayerShield>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ShieldMaterial>>,
    mut player_inventory: ResMut<PlayerInventory>,
) {
    for (colliding_entities, powerup, entity) in powerups.iter() {
        let Some(player_entity) = colliding_entities.filter_fulfills_query(&player).next() else {
            continue;
        };
        match powerup {
            PowerUp::Shield => {
                if player_shields.iter().next().is_some() {
                    continue;
                }
                let shield = commands
                    .spawn((
                        ShieldBundle {
                            material_mesh: MaterialMeshBundle {
                                mesh: meshes.add(Sphere { radius: 2. }),
                                material: materials.add(ShieldMaterial::default()),
                                transform: Transform::from_scale(Vec3 {
                                    z: 1.3,
                                    ..Vec3::ONE
                                }),
                                ..default()
                            },
                            collider: Collider::ball(2.),
                            rigid_body: RigidBody::Fixed,
                            active_collision_types: ActiveCollisionTypes::KINEMATIC_STATIC,
                            bullet_target: BulletTarget {
                                target_type: BulletType::Bot,
                                bullet_damage: Some(10.0),
                            },
                            health: Health::new(100.0),
                            ..default()
                        },
                        PlayerShield,
                        DespawnTimer::new(Duration::from_secs(20)),
                        SpaceshipBundle::COLLISION_GROUPS,
                    ))
                    .id();
                commands
                    .entity(player_entity)
                    .add_child(shield)
                    .insert(ShieldEnabled);
            }
            PowerUp::Bomb => {
                player_inventory.bombs += 1;
            }
        }
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(AssetCollection, Resource)]
pub struct PowerUpAssets {
    #[asset(path = "shield.glb#Scene0")]
    pub shield: Handle<Scene>,
    #[asset(path = "bomb.glb#Scene0")]
    pub bomb: Handle<Scene>,
}

pub struct PowerupPlugin;
impl Plugin for PowerupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ReplaceMaterialPlugin::<PowerUp, _>::new(
            replace_with_toon_materials(ToonMaterial {
                filter_scale: 0.0,
                ..default()
            }),
        ))
        .configure_loading_state(
            LoadingStateConfig::new(AppState::MainSceneLoading).load_collection::<PowerUpAssets>(),
        )
        .add_systems(Update, (powerup_collisions, shield_death));
    }
}
