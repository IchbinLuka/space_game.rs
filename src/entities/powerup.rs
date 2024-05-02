use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier3d::{
    dynamics::RigidBody,
    geometry::{ActiveCollisionTypes, Collider, CollidingEntities},
};

use crate::{
    components::{despawn_after::DespawnTimer, health::Health},
    materials::shield::{ShieldBundle, ShieldMaterial},
    states::ON_GAME_STARTED,
    utils::misc::CollidingEntitiesExtension,
};

use super::{
    bullet::{BulletTarget, BulletType},
    spaceship::{player::Player, SpaceshipBundle},
};

#[derive(Component)]
pub enum PowerUp {
    Shield,
}

#[derive(Component)]
pub struct PlayerShield;

fn powerup_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        CollidingEntities::default(),
        PowerUp::Shield,
        Collider::capsule(Vec3::Y * -3.5, Vec3::Y * 3.5, 5.0),
        RigidBody::Fixed,
        ActiveCollisionTypes::KINEMATIC_STATIC,
        BulletTarget {
            target_type: BulletType::Player,
            bullet_damage: Some(10.0),
        },
        Health::new(100.0),
        SpaceshipBundle::COLLISION_GROUPS,
        PbrBundle {
            transform: Transform::from_translation(Vec3::new(30., 0., 0.)),
            mesh: meshes.add(Capsule3d::new(5.0, 7.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::RED,
                unlit: true,
                ..default()
            }),
            ..default()
        },
    ));
}

fn powerup_collisions(
    powerups: Query<(&CollidingEntities, &PowerUp, Entity)>,
    player: Query<Entity, With<Player>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ShieldMaterial>>,
) {
    for (colliding_entities, powerup, entity) in powerups.iter() {
        let Some(player_entity) = colliding_entities.filter_fulfills_query(&player).next() else {
            continue;
        };
        match powerup {
            PowerUp::Shield => {
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
                commands.entity(player_entity).add_child(shield);
            }
        }
        commands.entity(entity).despawn_recursive();
    }
}

pub struct PowerupPlugin;
impl Plugin for PowerupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(ON_GAME_STARTED, powerup_setup)
            .add_systems(Update, powerup_collisions);
    }
}
