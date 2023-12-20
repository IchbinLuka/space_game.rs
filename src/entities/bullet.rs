use std::{f32::consts::PI, time::Duration};

use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};

use crate::{Movement, AppState};

use super::player::Player;

#[derive(Component)]
pub struct Bullet {
    pub spawn_time: Duration
}

fn bullet_setup(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let bullet_mesh = meshes.add(shape::Circle { radius: 0.1, vertices: 20 }.into());
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
    query: Query<(&Transform, &Movement, With<Player>)>, 
    mut commands: Commands,
    bullet_res: Res<BulletResource>,
    assets: Res<BulletAssets>, 
    mut last_bullet_info: ResMut<LastBulletInfo>,
) {
    last_bullet_info.timer.tick(time.delta());
    if !last_bullet_info.timer.finished() {
        return;
    }
    for (transform, movement, _) in &query {
        if keyboard_input.pressed(KeyCode::Space) {
            let side = last_bullet_info.side;

            let pos = transform.translation + transform.rotation.mul_vec3(side.into());
            let mut bullet_transform = Transform::from_translation(pos);
            
            
            bullet_transform.rotate_x(-PI * 0.5);
            debug!("Spawning bullet");
            commands.spawn((
                PbrBundle {
                    mesh: bullet_res.bullet_mesh.clone(),
                    material: bullet_res.bullet_material.clone(),
                    transform: bullet_transform,
                    ..default()
                }, 
                Movement {
                    vel: transform.forward().normalize() * 20.0 + movement.vel,
                    ..default()
                }, 
                Bullet {
                    spawn_time: time.elapsed()
                },
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
    const LEFT_POSITION: Vec3 = Vec3::new(-0.6, 0.0, -0.24);
    const RIGHT_POSITION: Vec3 = Vec3::new(0.6, 0.0, -0.24);

    fn other(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

impl Into<Vec3> for BulletSide {
    fn into(self) -> Vec3 {
        match self {
            BulletSide::Left => Self::LEFT_POSITION,
            BulletSide::Right => Self::RIGHT_POSITION,
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
                bullet_despawn
            ).run_if(in_state(AppState::Running)));
    }
}