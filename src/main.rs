#![allow(clippy::type_complexity)] // Query types can be really complex
#![feature(let_chains)]

#[macro_use]
extern crate rust_i18n;

i18n!();

use bevy::{log::LogPlugin, prelude::*, window::PresentMode};
use bevy_mod_outline::{AutoGenerateOutlineNormalsPlugin, OutlineBundle, OutlinePlugin};
use bevy_obj::ObjPlugin;
use bevy_rapier3d::prelude::*;
use bevy_round_ui::prelude::RoundUiPlugin;
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};
use bevy_toon_shader::{ToonShaderPlugin, ToonShaderSun};
use components::ComponentsPlugin;
use entities::EntitiesPlugin;
use materials::{MaterialsPlugin, outline::OutlineMaterial};
use particles::ParticlesPlugin;
use postprocessing::PostprocessingPlugin;
use states::{game_running, StatesPlugin, ON_GAME_STARTED};

use ui::UIPlugin;
use utils::{materials::default_outline, scene_outline::SceneOutlinePlugin};

mod components;
mod entities;
mod materials;
mod particles;
mod postprocessing;
mod ui;
mod utils;
mod states;

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
        app.add_systems(ON_GAME_STARTED, scene_setup_3d)
            .add_systems(
                Update,
                movement_system.run_if(game_running()),
            );
    }
}

fn setup_physics(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.gravity = Vec3::ZERO;
}

fn scene_setup_3d(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<OutlineMaterial>>,
) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
    });

    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 10000.0,
                color: Color::hex("fcd4b5").unwrap(),
                shadows_enabled: false,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 40.0, 0.0)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2 + 0.1)),
            ..default()
        },
        ToonShaderSun,
    ));

    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(shape::Cube::new(10.0).into()),
            material: materials.add(OutlineMaterial {
                color: Color::hex("ea6d25").unwrap(),
                ..default()
            }),
            ..default()
        },
        OutlineBundle {
            outline: default_outline(),
            ..default()
        },
    ));
}

fn main() {
    rust_i18n::set_locale("de");

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(LogPlugin {
                level: bevy::log::Level::INFO,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Space Game".into(),
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins((
        OutlinePlugin,
        AutoGenerateOutlineNormalsPlugin,
        RapierPhysicsPlugin::<NoUserData>::default(),
        ToonShaderPlugin,
        ObjPlugin,
        ScreenDiagnosticsPlugin {
            style: Style {
                top: Val::Px(10.),
                left: Val::Px(10.),
                ..default()
            },
            ..default()
        },
        ScreenFrameDiagnosticsPlugin,
        RoundUiPlugin,
    ))
    .add_systems(Startup, setup_physics)
    .add_plugins((
        StatesPlugin, 
        ScenePlugin3D,
        EntitiesPlugin,
        ComponentsPlugin,
        ParticlesPlugin,
        SceneOutlinePlugin,
        UIPlugin,
        PostprocessingPlugin,
        MaterialsPlugin,
    ));

    app.run();
}
