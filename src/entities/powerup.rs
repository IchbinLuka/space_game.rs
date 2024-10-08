use std::{f32::consts::FRAC_PI_2, time::Duration};

use bevy::{
    ecs::world::Command,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::render_resource::Face,
};
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
    turret::Turret,
};

#[derive(Component, Clone, Copy)]
pub enum PowerUp {
    Shield,
    Bomb,
    Turret,
}

#[derive(Component)]
pub struct PlayerShield;

#[derive(Component)]
pub struct ShieldEnabled;

#[derive(Component)]
pub struct TurretItem;

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
            collider: Collider::ball(3.0),
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
        const POWERUPS: [PowerUp; 3] = [PowerUp::Shield, PowerUp::Bomb, PowerUp::Turret];
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

        let Some(res) = world.get_resource::<PowerUpRes>() else {
            error!("Could not spawn powerup, resources are not loaded");
            return;
        };

        match self.powerup {
            PowerUp::Shield => {
                world.spawn((
                    PowerupBundle::default(),
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
                    PowerupBundle::default(),
                    PowerUp::Bomb,
                    SceneBundle {
                        scene: assets.bomb.clone(),
                        transform: Transform::from_translation(self.pos),
                        ..default()
                    },
                ));
            }
            PowerUp::Turret => {
                let scene = assets.turret.clone();

                world
                    .spawn((
                        PowerupBundle::default(),
                        PowerUp::Turret,
                        NotShadowCaster,
                        NotShadowReceiver,
                        MaterialMeshBundle {
                            transform: Transform::from_translation(self.pos),
                            mesh: res.turret_halo_mesh.clone(),
                            material: res.turret_halo.clone(),
                            ..default()
                        },
                    ))
                    .with_children(|c| {
                        c.spawn((
                            SceneBundle {
                                scene,
                                transform: Transform {
                                    scale: Vec3::splat(0.7),
                                    ..default()
                                },
                                ..default()
                            },
                            TurretItem,
                        ));
                    });
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
    powerup_res: Res<PowerUpRes>,
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
                                mesh: powerup_res.shield_mesh.clone(),
                                material: powerup_res.shield_material.clone(),
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
                if player_inventory.bombs < 3 {
                    player_inventory.bombs += 1;
                } else {
                    continue;
                }
            }
            PowerUp::Turret => {
                if player_inventory.turrets < 3 {
                    player_inventory.turrets += 1;
                } else {
                    continue;
                }
            }
        }
        commands.entity(entity).despawn_recursive();
    }
}

fn powerup_setup(
    mut commands: Commands,
    mut shield_materials: ResMut<Assets<ShieldMaterial>>,
    mut toon_materials: ResMut<Assets<ToonMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.insert_resource(PowerUpRes {
        shield_mesh: meshes.add(Sphere { radius: 2. }),
        shield_material: shield_materials.add(ShieldMaterial::default()),
        turret_halo: toon_materials.add(ToonMaterial {
            cull_mode: Some(Face::Front),
            disable_outline: true,
            color: Color::WHITE,
            ..default()
        }),
        turret_halo_mesh: meshes.add(Sphere { radius: 1. }),
    });
}

#[derive(AssetCollection, Resource)]
pub struct PowerUpAssets {
    #[asset(path = "shield.glb#Scene0")]
    pub shield: Handle<Scene>,
    #[asset(path = "bomb.glb#Scene0")]
    pub bomb: Handle<Scene>,
    #[asset(path = "turret.glb#Scene0")]
    pub turret: Handle<Scene>,
}

#[derive(Resource)]
struct PowerUpRes {
    pub shield_mesh: Handle<Mesh>,
    pub shield_material: Handle<ShieldMaterial>,
    pub turret_halo: Handle<ToonMaterial>,
    pub turret_halo_mesh: Handle<Mesh>,
}

pub struct PowerupPlugin;
impl Plugin for PowerupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ReplaceMaterialPlugin::<PowerUp, _>::new(replace_with_toon_materials(ToonMaterial {
                disable_outline: true,
                ..default()
            })),
            ReplaceMaterialPlugin::<Turret, _>::new(replace_with_toon_materials(ToonMaterial {
                disable_outline: true,
                ..default()
            })),
        ))
        .configure_loading_state(
            LoadingStateConfig::new(AppState::MainSceneLoading).load_collection::<PowerUpAssets>(),
        )
        .add_systems(Startup, powerup_setup)
        .add_systems(Update, (powerup_collisions, shield_death));
    }
}
