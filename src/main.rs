#![allow(clippy::type_complexity)] // Query types can be really complex
#![feature(let_chains)]

use bevy::{log::LogPlugin, prelude::*};
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};
use bevy_mod_outline::{AutoGenerateOutlineNormalsPlugin, OutlinePlugin};
use bevy_obj::ObjPlugin;
use bevy_rapier3d::prelude::*;
use bevy_round_ui::prelude::RoundUiPlugin;
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};
use bevy_toon_shader::{ToonShaderPlugin, ToonShaderSun};
use components::ComponentsPlugin;
use entities::{asteroid::AsteroidSpawnEvent, EntitiesPlugin};
use particles::ParticlesPlugin;
use rand::Rng;
use ui::UIPlugin;
use utils::scene_outline::SceneOutlinePlugin;

mod components;
mod entities;
mod particles;
mod ui;
mod utils;

#[derive(Component)]
pub struct Movement {
    /// Velocity
    pub vel: Vec3,
    /// Acceleration
    pub acc: Vec3,
    /// Maximum speed
    pub max_speed: Option<f32>,
    /// Friction must be between 0 and 1
    /// 0 means no friction, 1 that the object will stop immediately
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

fn movement_system(time: Res<Time>, mut query: Query<(&mut Transform, &mut Movement)>) {
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
        app.add_systems(Startup, scene_setup_3d)
            .add_systems(Update, movement_system);
    }
}

fn setup_physics(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.gravity = Vec3::ZERO;
}

fn scene_setup_3d(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut asteroid_spawn_events: EventWriter<AsteroidSpawnEvent>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Circle::new(4.0).into()),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
    });

    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 10000.0,
                color: Color::hex("fcd4b5").unwrap(),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 40.0, 0.0)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2 + 0.1)),
            ..default()
        },
        ToonShaderSun,
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Cube::new(10.0).into()),
            material: materials.add(Color::RED.into()),
            ..default()
        },
        Collider::cuboid(5.0, 5.0, 5.0),
        RigidBody::Fixed,
        Sensor,
        ActiveCollisionTypes::all(),
        ActiveEvents::COLLISION_EVENTS,
    ));

    for _ in 0..100 {
        asteroid_spawn_events.send(AsteroidSpawnEvent {
            position: Transform {
                translation: Vec3::new(
                    rand::random::<f32>() * 100.0 - 50.0,
                    0.0,
                    rand::random::<f32>() * 100.0 - 50.0,
                ),
                rotation: Quat::from_rotation_y(rand::random::<f32>() * std::f32::consts::PI * 2.0),
                ..default()
            },
            velocity: Velocity {
                linvel: Vec3::new(
                    rand::random::<f32>() - 0.5,
                    0.0,
                    rand::random::<f32>() - 0.5,
                ),
                angvel: Vec3::Y * (rand::random::<f32>() - 0.5),
            },
            size: rand::thread_rng().gen_range(0.5..2.0),
        });
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum AppState {
    #[default]
    Loading,
    Running,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: bevy::log::Level::INFO,
            ..default()
        }))
        .add_plugins((
            OutlinePlugin,
            AutoGenerateOutlineNormalsPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            ToonShaderPlugin,
            ObjPlugin,
            ScreenDiagnosticsPlugin::default(),
            ScreenFrameDiagnosticsPlugin,
            RoundUiPlugin,
        ))
        .add_state::<AppState>()
        .add_systems(Startup, setup_physics)
        .add_loading_state(
            LoadingState::new(AppState::Loading).continue_to_state(AppState::Running),
        )
        .add_plugins((
            ScenePlugin3D,
            EntitiesPlugin,
            ComponentsPlugin,
            ParticlesPlugin,
            SceneOutlinePlugin,
            UIPlugin,
        ))
        .run();
}
