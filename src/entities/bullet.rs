use std::time::Duration;

use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_mod_outline::{OutlineVolume, OutlineBundle};
use bevy_rapier3d::prelude::*;

use crate::{AppState, components::gravity::GravityAffected};

use super::player::Player;

#[derive(Component)]
pub struct Bullet {
    pub spawn_time: Duration, 
    pub relative_speed: Vec3, 
}
pub const BULLET_COLLISION_GROUP: Group = Group::GROUP_2;

const BULLET_GROUP: CollisionGroups = CollisionGroups::new(Group::GROUP_1, Group::GROUP_2);

const BULLET_CORNER_1: Vec3 = Vec3::new(0.03, 0.03, 0.6);
const BULLET_CORNER_2: Vec3 = Vec3::new(-0.03, -0.03, 0.0);

fn bullet_setup(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let bullet_mesh = meshes.add(shape::Box::from_corners(
        BULLET_CORNER_1,
        BULLET_CORNER_2,
    ).into());
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

    commands.insert_resource(LastBulletInfo::new());
}

fn bullet_shoot(
    keyboard_input: Res<Input<KeyCode>>, 
    time: Res<Time>,
    query: Query<(&Transform, &Velocity, With<Player>)>, 
    mut commands: Commands,
    bullet_res: Res<BulletResource>,
    assets: Res<BulletAssets>, 
    mut last_bullet_info: ResMut<LastBulletInfo>,
) {
    if !last_bullet_info.timer.finished() {
        last_bullet_info.timer.tick(time.delta());
        return;
    }
    let bullet_size = BULLET_CORNER_1 - BULLET_CORNER_2;
    for (transform, velocity, _) in &query {
        if keyboard_input.pressed(KeyCode::Space) {
            // If finished, the timer should wait for the player to shoot before ticking again 
            last_bullet_info.timer.tick(time.delta());
            let side = last_bullet_info.side;

            let pos = transform.translation + transform.rotation.mul_vec3(side.into());
            let mut bullet_transform = Transform::from_translation(pos);
            
            bullet_transform.rotate(transform.rotation);
            // bullet_transform.rotate_x(-PI * 0.5);
            debug!("Spawning bullet");
            commands.spawn((
                PbrBundle {
                    mesh: bullet_res.bullet_mesh.clone(),
                    material: bullet_res.bullet_material.clone(),
                    transform: bullet_transform,
                    ..default()
                }, 
                Bullet {
                    spawn_time: time.elapsed(), 
                    relative_speed: velocity.linvel,
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
                    linvel: transform.forward().normalize() * 20.0 + velocity.linvel, 
                    ..default()
                }, 
                CollidingEntities::default(), 
            ));
            commands.spawn(
                AudioBundle {
                    source: assets.test_sound.clone(), 
                    ..default()
                }
            );
            
            last_bullet_info.side = side.other();
        }
    }
}

fn bullet_despawn(
    time: Res<Time>,
    mut commands: Commands,
    query: Query<(Entity, &Bullet)>,
) {
    const BULLET_LIFETIME: Duration = Duration::from_secs(5);
    for (entity, bullet) in &query {
        if time.elapsed() - bullet.spawn_time > BULLET_LIFETIME {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn bullet_collision(
    query: Query<(Entity, &Bullet, &CollidingEntities)>,
    mut commands: Commands,
) {
    for (entity, _, colliding_entities) in &query {
        if colliding_entities.is_empty() { continue; }
        debug!("Bullet collided with something");
        commands.entity(entity).despawn_recursive();
    }
}

fn bullet_rotation_correction(
    mut query: Query<(&mut Transform, &Velocity, &Bullet)>,
) {
    for (mut transform, vel, bullet) in &mut query {
        let rotation = Quat::from_rotation_arc(transform.forward(), (vel.linvel - bullet.relative_speed).normalize());
        transform.rotate(rotation);
    }
}

#[derive(Resource)]
struct BulletResource {
    bullet_mesh: Handle<Mesh>,
    bullet_material: Handle<StandardMaterial>,
}

#[derive(Clone, Copy)]
enum BulletSide {
    Left,
    Right,
}

impl BulletSide {
    const LEFT_POSITION: Vec3 = Vec3::new(-0.6, 0.0, -0.44);
    const RIGHT_POSITION: Vec3 = Vec3::new(0.6, 0.0, -0.44);

    fn other(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

impl From<BulletSide> for Vec3 {
    fn from(value: BulletSide) -> Self {
        match value {
            BulletSide::Left => BulletSide::LEFT_POSITION,
            BulletSide::Right => BulletSide::RIGHT_POSITION,
        }
    }
}

impl Default for BulletSide {
    fn default() -> Self {
        Self::Left
    }
}


#[derive(Resource)]
struct LastBulletInfo {
    side: BulletSide,
    timer: Timer,
}

impl LastBulletInfo {
    fn new() -> Self {
        Self {
            side: BulletSide::default(),
            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
        }
    }
}

#[derive(AssetCollection, Resource)]
struct BulletAssets {
    #[asset(path = "fire_sound.ogg")]
    test_sound: Handle<AudioSource>
}

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_collection_to_loading_state::<_, BulletAssets>(AppState::Loading)
            .add_systems(OnEnter(AppState::Running), bullet_setup)
            .add_systems(Update, (
                bullet_shoot, 
                bullet_despawn, 
                bullet_collision,
                bullet_rotation_correction,
            ).run_if(in_state(AppState::Running)));
    }
}