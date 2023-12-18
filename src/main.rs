use std::default;

use bevy::prelude::*;
use bevy_asset_loader::{loading_state::{LoadingStateAppExt, LoadingState}, asset_collection::AssetCollection};
use entities::{camera::CameraComponentPlugin, player::PlayerPlugin, bullet::BulletPlugin};

mod entities;


#[derive(Component)]
pub struct Movement {
    pub vel: Vec3,
    pub acc: Vec3,
    pub max_speed: Option<f32>,
    pub friction: f32,
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            vel: Vec3::ZERO,
            acc: Vec3::ZERO,
            max_speed: None,
            friction: 0.0,
        }
    }
}

fn movement_system(
    time: Res<Time>, 
    mut query: Query<(&mut Transform, &mut Movement)>
) {
    for (mut transform, mut movement) in &mut query {
        let acc = movement.acc;
        movement.vel += acc * time.delta_seconds();
        transform.translation += movement.vel * time.delta_seconds();

        if let Some(max_speed) = movement.max_speed {
            if movement.vel.length() > max_speed {
                movement.vel = movement.vel.normalize() * max_speed;
            }
        }
        let friction = movement.friction;
        movement.vel *= 1.0 - friction * time.delta_seconds();
    }
}


pub struct ScenePlugin3D;

impl Plugin for ScenePlugin3D {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, scene_setup_3d)
            .add_systems(Update, movement_system)
            .add_plugins(CameraComponentPlugin)
            .add_plugins(PlayerPlugin);
    }
}

fn scene_setup_3d(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Circle::new(4.0).into()),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

#[derive(AssetCollection, Resource)]
struct GameAssets {
    #[asset(path = "spaceship.png")]
    spaceship: Handle<Image>
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum AppState {
    #[default]
    Loading, 
    Running
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ScenePlugin3D, BulletPlugin))
        .add_state::<AppState>()
        .add_loading_state(
            LoadingState::new(AppState::Loading)
                .continue_to_state(AppState::Running)
        )
        .add_collection_to_loading_state::<_, GameAssets>(AppState::Loading)
        .run();
}
