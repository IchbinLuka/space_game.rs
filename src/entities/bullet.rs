use std::time::Duration;

use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_audio::VolumeLevel;
use bevy_mod_outline::{OutlineBundle, OutlineVolume};
use bevy_rapier3d::prelude::*;

use crate::{components::gravity::GravityAffected, utils::sets::Set, AppState};

use super::spaceship::player::Player;

#[derive(Component)]
pub struct Bullet {
    pub spawn_time: Duration,
    pub relative_speed: Vec3,
    pub origin: Entity,
}

#[derive(Event)]
pub struct BulletSpawnEvent {
    pub position: Transform,
    pub entity_velocity: Velocity,
    pub direction: Vec3,
    pub entity: Entity,
}

pub const BULLET_COLLISION_GROUP: Group = Group::GROUP_2;

const BULLET_GROUP: CollisionGroups = CollisionGroups::new(Group::GROUP_1, Group::GROUP_2);

const BULLET_CORNER_1: Vec3 = Vec3::new(0.03, 0.03, 0.6);
const BULLET_CORNER_2: Vec3 = Vec3::new(-0.03, -0.03, 0.0);

const BULLET_SPEED: f32 = 40.0;

fn bullet_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let bullet_mesh = meshes.add(shape::Box::from_corners(BULLET_CORNER_1, BULLET_CORNER_2).into());
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
    player_query: Query<&Transform, With<Player>>,
    bullet_res: Res<BulletResource>,
    assets: Res<BulletAssets>,
    time: Res<Time>,
) {
    let bullet_size = BULLET_CORNER_1 - BULLET_CORNER_2;

    let Ok(player) = player_query.get_single() else {
        warn!("No player found");
        return;
    };

    for event in events.read() {
        // let pos = transform.translation + transform.rotation.mul_vec3(side.into());
        let mut bullet_transform = Transform::from_translation(event.position.translation);

        let rotation =
            Quat::from_rotation_arc(bullet_transform.forward(), event.direction.normalize());

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
                origin: event.entity,
            },
            OutlineBundle {
                outline: OutlineVolume {
                    colour: Color::RED,
                    width: 1.0,
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
            CollidingEntities::default(),
        ));
        commands.spawn(AudioBundle {
            source: assets.test_sound.clone(),
            settings: PlaybackSettings {
                volume: bevy_audio::Volume::Relative(VolumeLevel::new(f32::min(
                    1.0,
                    40.0 / event
                        .position
                        .translation
                        .distance_squared(player.translation),
                ))),
                ..default()
            },
        });
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

fn bullet_collision(query: Query<(Entity, &Bullet, &CollidingEntities)>, mut commands: Commands) {
    for (entity, bullet, colliding_entities) in &query {
        if colliding_entities.is_empty() {
            continue;
        }
        if colliding_entities.iter().all(|e| e == bullet.origin) {
            continue;
        }
        debug!("Bullet collided with something");
        commands.entity(entity).despawn_recursive();
    }
}

fn bullet_rotation_correction(mut query: Query<(&mut Transform, &Velocity, &Bullet)>) {
    for (mut transform, vel, bullet) in &mut query {
        let rotation = Quat::from_rotation_arc(
            transform.forward(),
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

#[derive(AssetCollection, Resource)]
struct BulletAssets {
    #[asset(path = "fire_sound.ogg")]
    test_sound: Handle<AudioSource>,
}

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, BulletAssets>(AppState::Loading)
            .add_systems(OnEnter(AppState::Running), bullet_setup)
            .add_systems(
                Update,
                (
                    bullet_despawn,
                    bullet_collision,
                    bullet_rotation_correction,
                    bullet_spawn.after(Set::BulletEvents),
                )
                    .run_if(in_state(AppState::Running)),
            )
            .add_event::<BulletSpawnEvent>();
    }
}
