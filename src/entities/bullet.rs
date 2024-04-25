use std::time::Duration;

use bevy::prelude::*;
use bevy_mod_outline::{OutlineBundle, OutlineVolume};
use bevy_rapier3d::prelude::*;

use crate::{
    components::{gravity::GravityAffected, health::Health},
    states::{game_running, DespawnOnCleanup, ON_GAME_STARTED},
    utils::{collisions::BULLET_COLLISION_GROUP, sets::Set},
};

use super::{explosion::ExplosionEvent, spaceship::player::LastHit};

#[derive(Component)]
pub struct Bullet {
    pub spawn_time: Duration,
    pub relative_speed: Vec3,
    pub bullet_type: BulletType,
}

#[derive(Component)]
pub struct BulletTarget {
    pub target_type: BulletType,
    pub bullet_damage: Option<f32>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BulletType {
    Player,
    Bot,
    Both,
}

#[derive(Event)]
pub struct BulletSpawnEvent {
    pub position: Transform,
    pub entity_velocity: Velocity,
    pub direction: Vec3,
    pub bullet_type: BulletType,
}

const BULLET_GROUP: CollisionGroups = CollisionGroups::new(BULLET_COLLISION_GROUP, Group::ALL);

const BULLET_CORNER_1: Vec3 = Vec3::new(0.04, 0.04, 0.7);
const BULLET_CORNER_2: Vec3 = Vec3::new(-0.04, -0.04, 0.0);

const BULLET_SPEED: f32 = 40.0;

fn bullet_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let bullet_mesh = meshes.add(Cuboid::from_corners(BULLET_CORNER_1, BULLET_CORNER_2));
    let bullet_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: Color::WHITE,
        unlit: true,
        diffuse_transmission: 1.0,
        ..default()
    });

    commands.insert_resource(BulletResource {
        bullet_mesh,
        bullet_material,
    });
}

fn bullet_spawn(
    mut commands: Commands,
    mut events: EventReader<BulletSpawnEvent>,
    bullet_res: Res<BulletResource>,
    time: Res<Time>,
) {
    let bullet_size = BULLET_CORNER_1 - BULLET_CORNER_2;

    for event in events.read() {
        // let pos = transform.translation + transform.rotation.mul_vec3(side.into());
        let mut bullet_transform = Transform::from_translation(event.position.translation);

        let rotation =
            Quat::from_rotation_arc(*bullet_transform.forward(), event.direction.normalize());

        bullet_transform.rotate(rotation);

        commands.spawn((
            PbrBundle {
                mesh: bullet_res.bullet_mesh.clone(),
                material: bullet_res.bullet_material.clone(),
                transform: bullet_transform,
                ..default()
            },
            Bullet {
                spawn_time: time.elapsed(),
                relative_speed: event.entity_velocity.linvel,
                bullet_type: event.bullet_type,
            },
            OutlineBundle {
                outline: OutlineVolume {
                    colour: Color::RED,
                    width: 2.0,
                    visible: true,
                },
                ..default()
            },
            Collider::cuboid(bullet_size.x, bullet_size.y, bullet_size.z),
            BULLET_GROUP,
            ActiveEvents::COLLISION_EVENTS,
            ActiveHooks::FILTER_INTERSECTION_PAIR,
            RigidBody::KinematicVelocityBased,
            Sensor,
            GravityAffected,
            Velocity {
                linvel: event.direction.normalize() * BULLET_SPEED + event.entity_velocity.linvel,
                ..default()
            },
            DespawnOnCleanup,
            CollidingEntities::default(),
        ));
        // TODO: play sound
    }
}

fn bullet_despawn(time: Res<Time>, mut commands: Commands, query: Query<(Entity, &Bullet)>) {
    const BULLET_LIFETIME: Duration = Duration::from_secs(5);
    for (entity, bullet) in &query {
        if time.elapsed() - bullet.spawn_time > BULLET_LIFETIME {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn bullet_collision(
    query: Query<(Entity, &Bullet, &CollidingEntities, &Transform)>,
    mut bullet_target_query: Query<(&BulletTarget, Option<&mut Health>, Option<&mut LastHit>)>,
    mut commands: Commands,
    mut explosions: EventWriter<ExplosionEvent>,
    time: Res<Time>,
) {
    for (entity, bullet, colliding_entities, transform) in &query {
        if colliding_entities.is_empty() {
            continue;
        }

        let mut despawn: bool = false;

        for entity in colliding_entities.iter() {
            let Ok((bullet_target, health, last_hit)) = bullet_target_query.get_mut(entity) else {
                continue;
            };

            if bullet_target.target_type != BulletType::Both
                && bullet_target.target_type != bullet.bullet_type
            {
                continue;
            }

            if let Some(damage) = bullet_target.bullet_damage
                && let Some(mut health) = health
            {
                health.take_damage(damage);
            }

            if let Some(mut last_hit) = last_hit {
                last_hit.0 = Some(time.elapsed_seconds());
            }

            despawn = true;
        }

        if despawn {
            explosions.send(ExplosionEvent {
                position: transform.translation,
                ..default()
            });

            debug!("Bullet collided with something");
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn bullet_rotation_correction(mut query: Query<(&mut Transform, &Velocity, &Bullet)>) {
    for (mut transform, vel, bullet) in &mut query {
        let rotation = Quat::from_rotation_arc(
            *transform.forward(),
            (vel.linvel - bullet.relative_speed).normalize(),
        );
        transform.rotate(rotation);
    }
}

#[derive(Resource)]
struct BulletResource {
    bullet_mesh: Handle<Mesh>,
    bullet_material: Handle<StandardMaterial>,
}

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(ON_GAME_STARTED, bullet_setup)
            .add_systems(
                Update,
                (
                    bullet_despawn,
                    bullet_collision,
                    bullet_rotation_correction,
                    bullet_spawn.after(Set::BulletEvents),
                )
                    .run_if(game_running()),
            )
            .add_event::<BulletSpawnEvent>();
    }
}
